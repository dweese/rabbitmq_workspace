// rabbitmq-config/src/client.rs

use futures_util::stream::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
        ExchangeDeleteOptions, QueueBindOptions, QueueDeclareOptions, QueueDeleteOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use log::info;
use std::time::Duration;
use tokio::time::timeout;

use crate::{ExchangeInfo, QueueInfo, RabbitMQConfig, RabbitMQError, RabbitMQMessage};

/// Represents a connection to RabbitMQ
pub struct RabbitMQClient {
    connection: Option<Connection>,
    channel: Option<Channel>,
    config: RabbitMQConfig,
}

impl RabbitMQClient {
    /// Creates a new RabbitMQ client and establishes a connection
    pub async fn new(config: RabbitMQConfig) -> Result<Self, RabbitMQError> {
        info!(
            "component=RabbitMQClient action=new host={} port={}",
            config.host, config.amqp_port
        );

        let mut client = Self {
            connection: None,
            channel: None,
            config,
        };

        client.connect().await?;
        Ok(client)
    }

    async fn connect(&mut self) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=connect");
        let uri = self.config.to_uri();
        let connection_timeout = Duration::from_secs(10);
        let connection_future = Connection::connect(&uri, ConnectionProperties::default());

        let connection = timeout(connection_timeout, connection_future)
            .await
            .map_err(|_| RabbitMQError::TimeoutError("Connection timeout".to_string()))?
            .map_err(|e| RabbitMQError::ConnectionError(format!("{e}")))?;

        let channel = connection
            .create_channel()
            .await
            .map_err(|e| RabbitMQError::ChannelError(format!("{e}")))?;

        self.connection = Some(connection);
        self.channel = Some(channel);
        Ok(())
    }

    pub async fn close(&self) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=close");
        if let Some(connection) = &self.connection {
            connection
                .close(0, "Closing connection")
                .await
                .map_err(|e| RabbitMQError::ConnectionError(format!("Error closing connection: {e}")))?;
        }
        Ok(())
    }

    pub async fn declare_queue(&self, queue_info: &QueueInfo) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=declare_queue queue={}", queue_info.name);
        if let Some(channel) = &self.channel {
            let options = QueueDeclareOptions {
                durable: queue_info.durable,
                auto_delete: queue_info.auto_delete,
                exclusive: queue_info.exclusive,
                ..Default::default()
            };
            channel
                .queue_declare(&queue_info.name, options, FieldTable::default())
                .await
                .map_err(|e| RabbitMQError::QueueError(format!("Failed to declare queue: {e}")))?;
            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    pub async fn declare_exchange(&self, exchange_info: &ExchangeInfo) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=declare_exchange exchange={}", exchange_info.name);
        if let Some(channel) = &self.channel {
            let kind = match exchange_info.kind.as_str() {
                "direct" => ExchangeKind::Direct,
                "fanout" => ExchangeKind::Fanout,
                "topic" => ExchangeKind::Topic,
                "headers" => ExchangeKind::Headers,
                _ => return Err(RabbitMQError::ExchangeError(format!("Invalid exchange type: {}", exchange_info.kind))),
            };
            let options = ExchangeDeclareOptions {
                durable: exchange_info.durable,
                auto_delete: exchange_info.auto_delete,
                internal: exchange_info.internal,
                ..Default::default()
            };
            channel
                .exchange_declare(&exchange_info.name, kind, options, FieldTable::default())
                .await
                .map_err(|e| RabbitMQError::ExchangeError(format!("Failed to declare exchange: {e}")))?;
            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    pub async fn publish_message(&self, message: &RabbitMQMessage) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=publish_message exchange={} routing_key={}", message.exchange, message.routing_key);
        if let Some(channel) = &self.channel {
            let payload = &message.payload;
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
            channel
                .basic_publish(&message.exchange, &message.routing_key, BasicPublishOptions::default(), payload, properties)
                .await
                .map_err(|e| RabbitMQError::PublishError(format!("Failed to publish message: {e}")))?;
            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    pub async fn bind_queue(&self, queue_name: &str, exchange_name: &str, routing_key: &str) -> Result<(), RabbitMQError> {
        info!("component=RabbitMQClient action=bind_queue queue={queue_name} exchange={exchange_name} routing_key={routing_key}");
        if let Some(channel) = &self.channel {
            channel
                .queue_bind(queue_name, exchange_name, routing_key, QueueBindOptions::default(), FieldTable::default())
                .await
                .map_err(|e| RabbitMQError::BindingError(format!("Failed to bind queue: {e}")))?;
            Ok(())
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }

    /// Consumes a single message from a queue and acknowledges it.
    /// Returns the message payload as a String, or None if no message is received within a short timeout.
    pub async fn consume_one(&self, queue_name: &str) -> Result<Option<String>, RabbitMQError> {
        info!("component=RabbitMQClient action=consume_one queue={}", queue_name);
        if let Some(channel) = &self.channel {
            let mut consumer = channel
                .basic_consume(
                    queue_name,
                    "consume-one-consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| RabbitMQError::ConsumeError(format!("Failed to start consumer: {}", e)))?;

            // Use a timeout to avoid waiting forever if the queue is empty
            match timeout(Duration::from_secs(2), consumer.next()).await {
                Ok(Some(Ok(delivery))) => {
                    let payload = String::from_utf8(delivery.data.clone())
                        .map_err(|e| RabbitMQError::ConsumeError(format!("Invalid UTF-8 in payload: {}", e)))?;
                    
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .map_err(|e| RabbitMQError::AckError(format!("Failed to ack message: {}", e)))?;
                    
                    Ok(Some(payload))
                }
                Ok(Some(Err(e))) => Err(RabbitMQError::ConsumeError(format!("Error receiving message: {}", e))),
                Ok(None) | Err(_) => Ok(None), // Timeout or stream ended
            }
        } else {
            Err(RabbitMQError::ChannelError("No channel available".to_string()))
        }
    }
}
