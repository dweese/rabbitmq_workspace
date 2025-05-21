use std::sync::Arc;
use lapin::{
    options::{
        QueueDeclareOptions, ExchangeDeclareOptions, BasicPublishOptions,
        QueueDeleteOptions, ExchangeDeleteOptions,
    },
    types::FieldTable,
    BasicProperties, Connection, Channel, ConnectionProperties, ExchangeKind,
};
use amq_protocol_uri::AMQPUri;
use futures_util::stream::StreamExt;
use tokio::time::timeout;
use std::time::Duration;
use thiserror::Error;
use crate::config::{RabbitMQConfig, ConnectionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Error types
#[derive(Debug, Error)]
pub enum RabbitMQError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Queue operation error: {0}")]
    QueueError(String),

    #[error("Exchange operation error: {0}")]
    ExchangeError(String),

    #[error("Message publish error: {0}")]
    PublishError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Lapin error: {0}")]
    LapinError(#[from] lapin::Error),  // Add #[from] here if it's missing
}


// Message properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    pub expiration: Option<String>,
    pub message_id: Option<String>,
    pub timestamp: Option<u64>,
    pub user_id: Option<String>,
    pub app_id: Option<String>,
    pub delivery_mode: Option<u8>, // 2 for persistent, 1 for non-persistent
}

impl Default for MessageProperties {
    fn default() -> Self {
        Self {
            content_type: Some("application/json".to_string()),
            content_encoding: None,
            correlation_id: None,
            reply_to: None,
            expiration: None,
            message_id: None,
            timestamp: None,
            user_id: None,
            app_id: Some("rabbitmq-ui".to_string()),
            delivery_mode: Some(2), // Persistent by default
        }
    }
}

// RabbitMQ message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQMessage {
    pub exchange: String,
    pub routing_key: String,
    pub payload: String,
    pub properties: Option<MessageProperties>,
}

// Queue info for declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub name: String,
    pub durable: bool,
    pub exclusive: bool,
    pub auto_delete: bool,
}

// Exchange info for declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub name: String,
    pub kind: String,
    pub durable: bool,
    pub auto_delete: bool,
}

// The main client for RabbitMQ operations
pub struct RabbitMQClient {
    connection: Connection,
    channel: Channel,
    config: RabbitMQConfig,
}

impl RabbitMQClient {
    // Create a new client and connect to RabbitMQ
    pub async fn new(config: RabbitMQConfig) -> Result<Self, RabbitMQError> {
        let uri = config.to_uri();
        
        // Parse the AMQP URI
        let amqp_uri: AMQPUri = uri.parse()
            .map_err(|e| RabbitMQError::ConnectionError(format!("Invalid URI: {}", e)))?;
        
        // Set up connection properties with methods available in lapin 2.5.3
        let connection_props = ConnectionProperties::default()
            .with_connection_name("rust-rabbitmq-client".into());
        
        // Connect with timeout
        let connect_timeout = Duration::from_secs(10);
        let connection_future = Connection::connect_uri(amqp_uri, connection_props);
        let connection = timeout(connect_timeout, connection_future)
            .await
            .map_err(|_| RabbitMQError::TimeoutError("Connection timeout".to_string()))?
            .map_err(|e| RabbitMQError::ConnectionError(e.to_string()))?;
        
        // Create a channel
        let channel = connection.create_channel()
            .await
            .map_err(|e| RabbitMQError::ChannelError(e.to_string()))?;
        
        Ok(Self {
            connection,
            channel,
            config,
        })
    }
    
    // Close the connection
    pub async fn close(&self) -> Result<(), RabbitMQError> {
        // Close the channel first
        self.channel.close(0, "Normal shutdown")
            .await
            .map_err(|e| RabbitMQError::ChannelError(e.to_string()))?;
        
        // Then close the connection
        self.connection.close(0, "Normal shutdown")
            .await
            .map_err(|e| RabbitMQError::ConnectionError(e.to_string()))?;
            
        Ok(())
    }
    
    // List available queues
    pub async fn list_queues(&self) -> Result<Vec<String>, RabbitMQError> {
        // This is a simplified implementation - in a real application, 
        // you would use the RabbitMQ management API or other methods
        // to get a comprehensive list of queues
        
        // For now, return an empty list that you can populate manually
        // or in future implementations
        Ok(Vec::new())
    }
    
    // List available exchanges
    pub async fn list_exchanges(&self) -> Result<Vec<String>, RabbitMQError> {
        // This is a simplified implementation - in a real application, 
        // you would use the RabbitMQ management API or other methods
        // to get a comprehensive list of exchanges
        
        // For now, return a list with the default exchanges
        Ok(vec![
            "".to_string(),                // Default exchange
            "amq.direct".to_string(),      // Standard direct exchange
            "amq.fanout".to_string(),      // Standard fanout exchange
            "amq.topic".to_string(),       // Standard topic exchange
            "amq.headers".to_string(),     // Standard headers exchange
            "amq.match".to_string(),       // Standard match exchange
        ])
    }
    
    // Publish a message
    pub async fn publish_message(&self, message: &RabbitMQMessage) -> Result<(), RabbitMQError> {
        // Default publish options
        let options = BasicPublishOptions::default();
        
        // Convert our message properties to lapin's BasicProperties
        let properties = match &message.properties {
            Some(props) => {
                let mut lapin_props = BasicProperties::default();
                
                if let Some(content_type) = &props.content_type {
                    lapin_props = lapin_props.with_content_type(content_type.as_str().into());
                }
                
                if let Some(content_encoding) = &props.content_encoding {
                    lapin_props = lapin_props.with_content_encoding(content_encoding.as_str().into());
                }
                
                if let Some(correlation_id) = &props.correlation_id {
                    lapin_props = lapin_props.with_correlation_id(correlation_id.as_str().into());
                }
                
                if let Some(reply_to) = &props.reply_to {
                    lapin_props = lapin_props.with_reply_to(reply_to.as_str().into());
                }
                
                if let Some(expiration) = &props.expiration {
                    lapin_props = lapin_props.with_expiration(expiration.as_str().into());
                }
                
                if let Some(message_id) = &props.message_id {
                    lapin_props = lapin_props.with_message_id(message_id.as_str().into());
                }
                
                if let Some(timestamp) = props.timestamp {
                    lapin_props = lapin_props.with_timestamp(timestamp);
                }
                
                if let Some(user_id) = &props.user_id {
                    lapin_props = lapin_props.with_user_id(user_id.as_str().into());
                }
                
                if let Some(app_id) = &props.app_id {
                    lapin_props = lapin_props.with_app_id(app_id.as_str().into());
                }
                
                if let Some(delivery_mode) = props.delivery_mode {
                    lapin_props = lapin_props.with_delivery_mode(delivery_mode);
                }
                
                lapin_props
            },
            None => BasicProperties::default()
        };
        
        // Publish the message
        self.channel.basic_publish(
            &message.exchange,
            &message.routing_key,
            options,
            message.payload.as_bytes(),
            properties,
        )
        .await
        .map_err(|e| RabbitMQError::PublishError(e.to_string()))?;
        
        Ok(())
    }
    
    // Declare a queue
    pub async fn declare_queue(&self, queue_info: &QueueInfo) -> Result<(), RabbitMQError> {
        // Set up the queue options
        let options = QueueDeclareOptions {
            durable: queue_info.durable,
            exclusive: queue_info.exclusive,
            auto_delete: queue_info.auto_delete,
            ..QueueDeclareOptions::default()
        };
        
        // Empty arguments
        let arguments = FieldTable::default();
        
        // Declare the queue
        self.channel.queue_declare(
            &queue_info.name,
            options,
            arguments,
        )
        .await
        .map_err(|e| RabbitMQError::QueueError(e.to_string()))?;
        
        Ok(())
    }
    
    // Declare an exchange
    pub async fn declare_exchange(&self, exchange_info: &ExchangeInfo) -> Result<(), RabbitMQError> {
        // Set up the exchange options
        let options = ExchangeDeclareOptions {
            durable: exchange_info.durable,
            auto_delete: exchange_info.auto_delete,
            ..ExchangeDeclareOptions::default()
        };
        
        // Convert the exchange type string to lapin's ExchangeKind
        let kind = match exchange_info.kind.as_str() {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            "topic" => ExchangeKind::Topic,
            "headers" => ExchangeKind::Headers,
            _ => return Err(RabbitMQError::ExchangeError(format!("Invalid exchange type: {}", exchange_info.kind)))
        };
        
        // Empty arguments
        let arguments = FieldTable::default();
        
        // Declare the exchange
        self.channel.exchange_declare(
            &exchange_info.name,
            kind,
            options,
            arguments,
        )
        .await
        .map_err(|e| RabbitMQError::ExchangeError(e.to_string()))?;
        
        Ok(())
    }
}

