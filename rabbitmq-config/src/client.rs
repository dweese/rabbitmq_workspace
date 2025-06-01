// rabbitmq-config/src/client.rs

use lapin::{
    Connection, Channel, ConnectionProperties,
    options::{
        QueueDeclareOptions, ExchangeDeclareOptions, BasicPublishOptions,
        QueueBindOptions, QueueDeleteOptions, ExchangeDeleteOptions
    },
    types::FieldTable,
    BasicProperties,
    ExchangeKind
};
use log::{info};
use std::time::Duration;
use tokio::time::timeout;

use crate::{
    RabbitMQConfig,
    RabbitMQError,
    QueueInfo,
    ExchangeInfo,
    RabbitMQMessage,

};

/// Represents a connection to RabbitMQ
pub struct RabbitMQClient {
    connection: Option<Connection>,
    channel: Option<Channel>,
    config: RabbitMQConfig,
}

impl RabbitMQClient {
    /// Creates a new RabbitMQ client and establishes a connection
    pub async fn new(config: RabbitMQConfig) -> Result<Self, RabbitMQError> {
        info!("component=RabbitMQClient action=new host={} port={}", config.host, config.port);

        let mut client = Self {
            connection: None,
            channel: None,
            config,
        };

        // Connect to RabbitMQ server
        client.connect().await?;

        Ok(client)
    }

    /// Lists all queues in the vhost - UI friendly version
    pub async fn list_queues(&self) -> Result<Vec<String>, RabbitMQError> {
        // Get the detailed queue objects
        let queues = self.get_queues().await?;

        // Return just the queue names as strings
        Ok(queues.into_iter()
            .map(|q| q.name)
            .collect())
    }

    /// Establishes a connection to RabbitMQ
    async fn connect(&mut self) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=connect");

        // Construct connection URI
        let uri = format!(
            "amqp://{}:{}@{}:{}/{}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.port,
            self.config.vhost
        );

        // Connect with timeout
        let connection_timeout = Duration::from_secs(10);
        let connection_future = Connection::connect(
            &uri,
            ConnectionProperties::default()
        );

        let connection = timeout(connection_timeout, connection_future)
            .await
            .map_err(|_| RabbitMQError::TimeoutError("Connection timeout".to_string()))?
            .map_err(|e| RabbitMQError::ConnectionError(format!("{}", e)))?;

        // Create a channel
        let channel = connection.create_channel()
            .await
            .map_err(|e| RabbitMQError::ChannelError(format!("{}", e)))?;

        self.connection = Some(connection);
        self.channel = Some(channel);

        Ok(())
    }

    /// Closes the connection to RabbitMQ
    pub async fn close(&self) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=close");

        if let Some(connection) = &self.connection {
            connection.close(0, "Closing connection")
                .await
                .map_err(|e| RabbitMQError::ConnectionError(format!("Error closing connection: {}", e)))?;
        }

        Ok(())
    }

    /// Gets information about all queues
    pub async fn get_queues(&self) -> Result<Vec<Queue>, RabbitMQError> {
        info!("component=RabbitMQClient action=get_queues");

        // For a real implementation, you would use the RabbitMQ HTTP API
        // This is a simplified example
        Ok(vec![]) // Placeholder - implement real API call
    }

    /// Lists all exchanges in the vhost
    pub async fn list_exchanges(&self) -> Result<Vec<String>, RabbitMQError> {
        info!("component=RabbitMQClient action=list_exchanges");

        // For a real implementation, you would use the RabbitMQ HTTP API
        // This is a simplified example
        Ok(vec![
            "amq.direct".to_string(),
            "amq.fanout".to_string(),
            "amq.topic".to_string(),
            "amq.headers".to_string(),
        ]) // Placeholder - implement real API call
    }

    /// Declares a new queue with the given parameters
    pub async fn declare_queue(&self, queue_info: &QueueInfo) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=declare_queue queue={}", queue_info.name);

        if let Some(channel) = &self.channel {
            // Create queue options based on QueueInfo
            let options = QueueDeclareOptions {
                durable: queue_info.durable,
                auto_delete: queue_info.auto_delete,
                exclusive: queue_info.exclusive,
                ..Default::default()
            };

            // Declare the queue
            channel.queue_declare(&queue_info.name, options, FieldTable::default())
                .await
                .map_err(|e| RabbitMQError::QueueError(format!("Failed to declare queue: {}", e)))?;

            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    /// Declares a new exchange with the given parameters
    pub async fn declare_exchange(&self, exchange_info: &ExchangeInfo) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=declare_exchange exchange={}", exchange_info.name);

        if let Some(channel) = &self.channel {
            // Map the exchange kind string to the ExchangeKind enum
            let kind = match exchange_info.kind.as_str() {
                "direct" => ExchangeKind::Direct,
                "fanout" => ExchangeKind::Fanout,
                "topic" => ExchangeKind::Topic,
                "headers" => ExchangeKind::Headers,
                _ => return Err(RabbitMQError::ExchangeError(
                    format!("Invalid exchange type: {}", exchange_info.kind)
                )),
            };

            // Create exchange options
            let options = ExchangeDeclareOptions {
                durable: exchange_info.durable,
                auto_delete: exchange_info.auto_delete,
                ..Default::default()
            };

            // Declare the exchange
            channel.exchange_declare(&exchange_info.name, kind, options, FieldTable::default())
                .await
                .map_err(|e| RabbitMQError::ExchangeError(format!("Failed to declare exchange: {}", e)))?;

            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    /// Publishes a message to RabbitMQ
    pub async fn publish_message(&self, message: &RabbitMQMessage) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=publish_message exchange={} routing_key={}", 
              message.exchange, message.routing_key);

        if let Some(channel) = &self.channel {
            // Convert payload to bytes
            let payload = &message.payload;
            
            // Set up properties
            let mut properties = BasicProperties::default();
            
            if let Some(props) = &message.properties {
                if let Some(content_type) = &props.content_type {
                    properties = properties.with_content_type(content_type.as_str().into());
                }

                if let Some(content_encoding) = &props.content_encoding {
                    properties = properties.with_content_encoding(content_encoding.as_str().into());
                }

                if let Some(delivery_mode) = props.delivery_mode {
                    properties = properties.with_delivery_mode(delivery_mode);
                }
            }

            // Publish the message
            channel.basic_publish(
                &message.exchange,
                &message.routing_key,
                BasicPublishOptions::default(),
                &payload,
                properties
            )
                .await
                .map_err(|e| RabbitMQError::PublishError(format!("Failed to publish message: {}", e)))?;

            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    /// Binds a queue to an exchange
    pub async fn bind_queue(&self, queue_name: &str, exchange_name: &str, routing_key: &str) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=bind_queue queue={} exchange={} routing_key={}", 
              queue_name, exchange_name, routing_key);

        if let Some(channel) = &self.channel {
            channel.queue_bind(
                queue_name,
                exchange_name,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default()
            )
                .await
                .map_err(|e| RabbitMQError::BindingError(format!("Failed to bind queue: {}", e)))?;

            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    /// Deletes a queue
    pub async fn delete_queue(&self, queue_name: &str) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=delete_queue queue={}", queue_name);

        if let Some(channel) = &self.channel {
            channel.queue_delete(
                queue_name,
                QueueDeleteOptions::default()
            )
                .await
                .map_err(|e| RabbitMQError::QueueError(format!("Failed to delete queue: {}", e)))?;

            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    /// Deletes an exchange
    pub async fn delete_exchange(&self, exchange_name: &str) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=delete_exchange exchange={}", exchange_name);

        if let Some(channel) = &self.channel {
            channel.exchange_delete(
                exchange_name,
                ExchangeDeleteOptions::default()
            )
                .await
                .map_err(|e| RabbitMQError::ExchangeError(format!("Failed to delete exchange: {}", e)))?;

            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }
}

// Define a Queue struct for the get_queues method
#[derive(Debug, Clone)]
pub struct Queue {
    pub name: String,
    pub messages: u32,
    pub consumers: u32,
}


