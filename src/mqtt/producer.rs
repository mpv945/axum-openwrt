use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use crate::model::mqtt_message::MqttMessage;

#[derive(Clone)]
pub struct MqttProducer {
    client: AsyncClient,
}

impl MqttProducer {
    pub async fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Self {
        let mut mqttoptions = MqttOptions::new("producer", host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));

        // 账号密码认证
        mqttoptions.set_credentials(username, password);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 100);

        // 后台驱动
        tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    tracing::error!("MQTT error: {:?}", e);
                }
            }
        });

        Self { client }
    }

    pub async fn publish(&self, msg: &MqttMessage) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(msg)?;

        self.client
            .publish(msg.topic.clone(), QoS::AtLeastOnce, false, payload)
            .await?;

        Ok(())
    }
}