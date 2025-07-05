//! Messaging Commands Library
//! 
//! A comprehensive library for handling messaging operations with various protocols,
//! primarily focused on RabbitMQ connectivity and message handling.
//! 
//! # Quick Start
//! 
//! ```rust
//! use messaging_commands::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), MessagingError> {
//!     let config = RabbitMQConfig::default();
//!     let client = RabbitMQClient::new(config).await?;
//!     // Use the client...
//!     Ok(())
//! }
//! ```

// Core module declarations
pub mod client;
pub mod clients;
pub mod common;
pub mod config;
pub mod protocol;
// Remove this line to avoid conflict with the #[cfg(test)] module below
// pub mod tests;
pub mod traits;
pub mod utils;
pub mod version;
pub mod error;

// Re-export commonly used items
pub use client::RabbitMQClient;
pub use error::MessagingError;

// Re-exports of core types for easy access
pub use rabbitmq_config::RabbitMQConfig;

/// Prelude module for convenient imports
/// 
/// This module re-exports the most commonly used types and traits.
/// Import everything with: `use messaging_commands::prelude::*;`
pub mod prelude {
    pub use crate::{RabbitMQClient, MessagingError};
    pub use rabbitmq_config::RabbitMQConfig;
    // Note: Commented out modules that are currently empty
    // pub use crate::traits::*;
    // pub use crate::common::*;
    // pub use crate::protocol::*;
    // pub use crate::utils::*;
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, MessagingError>;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Tests module (integration tests should be in tests/ directory)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn prelude_imports_work() {
        use crate::prelude::*;
        // Test that main types are available
        let _version = VERSION;
    }
}