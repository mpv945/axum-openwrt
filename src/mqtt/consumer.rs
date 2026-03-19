use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use crate::model::mqtt_message::MqttMessage;
use crate::redis::redis_client::RedisClient;

pub struct MqttConsumer {
    redis: RedisClient,
}

impl MqttConsumer {
    pub async fn start(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        redis: RedisClient,
    ) {

        let mut mqttoptions = MqttOptions::new("consumer_id_haijun", host, port);
        mqttoptions.set_credentials(username, password);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 100);

        client
            .subscribe("test/topic", QoS::AtLeastOnce)
            .await
            .unwrap();

        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(p))) => {
                    let payload = p.payload.to_vec();

                    if let Ok(msg) = serde_json::from_slice::<MqttMessage>(&payload) {
                        let store_clone = redis.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_message(store_clone, msg).await {
                                tracing::error!("handle error: {:?}", e);
                            }
                        });
                    }
                }
                Err(e) => {
                    tracing::error!("MQTT error: {:?}", e);
                }
                _ => {}
            }
        }
    }
}

// 核心处理逻辑（幂等 + ACK）
async fn handle_message(
    redis: RedisClient,
    msg: MqttMessage,
) -> anyhow::Result<()> {
    if redis
        .is_processed(&msg.client_id, &msg.message_id)
        .await?
    {
        tracing::info!("duplicate message skipped");
        return Ok(());
    }

    // ===== 业务处理 =====
    process_business(&msg).await?;

    // ===== ACK（标记完成）=====
    redis
        .mark_processed(&msg.client_id, &msg.message_id)
        .await?;

    Ok(())
}

async fn process_business(msg: &MqttMessage) -> anyhow::Result<()> {
    tracing::info!("processing: {:?}", msg);
    Ok(())
}