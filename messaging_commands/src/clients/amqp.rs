use async_trait::async_trait;
use lapin::{options::*, BasicProperties, Channel, Connection, ConnectionProperties};
use log::{debug, info};
use rabbitmq_config::RabbitMQConfig;

use crate::error::MessagingError;
use crate::traits::MessagingClient;

pub struct AmqpClient {
    config: RabbitMQConfig,
    connection: Option<Connection>,
    channel: Option<Channel>,
}


impl AmqpClient {
    /// Creates a new `AmqpClient` with the given configuration.
    pub fn new(config: RabbitMQConfig) -> Self {
        Self {
            config,
            connection: None,
            channel: None,
        }
    }
}

#[async_trait]
impl MessagingClient for AmqpClient {
    async fn connect(&mut self) -> Result<(), MessagingError> {
        // The `to_uri` method is defined in the `rabbitmq-config` crate.
        let addr = self.config.to_uri();
        info!("Connecting to AMQP broker at {}", addr);
        let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;

        info!("Successfully connected to AMQP broker and opened a channel.");
        self.connection = Some(conn);
        self.channel = Some(channel);
        Ok(())
    }

    async fn publish(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &[u8],
    ) -> Result<(), MessagingError> {
        debug!(
            "Publishing message to exchange '{}' with routing key '{}'",
            exchange, routing_key
        );
        let channel = self.channel.as_ref().ok_or(MessagingError::NotConnected)?;

        channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await?;

        debug!("Message published successfully.");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), MessagingError> {
        if let Some(conn) = self.connection.take() {
            info!("Closing AMQP connection.");
            // The `close` method on a lapin::Connection gracefully shuts down the connection.
            // The 200 code is a standard success code.
            conn.close(200, "Goodbye").await?;
            self.channel = None; // Also clear the channel
        }
        Ok(())
    }
}