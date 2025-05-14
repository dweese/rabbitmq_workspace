//! RabbitMQ configuration and client management for the RabbitMQ UI application.
use lapin::{
    options::{
        BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
// Import AMQPUri from the correct location
// use amq_protocol_uri::AMQPUri;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Connection configuration for RabbitMQ
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

impl RabbitMQConfig {
    pub fn to_uri(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.vhost
        )
    }
}

/// Custom error types for RabbitMQ operations
#[derive(Error, Debug)]
pub enum RabbitMQError {
    #[error("Failed to connect to RabbitMQ: {0}")]
    ConnectionError(#[from] lapin::Error),

    #[error("Failed to create channel: {0}")]
    ChannelError(String),

    #[error("Failed to declare queue: {0}")]
    QueueError(String),

    #[error("Failed to declare exchange: {0}")]
    ExchangeError(String),

    #[error("Failed to bind queue: {0}")]
    BindError(String),

    #[error("Failed to publish message: {0}")]
    PublishError(String),

    #[error("Failed to consume message: {0}")]
    ConsumeError(String),
}

/// Representation of a RabbitMQ message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQMessage {
    pub exchange: String,
    pub routing_key: String,
    pub payload: String,
    pub properties: Option<MessageProperties>,
}

/// Basic message properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub delivery_mode: Option<u8>, // 1 = non-persistent, 2 = persistent
    pub priority: Option<u8>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    pub expiration: Option<String>,
    pub message_id: Option<String>,
}

impl Default for MessageProperties {
    fn default() -> Self {
        Self {
            content_type: Some("application/json".to_string()),
            content_encoding: None,
            delivery_mode: Some(1), // non-persistent by default
            priority: None,
            correlation_id: None,
            reply_to: None,
            expiration: None,
            message_id: None,
        }
    }
}

/// Queue information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub name: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub exclusive: bool,
}

/// Exchange information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub name: String,
    pub kind: String, // direct, fanout, topic, headers
    pub durable: bool,
    pub auto_delete: bool,
}

/// Client for RabbitMQ operations
pub struct RabbitMQClient {
    connection: Arc<Connection>,
    channel: Arc<Mutex<Channel>>,
    config: RabbitMQConfig,
}

impl RabbitMQClient {
    /// Create a new RabbitMQ client with the given configuration
    pub async fn new(config: RabbitMQConfig) -> Result<Self, RabbitMQError> {
        // Use the Lapin URL-based connection without creating an AMQPUri
        // This avoids the need for AMQPUri parsing which has changed
        let connection = Connection::connect(
            &config.to_uri(),
            ConnectionProperties::default(),
        ).await?;

        let channel = connection.create_channel().await?;

        Ok(Self {
            connection: Arc::new(connection),
            channel: Arc::new(Mutex::new(channel)),
            config,
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &RabbitMQConfig {
        &self.config
    }

    /// Declare a queue
    pub async fn declare_queue(&self, queue_info: &QueueInfo) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;

        channel.queue_declare(
            &queue_info.name,
            QueueDeclareOptions {
                durable: queue_info.durable,
                auto_delete: queue_info.auto_delete,
                exclusive: queue_info.exclusive,
                ..Default::default()
            },
            FieldTable::default(),
        )
            .await
            .map_err(|e| RabbitMQError::QueueError(e.to_string()))?;

        Ok(())
    }

    /// Declare an exchange
    pub async fn declare_exchange(&self, exchange_info: &ExchangeInfo) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;

        let kind = match exchange_info.kind.as_str() {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            "topic" => ExchangeKind::Topic,
            "headers" => ExchangeKind::Headers,
            _ => return Err(RabbitMQError::ExchangeError("Invalid exchange type".to_string())),
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
        )
            .await
            .map_err(|e| RabbitMQError::ExchangeError(e.to_string()))?;

        Ok(())
    }

    /// Bind a queue to an exchange
    pub async fn bind_queue(&self, queue_name: &str, exchange_name: &str, routing_key: &str) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;

        channel.queue_bind(
            queue_name,
            exchange_name,
            routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
            .await
            .map_err(|e| RabbitMQError::BindError(e.to_string()))?;

        Ok(())
    }

    /// Publish a message to an exchange
    pub async fn publish_message(&self, message: &RabbitMQMessage) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;

        let props = if let Some(props) = &message.properties {
            let mut basic_props = BasicProperties::default();

            if let Some(content_type) = &props.content_type {
                basic_props = basic_props.with_content_type(content_type.clone().into());
            }

            if let Some(content_encoding) = &props.content_encoding {
                basic_props = basic_props.with_content_encoding(content_encoding.clone().into());
            }

            if let Some(delivery_mode) = props.delivery_mode {
                basic_props = basic_props.with_delivery_mode(delivery_mode);
            }

            if let Some(priority) = props.priority {
                basic_props = basic_props.with_priority(priority);
            }

            if let Some(correlation_id) = &props.correlation_id {
                basic_props = basic_props.with_correlation_id(correlation_id.clone().into());
            }

            if let Some(reply_to) = &props.reply_to {
                basic_props = basic_props.with_reply_to(reply_to.clone().into());
            }

            if let Some(expiration) = &props.expiration {
                basic_props = basic_props.with_expiration(expiration.clone().into());
            }

            if let Some(message_id) = &props.message_id {
                basic_props = basic_props.with_message_id(message_id.clone().into());
            }

            basic_props
        } else {
            BasicProperties::default()
        };

        channel.basic_publish(
            &message.exchange,
            &message.routing_key,
            BasicPublishOptions::default(),
            message.payload.as_bytes(),
            props,
        )
            .await
            .map_err(|e| RabbitMQError::PublishError(e.to_string()))?;

        Ok(())
    }

    /// Close the connection
    pub async fn close(&self) -> Result<(), RabbitMQError> {
        let channel = self.channel.lock().await;
        channel.close(0, "Closing channel").await
            .map_err(|e| RabbitMQError::ChannelError(e.to_string()))?;

        self.connection.close(0, "Closing connection").await
            .map_err(|e| RabbitMQError::ConnectionError(e))?;

        Ok(())
    }



/// Get a list of queues (Note: This is a simplified example - in a real app, you'd use the RabbitMQ HTTP API for this)
    pub async fn list_queues(&self) -> Result<Vec<String>, RabbitMQError> {
        // In a real application, you would use the RabbitMQ HTTP Management API
        // This is just a placeholder that would need to be implemented with an HTTP client
        Ok(vec!["This functionality requires RabbitMQ HTTP API".to_string()])
    }

    /// Get a list of exchanges (Note: This is a simplified example - in a real app, you'd use the RabbitMQ HTTP API for this)
    pub async fn list_exchanges(&self) -> Result<Vec<String>, RabbitMQError> {
        // In a real application, you would use the RabbitMQ HTTP Management API
        // This is just a placeholder that would need to be implemented with an HTTP client
        Ok(vec!["This functionality requires RabbitMQ HTTP API".to_string()])
    }
}