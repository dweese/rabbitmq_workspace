//! Messaging Commands Library
//!
//! A comprehensive library for handling messaging operations with various protocols,
//! primarily focused on RabbitMQ connectivity and message handling.
//!
//! # Quick Start
//!
//! ```rust
//! # use messaging_commands::prelude::*;
//! # use rabbitmq_config::RabbitMQConfig;
//! #
//! #[tokio::main]
//! async fn main() -> Result<(), MessagingError> {
//!     // 1. Create a configuration
//!     let config = RabbitMQConfig::default(); // Assumes a local RabbitMQ instance
//!
//!     // 2. Create a new client for a specific protocol
//!     let mut client = AmqpClient::new(config);
//!
//!     // 3. Connect to the broker
//!     client.connect().await?;
//!
//!     // 4. Publish a message
//!     let payload = b"Hello, world!";
//!     client.publish("my_routing_key", payload).await?;
//!
//!     Ok(())
//! }
//! ```

// Core module declarations
pub mod clients;
pub mod error;
pub mod traits;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits.
/// Import everything with: `use messaging_commands::prelude::*;`
pub mod prelude {
    pub use crate::clients::amqp::AmqpClient;
    pub use crate::error::MessagingError;
    pub use crate::traits::MessagingClient;
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, error::MessagingError>;

// Tests module (integration tests should be in tests/ directory)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_imports_work() {
        // This is a compile-time check to ensure the prelude exports are accessible.
        // If this code compiles, the test passes.
        use prelude::*;
        let _a: Option<AmqpClient> = None;
        let _b: Option<Box<dyn MessagingClient>> = None;
    }
}