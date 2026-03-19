use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttMessage {
    pub client_id: String,
    pub message_id: String,
    pub topic: String,
    pub payload: String,
    pub timestamp: i64,
}