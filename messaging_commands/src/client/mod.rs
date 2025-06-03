// was in messaging_commands/src/client.rs
// is now: messaging_commands/src/client/mod.rs

use crate::error::MessagingError;
use log::{debug, info};
// use tokio::net::TcpStream;
use lapin::{
    Connection, ConnectionProperties,
};
// use std::str::FromStr;

pub struct RabbitMQConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub vhost: String,
}

pub struct RabbitMQClient {
    connection: Connection,
}

impl RabbitMQClient {
    pub async fn new(config: RabbitMQConfig) -> Result<Self, MessagingError> {
        info!("Connecting to RabbitMQ at {}:{}", config.host, config.port);
        debug!("Using vhost: {}", config.vhost);

        // Build the connection URI
        let uri = format!(
            "amqp://{}:{}@{}:{}{}",
            percent_encoding::percent_encode(config.username.as_bytes(), percent_encoding::NON_ALPHANUMERIC),
            percent_encoding::percent_encode(config.password.as_bytes(), percent_encoding::NON_ALPHANUMERIC),
            config.host,
            config.port,
            config.vhost
        );

        // Create the connection
        let connection = match Connection::connect(
            &uri,
            ConnectionProperties::default()
                .with_executor(tokio_executor_trait::Tokio::current())
        ).await {
            Ok(conn) => conn,
            Err(err) => {
                return Err(MessagingError::ConnectionError(format!("Failed to connect: {}", err)));
            }
        };

        info!("Successfully connected to RabbitMQ");
        Ok(Self { connection })
    }

    pub async fn close(self) -> Result<(), MessagingError> {
        info!("Closing RabbitMQ connection");
        match self.connection.close(0, "Normal shutdown").await {
            Ok(_) => {
                info!("RabbitMQ connection closed successfully");
                Ok(())
            },
            Err(err) => {
                Err(MessagingError::ConnectionError(format!("Failed to close connection: {}", err)))
            }
        }
    }
}