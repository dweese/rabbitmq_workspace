// rabbitmq-config/src/lib.rs

// Module declarations
mod client;
mod config;
mod error;
mod models;
mod topology;

// Re-export the models needed by the UI
pub use client::RabbitMQClient;
pub use config::RabbitMQConfig;
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
    QueueInfo, // Use QueueInfo from models instead of defining it here
    RabbitMQMessage,
    RabbitMQServerDefinition,
    TopicPermissionDefinition,
    UserDefinition,
    VhostDefinition,
};

// Remove the QueueInfo struct definition from here since we're now using the one from models
