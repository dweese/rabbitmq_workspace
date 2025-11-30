use clap::Parser;
use log::info;
use rabbitmq_config::{get_password, load_config_file, RabbitMQClient, RabbitMQConfig};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The queue to consume a message from.
    #[arg(short, long)]
    queue: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    // --- Connect to RabbitMQ ---
    let file_config = load_config_file()?;
    let conn_info = file_config.connection;

    info!("Connecting to RabbitMQ as user: '{}'", conn_info.username);
    let password = get_password()?;

    let config = RabbitMQConfig {
        host: conn_info.host,
        amqp_port: conn_info.amqp_port,
        management_port: conn_info.management_port,
        username: conn_info.username,
        password,
        vhost: conn_info.vhost,
    };

    let client = RabbitMQClient::new(config).await?;
    info!("Successfully connected to RabbitMQ.");

    // --- Consume Message ---
    info!("Attempting to consume one message from queue '{}'...", args.queue);
    match client.consume_one(&args.queue).await? {
        Some(payload) => {
            // Print the raw payload to stdout for verification
            println!("{}", payload);
            info!("Message consumed and acknowledged successfully.");
        }
        None => {
            info!("No message received from queue '{}'.", args.queue);
        }
    }

    client.close().await?;
    Ok(())
}
