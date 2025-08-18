// messaging_commands/src/main.rs

use clap::{Parser, Subcommand};
use env_logger::{Builder, Target};
use log::{error, info, warn};
use std::fs::OpenOptions;
use std::io::Write;

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
    Reload {
        #[arg(short, long, default_value = "artifacts/rabbitmq_config.json")]
        config_file: String,
    },
}

fn setup_file_logging() -> Result<(), Box<dyn std::error::Error>> {
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

fn reload_config(config_file: &str) {
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

fn main() {
    // Setup file logging
    if let Err(e) = setup_file_logging() {
        eprintln!("Failed to setup logging: {}", e);
        env_logger::init(); // Fallback to console logging
    }

    let cli = Cli::parse();

    info!("Starting messaging_commands application");
    info!("Verbose mode: {}", cli.verbose);

    match &cli.command {
        Commands::Connect {
            protocol,
            host,
            port,
            username,
            password,
        } => {
            info!("Connect command received");
            info!("Protocol: {}", protocol);
            info!("Host: {}", host);
            info!("Port: {}", port);
            info!("Username: {:?}", username);
            info!(
                "Password: {}",
                if password.is_some() {
                    "[PROVIDED]"
                } else {
                    "[NOT PROVIDED]"
                }
            );

            // TODO: Implement actual connection logic here
            info!("Connection logic would be implemented here");
        }
        Commands::Reload { config_file } => {
            reload_config(&config_file);
        }
    }

    info!("Application finished");
}
