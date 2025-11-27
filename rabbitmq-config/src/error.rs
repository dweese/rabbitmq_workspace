use thiserror::Error;

#[derive(Error, Debug)]
pub enum RabbitMQError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Queue operation error: {0}")]
    QueueError(String),

    #[error("Exchange operation error: {0}")]
    ExchangeError(String),

    #[error("Binding operation error: {0}")]
    BindingError(String),

    #[error("Message publish error: {0}")]
    PublishError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlError(#[from] toml::de::Error),

    // KeyringError is being removed

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Lapin error: {0}")]
    LapinError(#[from] lapin::Error),
}
