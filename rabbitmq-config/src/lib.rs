// rabbitmq-config/src/lib.rs

use serde::{Serialize, Deserialize};

// Module declarations
mod client;
mod config;
mod error;
mod topology;
mod models; // Include the new models module

// Re-exports of core types
pub use client::RabbitMQClient;
pub use config::RabbitMQConfig;
pub use error::RabbitMQError;

// Re-export the models needed by the UI
pub use models::{
    MessageProperties,
    RabbitMQMessage,
    ExchangeInfo,
};

// If QueueInfo is not already defined in your models,
// you can include it here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub name: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub exclusive: bool,
}

impl Default for QueueInfo {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            durable: true,
            auto_delete: false,
            exclusive: false,
        }
    }
}