use std::sync::{Arc, RwLock};
use sqlx::MySqlPool;
use crate::mqtt::producer::MqttProducer;
use crate::redis::redis_client::RedisClient;
use crate::utils::http_client::HttpClient;

// 共享状态
#[derive(Clone)]
pub struct AppState {

    pub config: Config,

    pub db: MySqlPool,

    pub producer: MqttProducer,

    pub redis: Arc<RedisClient>,

    pub http: Arc<HttpClient>,

    pub counter: Arc<RwLock<u64>>, // 改成 Arc<RwLock>

    // 数据库
    //pub db: sqlx::PgPool,

    // Redis
    //pub redis: redis::Client,
}

impl AppState {
    pub fn new(config_param: Config,
               db_client: MySqlPool,
               producer_client: MqttProducer,
               http_client: Arc<HttpClient>,
               redis_client: Arc<RedisClient>) -> Self {
        Self {
            counter: Arc::new(RwLock::new(0)), // 初始化,
            config: config_param,
            db:db_client,
            producer: producer_client,
            http: http_client,
            redis:redis_client,
        }
    }

    // 增加计数器
    pub fn increment_counter(&self) {
        // 获取写锁
        let mut counter = self.counter.write().unwrap();
        *counter += 1; // 增加计数器
    }
}

#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
}

// 定义用户结构
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user_id: i64,
    pub username: String,
}