// rabbitmq-config/src/lib.rs

mod config;
mod client;

// Re-export everything from the modules
pub use config::*;
pub use client::*;
