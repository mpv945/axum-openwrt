use anyhow::Result;
use tokio::time::{sleep, Duration};

use deadpool_redis::redis::Value;


#[derive(Debug)]
pub struct StreamMessage {
    pub stream: String,
    pub id: String,
    pub fields: Vec<(String, String)>,
}

use crate::redis::redis_client::RedisClient;

pub async fn start_stream_worker(redis: RedisClient) -> Result<()> {

    let stream = "orders";
    let group = "order_group";
    let consumer = "consumer_1";

    // 自动创建消费组
    redis.ensure_consumer_group(stream, group).await?;

    loop {

        let mut conn = redis.get_conn().await?;

        let res: deadpool_redis::redis::RedisResult<Value> =
            deadpool_redis::redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(group)
                .arg(consumer)
                .arg("COUNT")
                .arg(10)
                .arg("BLOCK")
                .arg(5000)
                .arg("STREAMS")
                .arg(stream)
                .arg(">")
                .query_async(&mut conn)
                .await;

        match res {

            Ok(v) => {

                if v != Value::Nil {

                    /*println!("stream msg: {:?}", v);
                    parse_stream_value(v);

                    // TODO 业务处理*/


                    // 解析成对象
                    let messages = parse_stream_messages(v);

                    for msg in messages {

                        println!("stream={}, id={}", msg.stream, msg.id);

                        for (k, v) in msg.fields {
                            println!("{} => {}", k, v);
                        }

                        // TODO 业务处理

                        let mut conn = redis.get_conn().await?;

                        let _: i32 = deadpool_redis::redis::cmd("XACK")
                            .arg(stream)
                            .arg(group)
                            .arg(&msg.id)
                            .query_async(&mut conn)
                            .await?;
                    }
                }
            }

            Err(e) => {

                println!("stream error: {}", e);

                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub fn parse_stream_messages(v: Value) -> Vec<StreamMessage> {

    let mut result = Vec::new();

    let streams = match v {
        Value::Array(s) => s,
        _ => return result,
    };

    for stream in streams {

        let stream_data = match stream {
            Value::Array(sd) => sd,
            _ => continue,
        };

        if stream_data.len() != 2 {
            continue;
        }

        let stream_name = match &stream_data[0] {
            Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
            _ => continue,
        };

        let entries = match &stream_data[1] {
            Value::Array(e) => e,
            _ => continue,
        };

        for entry in entries {

            let entry_data = match entry {
                Value::Array(ed) => ed,
                _ => continue,
            };

            if entry_data.len() != 2 {
                continue;
            }

            let id = match &entry_data[0] {
                Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
                _ => continue,
            };

            let kvs = match &entry_data[1] {
                Value::Array(kvs) => kvs,
                _ => continue,
            };

            let mut fields = Vec::new();

            for pair in kvs.chunks(2) {

                if pair.len() != 2 {
                    continue;
                }

                let field = match &pair[0] {
                    Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
                    _ => continue,
                };

                let value = match &pair[1] {
                    Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
                    _ => continue,
                };

                fields.push((field, value));
            }

            result.push(StreamMessage {
                stream: stream_name.clone(),
                id,
                fields,
            });
        }
    }

    result
}

fn parse_stream_value(v: Value) {

    if let Value::Array(streams) = v {

        for stream in streams {

            if let Value::Array(stream_data) = stream {

                let stream_name = &stream_data[0];

                let entries = &stream_data[1];

                println!("stream = {:?}", stream_name);

                if let Value::Array(items) = entries {

                    for item in items {

                        if let Value::Array(entry) = item {

                            let id = &entry[0];
                            let fields = &entry[1];

                            println!("id = {:?}", id);

                            if let Value::Array(kvs) = fields {

                                for pair in kvs.chunks(2) {

                                    let field = &pair[0];
                                    let value = &pair[1];

                                    println!("{:?} => {:?}", field, value);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}