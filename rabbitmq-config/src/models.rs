// rabbitmq-config/src/models.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub delivery_mode: Option<u8>,
}

impl Default for MessageProperties {
    fn default() -> Self {
        Self {
            content_type: None,
            content_encoding: None,
            delivery_mode: Some(1), // Non-persistent by default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQMessage {
    pub exchange: String,
    pub routing_key: String,
    pub payload: String,
    pub properties: Option<MessageProperties>,
}

// UI-specific exchange info (with 'kind' instead of 'exchange_type')
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub name: String,
    pub kind: String,
    pub durable: bool,
    pub auto_delete: bool,
}