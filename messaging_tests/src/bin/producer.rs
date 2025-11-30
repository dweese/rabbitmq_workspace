use clap::Parser;
use log::info;
use rabbitmq_config::{get_password, load_config_file, RabbitMQClient, RabbitMQConfig, RabbitMQMessage};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The exchange to publish the message to.
    #[arg(short, long)]
    exchange: String,

    /// The routing key for the message.
    #[arg(short, long)]
    routing_key: String,

    /// The message payload, as a JSON string.
    #[arg(short, long)]
    payload: String,
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

    // --- Publish Message ---
    let message = RabbitMQMessage {
        exchange: args.exchange,
        routing_key: args.routing_key,
        payload: args.payload.into_bytes(),
        properties: None, // For now, we don't need special properties
    };

    info!("Publishing message...");
    client.publish_message(&message).await?;
    info!("Message published successfully.");

    client.close().await?;
    Ok(())
}
