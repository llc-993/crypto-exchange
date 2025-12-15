use serde::{Deserialize, Serialize};

/// MQTT消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttMsg {
    /// 主题
    pub topic: String,
    /// 消息主体
    pub payload: String,
}

impl MqttMsg {
    pub fn new(topic: String, payload: String) -> Self {
        Self { topic, payload }
    }
}
