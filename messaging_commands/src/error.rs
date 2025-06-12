// messaging_commands/src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Lapin error: {0}")]
    LapinError(#[from] lapin::Error),
}