// rabbitmq-config/src/lib.rs

// Module declarations
mod client;
mod config;
mod error;
mod models;
mod topology;

// Re-export the models needed by the UI
pub use client::RabbitMQClient;
pub use config::{ConnectionConfig, RabbitMQConfig, RabbitMQFullConfig};
pub use error::RabbitMQError;

// Re-export the models needed by the UI
pub use models::{
    BindingDefinition,
    ExchangeDefinition,
    ExchangeInfo,
    GlobalParameterDefinition,
    MessageProperties,
    PermissionDefinition,
    QueueDefinition,
    QueueInfo,
    RabbitMQMessage, // Export this struct
    RabbitMQServerDefinition,
    TopicPermissionDefinition,
    UserDefinition,
    VhostDefinition,
};
use std::fs;
use std::path::PathBuf;

/// Loads and parses the `rabbitmq-mon.toml` file to get non-sensitive connection info.
pub fn load_config_file() -> Result<RabbitMQFullConfig, RabbitMQError> {
    let config_dir = dirs::config_dir().ok_or_else(|| RabbitMQError::ConfigError("Could not determine config directory.".to_string()))?;
    let config_path: PathBuf = [config_dir.to_str().unwrap(), "rabbitmq-mon", "rabbitmq-mon.toml"].iter().collect();

    if config_path.exists() {
        log::info!("Loading configuration from: {:?}", config_path);
        let toml_str = fs::read_to_string(&config_path)?;
        toml::from_str(&toml_str).map_err(RabbitMQError::from)
    } else {
        log::warn!("Configuration file not found at: {:?}. Using defaults.", config_path);
        Ok(RabbitMQFullConfig::default())
    }
}
