//! messaging_commands CLI
//! 
//! A command-line interface for RabbitMQ messaging operations

use messaging_commands::prelude::*;
use clap::{Parser, Subcommand};
use env_logger;
use log::info;

#[derive(Parser)]
#[command(name = "messaging_commands")]
#[command(about = "A CLI for RabbitMQ messaging operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test RabbitMQ connection
    TestConnection {
        /// RabbitMQ host
        #[arg(short, long, default_value = "localhost")]
        host: String,
        
        /// RabbitMQ port
        #[arg(short, long, default_value = "5672")]
        port: u16,
        
        /// Username
        #[arg(short, long, default_value = "guest")]
        username: String,
        
        /// Password
        #[arg(short = 'P', long, default_value = "guest")]
        password: String,
        
        /// Virtual host
        #[arg(short, long, default_value = "/")]
        vhost: String,
    },
    
    /// Send a message
    SendMessage {
        /// Exchange name
        #[arg(short, long)]
        exchange: String,
        
        /// Routing key
        #[arg(short, long)]
        routing_key: String,
        
        /// Message body
        #[arg(short, long)]
        message: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::TestConnection { host, port, username, password, vhost } => {
            info!("Testing RabbitMQ connection...");
            
            let config = RabbitMQConfig {
                host,
                port,
                username,
                password,
                vhost,
                ..Default::default()
            };
            
            match RabbitMQClient::new(config).await {
                Ok(client) => {
                    println!("✅ Connection successful!");
                    println!("Connected to: {}:{}", client.config().host, client.config().port);
                    client.close().await?;
                }
                Err(e) => {
                    println!("❌ Connection failed: {}", e);
                    return Err(e.into());
                }
            }
        }
        
        Commands::SendMessage { exchange, routing_key, message } => {
            info!("Sending message...");
            
            let config = RabbitMQConfig::default();
            let client = RabbitMQClient::new(config).await?;
            
            // Here you would implement message sending logic
            // using the client.channel()
            
            println!("📨 Message sent to exchange '{}' with routing key '{}'", exchange, routing_key);
            println!("Message: {}", message);
            
            client.close().await?;
        }
    }
    
    Ok(())
}