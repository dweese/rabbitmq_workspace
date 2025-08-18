// rabbitmq-info/src/lib.rs

// Import necessary dependencies
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

// Define the modules
pub mod api;
pub mod collector;
pub mod export;

// Define the data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub erlang_version: String,
    pub cluster_name: String,
    pub management_version: String,
    pub uptime: u64,
    pub node_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub name: String,
    pub vhost: String,
    pub exchange_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub name: String,
    pub vhost: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub exclusive: bool,
    pub arguments: Value,
    pub messages: Option<u64>,
    pub messages_ready: Option<u64>,
    pub messages_unacknowledged: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingInfo {
    pub source: String,
    pub destination: String,
    pub destination_type: String,
    pub routing_key: String,
    pub arguments: Value,
    pub vhost: String,
}

// Define any crate-level errors
#[derive(Debug, Error)]
pub enum InfoError {
    #[error("API error: {0}")]
    ApiError(#[from] api::ApiError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

// Re-export the collector's RabbitMQInfo for convenience
pub use collector::RabbitMQInfo;
