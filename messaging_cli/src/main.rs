use clap::{Parser, Subcommand};
use log::info;

#[derive(Parser)]
#[command(name = "messaging_cli")]
#[command(about = "A CLI tool for messaging operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a messaging server
    Connect {
        /// Protocol to use (amqp, mqtt, etc.)
        #[arg(long, default_value = "amqp")]
        protocol: String,

        /// Host to connect to
        #[arg(long, default_value = "localhost")]
        host: String,

        /// Port to connect on
        #[arg(long, default_value = "5672")]
        port: u16,

        /// Username for authentication
        #[arg(long)]
        username: Option<String>,

        /// Password for authentication
        #[arg(long)]
        password: Option<String>,
    },
    /// List queues, exchanges, etc.
    List {
        /// What to list (queues, exchanges, bindings)
        #[arg(value_enum)]
        resource: ListResource,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum ListResource {
    Queues,
    Exchanges,
    Bindings,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Connect {
            protocol,
            host,
            port,
            username,
            password: _password,
        } => {
            info!("Connecting to {protocol}://{host}:{port}");

            if let Some(user) = username {
                info!("Using username: {user}");
            }

            // TODO: Implement actual connection logic using your messaging_commands crate
            println!("Would connect to {protocol}://{host}:{port}");
        }
        Commands::List { resource } => {
            match resource {
                ListResource::Queues => {
                    info!("Listing queues");
                    // TODO: Use rabbitmq-info crate to list queues
                    println!("Would list queues");
                }
                ListResource::Exchanges => {
                    info!("Listing exchanges");
                    println!("Would list exchanges");
                }
                ListResource::Bindings => {
                    info!("Listing bindings");
                    println!("Would list bindings");
                }
            }
        }
    }
}
