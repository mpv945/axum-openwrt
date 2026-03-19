use anyhow::Result;
use deadpool_redis::{Config, Connection, Pool, Runtime};
//use redis::AsyncCommands; // 必须导入
use deadpool_redis::redis::{AsyncCommands, RedisError};
use url::Url;

use crate::{
    common::{error::AppError, response::ApiResponse, result::AppResult},
    //state::app_state::AppState ,
};

#[derive(Clone)]
pub struct RedisClient {
    pool: Pool,
}

pub fn build_redis_url(
    host: &str,
    port: u16,
    password: &str,
) -> String {

    let mut url = Url::parse(&format!("redis://{}:{}/", host, port)).unwrap();

    url.set_password(Some(password)).unwrap();

    url.to_string()
}

impl RedisClient {

    pub async fn new(url: &str) -> Result<Self> {

        // 推荐：使用 from_url
        //let mut cfg = Config::from_url(url);

        /*println!("Connecting Redis: {}", url);

        let mut cfg = Config::default();

        cfg.url = Some(url.to_string());*/

        let mut cfg = Config::from_url(url);

        // 连接池配置
        /*cfg.pool = Some(PoolConfig {
            max_size: 32,
            ..Default::default()
        });*/

        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

        // 启动时测试连接
        {
            let mut conn = pool.get().await?;
            let _: String = deadpool_redis::redis::cmd("PING")
                .query_async(&mut conn)
                .await?;
        }

        println!("Redis pool initialized successfully");

        Ok(Self { pool })
    }

    async fn conn(&self) -> Result<Connection> {
        Ok(self.pool.get().await?)
    }

    /// 提供安全获取连接的方法
    pub async fn get_conn(&self) -> Result<Connection> {

        let conn = self.pool.get().await?;

        Ok(conn)
    }
    // KV SET
    pub async fn set(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let mut conn = self.conn().await?;

        conn.set::<_, _, ()>(key, value).await?;

        Ok(())
    }

    // KV GET
    pub async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {

        let mut conn = self.conn().await?;

        let val: Option<String> = conn.get(key).await?;

        Ok(val)
    }

    // DEL
    pub async fn del(&self, key: &str) -> anyhow::Result<()> {

        let mut conn = self.conn().await?;

        let _: () = conn.del(key).await?;

        Ok(())
    }

    // EXISTS
    pub async fn exists(&self, key: &str) -> anyhow::Result<bool> {

        let mut conn = self.conn().await?;

        let exists: bool = conn.exists(key).await?;

        Ok(exists)
    }

    // ================= TTL =================

    pub async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.set_ex(key, value, ttl_secs).await?;
        Ok(())
    }

    pub async fn expire(&self, key: &str, ttl_secs: u64) -> Result<bool> {
        let mut conn = self.conn().await?;
        Ok(conn.expire(key, ttl_secs as i64).await?)
    }

    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn = self.conn().await?;
        Ok(conn.ttl(key).await?)
    }

    // ---------------- PubSub publish ----------------

    pub async fn publish(&self, channel: &str, msg: &str) -> Result<i64> {

        let mut conn = self.conn().await?;

        let subscribers: i64 = conn.publish(channel, msg).await?;

        Ok(subscribers)
    }

    // ================= 分布式锁 =================

    /// 加锁（SET key value NX EX ttl）
    pub async fn lock(&self, key: &str, value: &str, ttl_secs: u64) -> Result<bool> {
        let mut conn = self.conn().await?;

        let res: Option<String> = deadpool_redis::redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await?;

        Ok(res.is_some())
    }

    /// 解锁（Lua保证原子性）
    pub async fn unlock(&self, key: &str, value: &str) -> Result<bool> {
        let mut conn = self.conn().await?;

        let result: i32 = deadpool_redis::redis::cmd("EVAL")
        .arg(r#"
        if redis.call("GET", KEYS[1]) == ARGV[1] then
            return redis.call("DEL", KEYS[1])
        else
            return 0
        end
    "#)
            .arg(1) // number of keys
            .arg(key)
            .arg(value)
            .query_async(&mut conn)
            .await?;

        Ok(result == 1)
    }

    // ================= Hash =================

    pub async fn h_set(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.hset(key, field, value).await?;
        Ok(())
    }

    pub async fn h_get(&self, key: &str, field: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        Ok(conn.hget(key, field).await?)
    }

    pub async fn h_get_all(&self, key: &str) -> Result<std::collections::HashMap<String, String>> {
        let mut conn = self.conn().await?;
        Ok(conn.hgetall(key).await?)
    }

    pub async fn h_del(&self, key: &str, field: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.hdel(key, field).await?;
        Ok(())
    }

    // ================= Set =================

    pub async fn s_add(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.sadd(key, member).await?;
        Ok(())
    }

    pub async fn s_members(&self, key: &str) -> Result<Vec<String>> {
        let mut conn = self.conn().await?;
        Ok(conn.smembers(key).await?)
    }

    pub async fn s_rem(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.srem(key, member).await?;
        Ok(())
    }

    // =========================
    // Redis Stream Producer
    // =========================

    pub async fn stream_add(
        &self,
        stream: &str,
        field: &str,
        value: &str,
    ) -> Result<String> {

        let mut conn = self.conn().await?;

        let id: String = deadpool_redis::redis::cmd("XADD")
            .arg(stream)
            .arg("*")
            .arg(field)
            .arg(value)
            .query_async(&mut conn)
            .await?;

        Ok(id)
    }

    // =========================
    // 自动创建 Consumer Group
    // =========================

    pub async fn ensure_consumer_group(
        &self,
        stream: &str,
        group: &str,
    ) -> Result<()> {

        let mut conn = self.conn().await?;

        let res: Result<(), RedisError> =
            deadpool_redis::redis::cmd("XGROUP")
                .arg("CREATE")
                .arg(stream)
                .arg(group)
                .arg("0")
                .arg("MKSTREAM")
                .query_async(&mut conn)
                .await;

        match res {

            Ok(_) => {
                println!("created consumer group: {}", group);
            }

            Err(e) => {

                // BUSYGROUP 表示已经存在
                if e.to_string().contains("BUSYGROUP") {

                    println!("consumer group already exists: {}", group);

                } else {

                    return Err(e.into());
                }
            }
        }

        Ok(())
    }


    // MQTT 幂等性操作
    // 检查是否已处理
    pub async fn is_processed(
        &self,
        client_id: &str,
        message_id: &str,
    ) -> anyhow::Result<bool> {
        //let mut conn = self.conn().await?;
        let key = format!("mqtt:msg:{}:{}", client_id, message_id);
        //let exists: bool = conn.exists(key).await?;
        let exists: bool = self.exists(&key).await?;
        Ok(exists)
    }

    // 标记已处理（带TTL防止无限增长）
    pub async fn mark_processed(
        &self,
        client_id: &str,
        message_id: &str,
    ) -> anyhow::Result<()> {
        //let mut conn = self.client.get_async_connection().await?;
        let key = format!("mqtt:msg:{}:{}", client_id, message_id);

        //self.set_ex("k1", "v1", 60).await?;
        let _: () = self.set_ex(&key, "1", 3600 * 24).await?; // 1天
        Ok(())
    }
}