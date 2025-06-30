use rabbitmq_config::RabbitMQConfig;
use crate::error::MessagingError;
use lapin::{Connection, ConnectionProperties, Channel};

/// RabbitMQ client for the messaging_commands crate
pub struct RabbitMQClient {
    connection: Connection,
    channel: Channel,
    config: RabbitMQConfig,
}

impl RabbitMQClient {
    /// Create a new RabbitMQ client with the given configuration
    pub async fn new(config: RabbitMQConfig) -> Result<Self, MessagingError> {
        // Build the AMQP URI
        let uri = format!(
            "amqp://{}:{}@{}:{}/{}",
            config.username,
            config.password,
            config.host,
            config.port,
            config.vhost
        );

        // Connect to RabbitMQ
        let connection = Connection::connect(&uri, ConnectionProperties::default()).await?;
        
        // Create a channel
        let channel = connection.create_channel().await?;

        Ok(RabbitMQClient {
            connection,
            channel,
            config,
        })
    }

    /// Get a reference to the channel
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Get a reference to the connection
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Get a reference to the config
    pub fn config(&self) -> &RabbitMQConfig {
        &self.config
    }

    /// Close the connection
    pub async fn close(self) -> Result<(), MessagingError> {
        self.connection.close(200, "Normal shutdown").await?;
        Ok(())
    }
}