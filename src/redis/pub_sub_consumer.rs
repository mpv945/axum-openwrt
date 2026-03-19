use deadpool_redis::redis::{ Client};
use anyhow::Result;
use tokio_stream::StreamExt; // <- 必须引入

pub async fn pub_sub_listener(redis_url: String, channel: &str) -> Result<()> {
    // 创建 Redis Client
    let client = Client::open(redis_url)?;

    // 获取 MultiplexedConnection
    //let mut conn = client.get_multiplexed_async_connection().await?;

    // 构造异步 PubSub
    let mut pubsub =  client.get_async_pubsub().await?;
    pubsub.subscribe(channel).await?;
    println!("Subscribed to Redis channel: {}", channel);
    // 获取异步消息流
    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        // msg 类型是 redis::Msg
        let payload: String = msg.get_payload()?;       // 消息内容
        let channel_name = msg.get_channel_name();      // 订阅的频道

        println!("[{}] {}", channel_name, payload);

        // TODO: 这里可以做你的业务处理
    }

    Ok(())
}