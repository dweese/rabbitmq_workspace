// rabbitmq-config/src/config.rs
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A flattened, simple config struct for use by client applications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQConfig {
    pub host: String,
    pub amqp_port: u16,
    pub management_port: u16,
    pub username: String,
    pub password: String,
    pub vhost: String,
}

impl Default for RabbitMQConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            amqp_port: 5672,
            management_port: 15672,
            username: "guest".to_string(),
            password: "guest".to_string(),
            vhost: "/".to_string(),
        }
    }
}

impl RabbitMQConfig {
    /// Converts the configuration into a valid AMQP URI string.
    pub fn to_uri(&self) -> String {
        let vhost_encoded = if self.vhost.is_empty() || self.vhost == "/" {
            "/".to_string()
        } else {
            format!("/{}", utf8_percent_encode(&self.vhost, NON_ALPHANUMERIC))
        };

        format!(
            "amqp://{}:{}@{}:{}{}",
            self.username, self.password, self.host, self.amqp_port, vhost_encoded
        )
    }
}

/// A comprehensive, hierarchical configuration for a RabbitMQ setup.
/// This struct is designed to be deserialized from a single configuration file (e.g., TOML).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQFullConfig {
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub channel: ChannelConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub exchanges: Vec<ExchangeConfig>,
    #[serde(default)]
    pub queues: Vec<QueueConfig>,
    #[serde(default)]
    pub bindings: Vec<BindingConfig>,
    #[serde(default)]
    pub consumers: Vec<ConsumerConfig>,
    #[serde(default)]
    pub publishers: Vec<PublisherConfig>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

impl Default for RabbitMQFullConfig {
    fn default() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            channel: ChannelConfig::default(),
            logging: LoggingConfig::default(),
            exchanges: vec![],
            queues: vec![],
            bindings: vec![],
            consumers: vec![],
            publishers: vec![],
            retry: None,
        }
    }
}


// --- Configuration Sub-structs ---

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    pub level: String,
    #[serde(default)]
    pub include_connection_events: bool,
    #[serde(default)]
    pub include_channel_events: bool,
    #[serde(default)]
    pub include_consumer_events: bool,
    #[serde(default)]
    pub include_publisher_events: bool,
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub amqp_port: u16,
    pub management_port: u16,
    pub vhost: String,
    pub username: String,
    pub password: Option<String>,
    #[serde(default)]
    pub connection_timeout_ms: u32,
    #[serde(default)]
    pub heartbeat_interval_sec: u32,
    #[serde(default)]
    pub connection_name: String,
    #[serde(default)]
    pub use_tls: bool,
    #[serde(default)]
    pub tls_options: Option<TlsOptions>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            amqp_port: 5672,
            management_port: 15672,
            vhost: "/".to_string(),
            username: "guest".to_string(),
            password: None,
            connection_timeout_ms: 10000,
            heartbeat_interval_sec: 60,
            connection_name: "rabbitmq_app".to_string(),
            use_tls: false,
            tls_options: None,
        }
    }
}


/// TLS configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TlsOptions {
    #[serde(default)]
    pub verify_hostname: bool,
    pub ca_cert_path: String,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelConfig {
    #[serde(default)]
    pub default_prefetch_count: u16,
    #[serde(default)]
    pub default_prefetch_size: u32,
    #[serde(default)]
    pub confirm_deliveries: bool,
}

/// Exchange configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExchangeConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub exchange_type: String,
    #[serde(default)]
    pub durable: bool,
    #[serde(default)]
    pub auto_delete: bool,
    #[serde(default)]
    pub internal: bool,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Queue configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueConfig {
    pub name: String,
    #[serde(default)]
    pub durable: bool,
    #[serde(default)]
    pub exclusive: bool,
    #[serde(default)]
    pub auto_delete: bool,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Binding configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BindingConfig {
    pub exchange: String,
    pub queue: String,
    pub routing_key: String,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsumerConfig {
    pub queue: String,
    pub consumer_tag: String,
    #[serde(default)]
    pub exclusive: bool,
    #[serde(default)]
    pub no_local: bool,
    #[serde(default)]
    pub no_ack: bool,
    #[serde(default)]
    pub prefetch_count: u16,
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Publisher configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublisherConfig {
    pub exchange: String,
    pub routing_key: String,
    #[serde(default)]
    pub mandatory: bool,
    #[serde(default)]
    pub immediate: bool,
    #[serde(default)]
    pub persistence: bool,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub content_encoding: String,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub correlation_id: String,
    #[serde(default)]
    pub reply_to: String,
    #[serde(default)]
    pub expiration: String,
    #[serde(default)]
    pub message_id: String,
    #[serde(default)]
    pub timestamp: bool,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub delivery_mode: u8,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_interval_ms: u32,
    pub multiplier: f64,
    pub max_interval_ms: u32,
    pub randomization_factor: f64,
}
