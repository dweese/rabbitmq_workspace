// messaging_commands/src/lib.rs

// Module declarations
pub mod client;
pub mod config;
pub mod error;
pub mod clients;
pub mod common;
pub mod protocol;

pub mod traits;
pub mod utils;
pub mod version;

// Re-exports of core types
pub use client::RabbitMQClient;
pub use client::RabbitMQConfig;
pub use error::MessagingError;


// Tests module (needs to point to tests/mod.rs)
#[cfg(test)]
pub mod tests;

// Example function
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// Unit tests directly in lib.rs
#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}