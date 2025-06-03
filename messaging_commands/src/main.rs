
// src/main.rs

use clap::{Parser, Subcommand};
use log::{debug, info};

// Use crate:: to reference modules within your own crate
// Import from the library crate
use messaging_commands::client::RabbitMQClient;
use messaging_commands::client::RabbitMQConfig;
use messaging_commands::error::MessagingError;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Connect {
        #[arg(short, long)]
        protocol: String,

        #[arg(short, long, default_value = "localhost")]
        host: String,

        #[arg(short, long)]
        port: u16,

        #[arg(short, long)]
        username: Option<String>,

        #[arg(short, long)]
        password: Option<String>,
    },
    // Add other commands as needed
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logger
    env_logger::Builder::new()
        .filter_level(if cli.verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .init();

    // CLI command handling logic
    match cli.command {
        Commands::Connect { protocol, host, port, username, password } => {
            info!("Connecting to {}://{}:{}", protocol, host, port);
            debug!("Username: {:?}, Password: {}", 
                   username, 
                   if password.is_some() { "[REDACTED]" } else { "None" });

            // Only handle AMQP protocol for now
            if protocol.to_lowercase() != "amqp" {
                eprintln!("Error: Only AMQP protocol is supported currently");
                return Ok(());
            }

            // Use tokio runtime for async operations
            let rt = tokio::runtime::Runtime::new()?;

            // Create RabbitMQ config
            let config = RabbitMQConfig {
                host,
                port,
                username: username.unwrap_or_else(|| "guest".to_string()),
                password: password.unwrap_or_else(|| "guest".to_string()),
                vhost: "/".to_string(),
            };

            // Try to connect
            let result = rt.block_on(async {
                let client = RabbitMQClient::new(config).await?;
                println!("Successfully connected to RabbitMQ!");

                // Close the connection
                client.close().await?;
                println!("Connection closed");

                Ok::<_, MessagingError>(())
            });

            if let Err(err) = result {
                eprintln!("Error: {}", err);
            }
        },
        // Other command handlers
    }

    Ok(())
}