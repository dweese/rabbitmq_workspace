// rabbitmq-config/src/config.rs
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub vhost: String,
}

impl Default for RabbitMQConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5672,
            username: "guest".to_string(),
            password: "guest".to_string(),
            vhost: "/".to_string(),
        }
    }
}


// Keep your existing code for RabbitMQFullConfig and other structures here
// For rabbitmq-config/src/config.rs
//use crate::error::RabbitMQError;  // Import from error module, not client

//use serde::{Deserialize, Serialize};
//use std::fs;
//use std::io::{ BufReader, BufWriter};
//use std::path::Path;
//use std::collections::HashMap;

// For config.rs - add this struct if it's missing

// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub include_connection_events: bool,
    pub include_channel_events: bool,
    pub include_consumer_events: bool,
    pub include_publisher_events: bool,
}



// Legacy simple config (for backward compatibility)
//#[derive(Debug, Clone, Serialize, Deserialize)]
//pub struct RabbitMQConfig {
//    pub host: String,
//    pub port: u16,
//    pub username: String,
//    pub password: String,
//    pub vhost: String,
//}

impl RabbitMQConfig {
    pub fn to_uri(&self) -> String {
        // Special case for the default vhost "/"
        if self.vhost == "/" {
            format!(
                "amqp://{}:{}@{}:{}{}",
                self.username, self.password, self.host, self.port, self.vhost
            )
        } else {
            format!(
                "amqp://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.vhost
            )
        }
    }

    pub fn from_connection_config(conn: &ConnectionConfig) -> Self {
        Self {
            host: conn.host.clone(),
            port: conn.port,
            username: conn.username.clone(),
            password: conn.password.clone(),
            vhost: conn.vhost.clone(),
        }
    }
}

// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub vhost: String,
    pub username: String,
    pub password: String,
    pub connection_timeout_ms: u32,
    pub heartbeat_interval_sec: u32,
    pub connection_name: String,
    pub use_tls: bool,
    pub tls_options: TlsOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsOptions {
    pub verify_hostname: bool,
    pub ca_cert_path: String,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub default_prefetch_count: u16,
    pub default_prefetch_size: u32,
    pub confirm_deliveries: bool,
}

// Exchange configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub exchange_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

// Queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub name: String,
    pub durable: bool,
    pub exclusive: bool,
    pub auto_delete: bool,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

// Binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingConfig {
    pub exchange: String,
    pub queue: String,
    pub routing_key: String,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

// Consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    pub queue: String,
    pub consumer_tag: String,
    pub exclusive: bool,
    pub no_local: bool,
    pub no_ack: bool,
    pub prefetch_count: u16,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

// Publisher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherConfig {
    pub exchange: String,
    pub routing_key: String,
    pub mandatory: bool,
    pub immediate: bool,
    pub persistence: bool,
    pub content_type: String,
    pub content_encoding: String,
    pub priority: u8,
    pub correlation_id: String,
    pub reply_to: String,
    pub expiration: String,
    pub message_id: String,
    pub timestamp: bool,
    pub user_id: String,
    pub app_id: String,
    pub delivery_mode: u8,
}

// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_interval_ms: u32,
    pub multiplier: f64,
    pub max_interval_ms: u32,
    pub randomization_factor: f64,
}