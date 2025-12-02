use chrono::Local;
use rabbitmq_config::{get_password, load_config_file, RabbitMQConfig};
use rabbitmq_info::api::RabbitMQApiClient;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // --- Connect to RabbitMQ ---
    let file_config = load_config_file()?;
    let conn_info = file_config.connection;

    println!("Connecting to RabbitMQ Management API as user: '{}'", conn_info.username);
    let password = get_password()?;

    let config = RabbitMQConfig {
        host: conn_info.host,
        amqp_port: conn_info.amqp_port,
        management_port: conn_info.management_port,
        username: conn_info.username,
        password,
        vhost: conn_info.vhost,
    };

    let client = RabbitMQApiClient::new(&config)?;

    // --- Fetch Definitions ---
    println!("Fetching server definitions...");
    let definitions: Value = client.get_definitions().await?;
    println!("Successfully fetched definitions.");

    // --- Save to File ---
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("server_state_{}.json", timestamp);
    
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../artifacts");
    fs::create_dir_all(&path)?; // Ensure the artifacts directory exists
    path.push(filename);

    println!("Saving definitions to: {:?}", path);
    let pretty_json = serde_json::to_string_pretty(&definitions)?;
    fs::write(&path, pretty_json)?;

    println!("Successfully saved server state snapshot.");
    Ok(())
}
