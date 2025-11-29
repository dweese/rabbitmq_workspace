// rabbitmq-config/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RabbitMQError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

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

    #[error("Ack error: {0}")]
    AckError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlError(#[from] toml::de::Error),
}
