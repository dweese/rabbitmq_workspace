//! Defines the common traits for all messaging clients.

use crate::error::MessagingError;
use async_trait::async_trait;

/// A trait representing a client that can connect to a messaging broker,
/// publish messages, and potentially consume them.
///
/// By requiring `Send + Sync`, we ensure that any type implementing this trait
/// can be safely used across threads, which is crucial for concurrent applications.
#[async_trait]
pub trait MessagingClient: Send + Sync {
    /// Connect to the broker using the provided configuration.
    async fn connect(&mut self) -> Result<(), MessagingError>;

    /// Publish a message to a specific exchange with a routing key.
    async fn publish(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &[u8],
    ) -> Result<(), MessagingError>;

    /// Disconnect from the broker and release resources.
    async fn disconnect(&mut self) -> Result<(), MessagingError>;
}