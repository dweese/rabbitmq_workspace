use thiserror::Error;
use rabbitmq_config::RabbitMQError;

#[derive(Error, Debug)]
pub enum MessagingError {
    #[error("Configuration Error: {0}")]
    Config(#[from] RabbitMQError),

    #[error("AMQP protocol error: {0}")]
    Amqp(#[from] lapin::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Client is not connected")]
    NotConnected,
}