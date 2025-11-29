
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
//!     client.publish("my_exchange", "my_routing_key", payload).await?;
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

// CLI-specific functionality (only available with cli feature)
#[cfg(feature = "cli")]
pub mod cli {
    pub use clap::{Parser, Subcommand};
    pub use env_logger::{Builder, Target};
    pub use log::{error, info, warn};
    pub use std::fs::OpenOptions;
    pub use std::io::Write;

    #[derive(Parser)]
    #[command(author, version, about, long_about = None)]
    pub struct Cli {
        /// Enable verbose output
        #[arg(short, long)]
        pub verbose: bool,

        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Subcommand)]
    pub enum Commands {
        Connect {
            #[arg(short = 'P', long)]  // Capital P for Protocol
            protocol: String,

            #[arg(short = 'H', long, default_value = "localhost")]  // Capital H for Host (since -h is reserved for help)
            host: String,

            #[arg(short, long)]  // -p for port
            port: u16,

            #[arg(short, long)]  // -u for username
            username: Option<String>,

            #[arg(short = 'w', long)]  // -w for password
            password: Option<String>,
        },
        Reload {
            #[arg(short, long, default_value = "artifacts/rabbitmq_config.json")]
            config_file: String,
        },
    }

    pub fn setup_file_logging() -> Result<(), Box<dyn std::error::Error>> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("rabbitmq_workspace.log")?;

        Builder::from_default_env()
            .target(Target::Pipe(Box::new(log_file)))
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] [{}:{}] - {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();

        Ok(())
    }

    pub fn reload_config(config_file: &str) {
        info!("=== Starting configuration reload ===");
        info!("Reading configuration from: {}", config_file);

        match std::fs::read_to_string(config_file) {
            Ok(content) => {
                info!("File read successfully, size: {} bytes", content.len());

                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        info!("✅ JSON parsed successfully");

                        if let Some(obj) = json.as_object() {
                            info!("Configuration contains {} top-level keys:", obj.len());
                            for key in obj.keys() {
                                info!("  - {}", key);
                            }

                            // Log some specific interesting sections
                            if let Some(exchanges) = obj.get("exchanges") {
                                if let Some(arr) = exchanges.as_array() {
                                    info!("Found {} exchanges in configuration", arr.len());
                                }
                            }

                            if let Some(queues) = obj.get("queues") {
                                if let Some(arr) = queues.as_array() {
                                    info!("Found {} queues in configuration", arr.len());
                                }
                            }

                            if let Some(bindings) = obj.get("bindings") {
                                if let Some(arr) = bindings.as_array() {
                                    info!("Found {} bindings in configuration", arr.len());
                                }
                            }

                            if let Some(users) = obj.get("users") {
                                if let Some(arr) = users.as_array() {
                                    info!("Found {} users in configuration", arr.len());
                                }
                            }
                        } else {
                            warn!("JSON root is not an object");
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to parse JSON: {}", e);
                        error!("Check the JSON syntax in: {}", config_file);
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to read configuration file: {}", e);
                error!("Make sure the file exists: {}", config_file);
            }
        }

        info!("=== Configuration reload finished ===");
    }
}

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

    #[cfg(feature = "cli")]
    mod cli_tests {
        use super::*;
        use std::io::Write;
        use tempfile::NamedTempFile;

        #[test]
        fn test_cli_parsing_connect_command() {
            use clap::Parser;

            // Test basic connect command with updated short options
            let args = vec![
                "messaging_commands",
                "connect",
                "-P", "amqp",         // -P for protocol
                "-H", "localhost",    // -H for host
                "-p", "5672",         // -p for port
                "-u", "guest",        // -u for username
                "-w", "guest"         // -w for password
            ];

            let cli = cli::Cli::try_parse_from(args).unwrap();

            match cli.command {
                cli::Commands::Connect { protocol, host, port, username, password } => {
                    assert_eq!(protocol, "amqp");
                    assert_eq!(host, "localhost");
                    assert_eq!(port, 5672);
                    assert_eq!(username, Some("guest".to_string()));
                    assert_eq!(password, Some("guest".to_string()));
                }
                _ => panic!("Expected Connect command"),
            }
        }

        #[test]
        fn test_cli_parsing_connect_with_defaults() {
            use clap::Parser;

            // Test connect command with default host
            let args = vec![
                "messaging_commands",
                "connect",
                "--protocol", "amqp",
                "--port", "5672"
            ];

            let cli = cli::Cli::try_parse_from(args).unwrap();

            match cli.command {
                cli::Commands::Connect { protocol, host, port, username, password } => {
                    assert_eq!(protocol, "amqp");
                    assert_eq!(host, "localhost"); // Default value
                    assert_eq!(port, 5672);
                    assert_eq!(username, None);
                    assert_eq!(password, None);
                }
                _ => panic!("Expected Connect command"),
            }
        }

        #[test]
        fn test_cli_parsing_reload_command() {
            use clap::Parser;

            // Test reload command with default config file
            let args = vec![
                "messaging_commands",
                "reload"
            ];

            let cli = cli::Cli::try_parse_from(args).unwrap();

            match cli.command {
                cli::Commands::Reload { config_file } => {
                    assert_eq!(config_file, "artifacts/rabbitmq_config.json");
                }
                _ => panic!("Expected Reload command"),
            }
        }

        #[test]
        fn test_cli_parsing_reload_custom_config() {
            use clap::Parser;

            // Test reload command with custom config file
            let args = vec![
                "messaging_commands",
                "reload",
                "--config-file", "/custom/path/config.json"
            ];

            let cli = cli::Cli::try_parse_from(args).unwrap();

            match cli.command {
                cli::Commands::Reload { config_file } => {
                    assert_eq!(config_file, "/custom/path/config.json");
                }
                _ => panic!("Expected Reload command"),
            }
        }

        #[test]
        fn test_verbose_flag() {
            use clap::Parser;

            let args = vec![
                "messaging_commands",
                "--verbose",
                "connect",
                "--protocol", "amqp",
                "--port", "5672"
            ];

            let cli = cli::Cli::try_parse_from(args).unwrap();
            assert!(cli.verbose);
        }

        #[test]
        fn test_config_reload_with_valid_json() {
            // Create a temporary file with valid JSON
            let mut temp_file = NamedTempFile::new().unwrap();
            let config_content = r#"{
                "exchanges": [
                    {"name": "test_exchange", "type": "direct"}
                ],
                "queues": [
                    {"name": "test_queue", "durable": true}
                ],
                "bindings": [
                    {"source": "test_exchange", "destination": "test_queue", "routing_key": "test"}
                ],
                "users": [
                    {"name": "test_user", "password": "test_pass"}
                ]
            }"#;

            temp_file.write_all(config_content.as_bytes()).unwrap();
            temp_file.flush().unwrap();

            // This test would need to capture log output to verify proper parsing
            // For now, we just ensure it doesn't panic
            cli::reload_config(temp_file.path().to_str().unwrap());
        }

        #[test]
        fn test_config_reload_with_invalid_json() {
            // Create a temporary file with invalid JSON
            let mut temp_file = NamedTempFile::new().unwrap();
            let invalid_content = r#"{ "invalid": json content }"#;

            temp_file.write_all(invalid_content.as_bytes()).unwrap();
            temp_file.flush().unwrap();

            // This should handle the error gracefully (logged, not panicked)
            cli::reload_config(temp_file.path().to_str().unwrap());
        }

        #[test]
        fn test_config_reload_with_nonexistent_file() {
            // Test with a file that doesn't exist
            cli::reload_config("/nonexistent/path/config.json");
            // Should handle the error gracefully (logged, not panicked)
        }

        #[test]
        fn test_cli_missing_required_args() {
            use clap::Parser;

            // Test that missing required arguments fail parsing
            let args = vec![
                "messaging_commands",
                "connect",
                "--protocol", "amqp"
                // Missing required --port argument
            ];

            let result = cli::Cli::try_parse_from(args);
            assert!(result.is_err());
        }

        #[test]
        fn test_cli_invalid_port() {
            use clap::Parser;

            // Test that invalid port values fail parsing
            let args = vec![
                "messaging_commands",
                "connect",
                "--protocol", "amqp",
                "--port", "invalid_port"
            ];

            let result = cli::Cli::try_parse_from(args);
            assert!(result.is_err());
        }
    }
}