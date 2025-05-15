// For rabbitmq-config/src/client.rs
use crate::*;
use lapin::{
    options::*,
    types::{FieldTable, ShortString, AMQPValue},
    Connection, ConnectionProperties, Channel, BasicProperties,
    ExchangeKind,
};
use tokio::sync::Mutex; // Change this from std::sync::Mutex

use amq_protocol_uri::AMQPUri;
use std::sync::{Arc};
use std::str::FromStr;
use thiserror::Error;
use std::path::Path;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum RabbitMQError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("Exchange error: {0}")]
    ExchangeError(String),

    #[error("Binding error: {0}")]
    BindingError(String),

    #[error("Publish error: {0}")]
    PublishError(String),

    #[error("Consume error: {0}")]
    ConsumeError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid exchange type: {0}")]
    InvalidExchangeType(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("JSON serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Lapin error: {0}")]
    LapinError(#[from] lapin::Error),

    #[error("IO error: {0}")]
    StdIoError(#[from] std::io::Error),
}


#[derive(Debug, Clone)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub delivery_mode: Option<u8>,
    pub priority: Option<u8>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    pub expiration: Option<String>,
    pub message_id: Option<String>,
    pub timestamp: Option<bool>,
    pub user_id: Option<String>,
    pub app_id: Option<String>,
}

impl Default for MessageProperties {
    fn default() -> Self {
        Self {
            content_type: Some("text/plain".to_string()),
            content_encoding: None,
            delivery_mode: Some(1), // 1 = non-persistent, 2 = persistent
            priority: None,
            correlation_id: None,
            reply_to: None,
            expiration: None,
            message_id: None,
            timestamp: None,
            user_id: None,
            app_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RabbitMQMessage {
    pub exchange: String,
    pub routing_key: String,
    pub payload: String,
    pub properties: Option<MessageProperties>,
}

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub name: String,
    pub durable: bool,
    pub exclusive: bool,
    pub auto_delete: bool,
}

#[derive(Debug, Clone)]
pub struct ExchangeInfo {
    pub name: String,
    pub kind: String,
    pub durable: bool,
    pub auto_delete: bool,
}




pub struct RabbitMQClient {
    connection: Connection,
    channel: Arc<tokio::sync::Mutex<Channel>>, // Explicitly qualify the type
    config: RabbitMQConfig,
    full_config: Option<RabbitMQFullConfig>,
}

impl RabbitMQClient {
    /// Create a new RabbitMQ client with the simple configuration
    pub async fn new(config: RabbitMQConfig) -> Result<Self, RabbitMQError> {
        let connection = Self::create_connection(&config.to_uri()).await?;
        let channel = Self::create_channel(&connection).await?;

        Ok(Self {
            connection,
            channel: Arc::new(Mutex::new(channel)),
            config,
            full_config: None,
        })
    }

    /// Create a new RabbitMQ client with the full configuration
    pub async fn new_with_full_config(config: RabbitMQFullConfig) -> Result<Self, RabbitMQError> {
        // Extract simple config for backward compatibility
        let simple_config = config.to_simple_config();

        // Create connection with advanced settings
        let connection = Self::create_connection_with_options(&config.connection).await?;

        // Create channel with configured settings
        let channel = Self::create_channel_with_options(&connection, &config.channels).await?;

        // Create client
        let client = Self {
            connection,
            channel: Arc::new(Mutex::new(channel)),
            config: simple_config,
            full_config: Some(config),
        };

        Ok(client)
    }

    /// Create a new RabbitMQ client from a configuration file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RabbitMQError> {
        let config = RabbitMQFullConfig::load_from_file(path)?;
        Self::new_with_full_config(config).await
    }

    /// Helper to create a connection with advanced options
    async fn create_connection_with_options(conn_config: &ConnectionConfig) -> Result<Connection, RabbitMQError> {
        let mut uri = format!(
            "amqp://{}:{}@{}:{}/{}",
            conn_config.username,
            conn_config.password,
            conn_config.host,
            conn_config.port,
            conn_config.vhost
        );

        // Add TLS if configured
        if conn_config.use_tls {
            uri = uri.replace("amqp://", "amqps://");
        }

        // Parse URI
        let amqp_uri = AMQPUri::from_str(&uri)
            .map_err(|e| RabbitMQError::ConnectionError(format!("Invalid AMQP URI: {}", e)))?;

        // Just use default ConnectionProperties for now
        let connection_properties = ConnectionProperties::default();

        // Connect with the URI
        let connection = Connection::connect_uri(amqp_uri, connection_properties).await?;

        Ok(connection)
    }

    /// Helper to create a channel with configured options
    async fn create_channel_with_options(
        connection: &Connection,
        channel_config: &ChannelConfig
    ) -> Result<Channel, RabbitMQError> {
        let channel = connection.create_channel().await?;

        // Set QoS (prefetch) with the correct signature
        // The first parameter should be the count, not size
        channel.basic_qos(
            channel_config.default_prefetch_count,
            BasicQosOptions {
                global: false
            }
        ).await?;

        // Enable publisher confirms if configured
        if channel_config.confirm_deliveries {
            channel.confirm_select(ConfirmSelectOptions::default()).await?;
        }

        Ok(channel)
    }

    // Close the connection
    pub async fn close(&self) -> Result<(), RabbitMQError> {
        self.connection.close(0, "Normal shutdown").await?;
        Ok(())
    }

    // List available queues
    pub async fn list_queues(&self) -> Result<Vec<String>, RabbitMQError> {
        // This would typically involve an admin API call
        // For simplicity, we'll return an empty vector
        // In a real implementation, you might use the RabbitMQ Management API
        Ok(Vec::new())
    }

    // List available exchanges
    pub async fn list_exchanges(&self) -> Result<Vec<String>, RabbitMQError> {
        // Similar to list_queues, this would use the Management API
        Ok(Vec::new())
    }

    // Publish a message
    pub async fn publish_message(&self, message: &RabbitMQMessage) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;

        // Set up basic properties
        let mut props = BasicProperties::default();

        if let Some(properties) = &message.properties {
            if let Some(content_type) = &properties.content_type {
                props = props.with_content_type(content_type.clone().into());
            }

            if let Some(delivery_mode) = properties.delivery_mode {
                props = props.with_delivery_mode(delivery_mode);
            }

            // Add other properties as needed
        }

        // Publish message
        channel.basic_publish(
            &message.exchange,
            &message.routing_key,
            BasicPublishOptions::default(),
            message.payload.as_bytes(),
            props,
        ).await?;

        Ok(())
    }

    // Declare a queue
    pub async fn declare_queue(&self, queue_info: &QueueInfo) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;

        channel.queue_declare(
            &queue_info.name,
            QueueDeclareOptions {
                durable: queue_info.durable,
                exclusive: queue_info.exclusive,
                auto_delete: queue_info.auto_delete,
                ..Default::default()
            },
            FieldTable::default(),
        ).await?;

        Ok(())
    }

    // Declare an exchange
    pub async fn declare_exchange(&self, exchange_info: &ExchangeInfo) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;

        // Convert string kind to ExchangeKind
        let kind = match exchange_info.kind.as_str() {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            "topic" => ExchangeKind::Topic,
            "headers" => ExchangeKind::Headers,
            _ => return Err(RabbitMQError::InvalidExchangeType(exchange_info.kind.clone())),
        };

        channel.exchange_declare(
            &exchange_info.name,
            kind,
            ExchangeDeclareOptions {
                durable: exchange_info.durable,
                auto_delete: exchange_info.auto_delete,
                ..Default::default()
            },
            FieldTable::default(),
        ).await?;

        Ok(())
    }

    /// Initialize infrastructure (exchanges, queues, bindings) from configuration
    pub async fn initialize_infrastructure(&self) -> Result<(), RabbitMQError> {
        if let Some(full_config) = &self.full_config {
            // Create exchanges
            for exchange_config in &full_config.exchanges {
                self.declare_exchange_from_config(exchange_config).await?;
            }

            // Create queues
            for queue_config in &full_config.queues {
                self.declare_queue_from_config(queue_config).await?;
            }

            // Create bindings
            for binding_config in &full_config.bindings {
                self.bind_queue_from_config(binding_config).await?;
            }

            Ok(())
        } else {
            Err(RabbitMQError::ConfigError("Full configuration not available".to_string()))
        }
    }

    /// Declare exchange from configuration
    async fn declare_exchange_from_config(&self, config: &ExchangeConfig) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;  // Use await instead of unwrap for mutex

        // Convert string exchange type to ExchangeKind
        let exchange_kind = match config.exchange_type.as_str() {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            "topic" => ExchangeKind::Topic,
            "headers" => ExchangeKind::Headers,
            _ => return Err(RabbitMQError::ConfigError(format!(
                "Invalid exchange type: {}", config.exchange_type
            ))),
        };

        // Convert arguments HashMap to FieldTable
        let args = Self::convert_arguments_to_field_table(&config.arguments)?;

        channel.exchange_declare(
            &config.name,
            exchange_kind,
            ExchangeDeclareOptions {
                durable: config.durable,
                auto_delete: config.auto_delete,
                internal: config.internal,
                ..Default::default()
            },
            args,
        ).await?;

        Ok(())
    }

    /// Declare queue from configuration
    async fn declare_queue_from_config(&self, config: &QueueConfig) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;  // Use await instead of unwrap for mutex

        // Convert arguments HashMap to FieldTable
        let args = Self::convert_arguments_to_field_table(&config.arguments)?;

        channel.queue_declare(
            &config.name,
            QueueDeclareOptions {
                durable: config.durable,
                exclusive: config.exclusive,
                auto_delete: config.auto_delete,
                ..Default::default()
            },
            args,
        ).await?;

        Ok(())
    }

    /// Bind queue from configuration
    async fn bind_queue_from_config(&self, config: &BindingConfig) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;  // Use await instead of unwrap for mutex

        // Convert arguments HashMap to FieldTable
        let args = Self::convert_arguments_to_field_table(&config.arguments)?;

        channel.queue_bind(
            &config.queue,
            &config.exchange,
            &config.routing_key,
            QueueBindOptions::default(),
            args,
        ).await?;

        Ok(())
    }

    /// Helper function to convert HashMap arguments to FieldTable
    fn convert_arguments_to_field_table(
        args: &HashMap<String, serde_json::Value>
    ) -> Result<FieldTable, RabbitMQError> {
        let mut field_table = FieldTable::default();

        for (key, value) in args {
            // Convert key to ShortString
            let k: ShortString = key.clone().into();

            // Handle different value types
            if let Some(string_val) = value.as_str() {
                // For strings, we need to use LongString
                field_table.insert(k, AMQPValue::LongString(string_val.to_string().into()));
            } else if let Some(num_val) = value.as_i64() {
                // For integers
                field_table.insert(k, AMQPValue::LongLongInt(num_val));
            } else if let Some(bool_val) = value.as_bool() {
                // For booleans
                field_table.insert(k, AMQPValue::Boolean(bool_val));
            } else {
                // For other types, convert to string representation
                let string_val = value.to_string();
                field_table.insert(k, AMQPValue::LongString(string_val.into()));
            }
        }

        Ok(field_table)
    }

    /// Get configuration
    pub fn config(&self) -> &RabbitMQConfig {
        &self.config
    }

    /// Get full configuration if available
    pub fn full_config(&self) -> Option<&RabbitMQFullConfig> {
        self.full_config.as_ref()
    }

    /// Create a basic connection using a URI string
    async fn create_connection(uri: &str) -> Result<Connection, RabbitMQError> {
        Connection::connect(
            uri,
            ConnectionProperties::default(),
        ).await.map_err(RabbitMQError::from)
    }

    /// Create a basic channel
    async fn create_channel(connection: &Connection) -> Result<Channel, RabbitMQError> {
        connection.create_channel().await.map_err(RabbitMQError::from)
    }

    /// Bind a queue to an exchange
    pub async fn bind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;  // Use await instead of unwrap for mutex

        channel.queue_bind(
            queue,
            exchange,
            routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
            .await?;

        Ok(())
    }
}