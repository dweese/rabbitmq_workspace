// rabbitmq-config/src/lib.rs



// Module declarations
mod client;
mod config;
mod error;
mod topology;
mod models;

// Re-export the models needed by the UI
pub use client::RabbitMQClient;
pub use config::RabbitMQConfig;
pub use error::RabbitMQError;

// Re-export the models needed by the UI
pub use models::{
    MessageProperties,
    RabbitMQMessage,
    ExchangeInfo,
    QueueInfo,  // Use QueueInfo from models instead of defining it here
    RabbitMQServerDefinition,
    UserDefinition,
    VhostDefinition,
    PermissionDefinition,
    TopicPermissionDefinition,
    GlobalParameterDefinition,
    QueueDefinition,
    ExchangeDefinition,
    BindingDefinition,
};


// Remove the QueueInfo struct definition from here since we're now using the one from models