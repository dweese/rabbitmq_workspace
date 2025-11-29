use rabbitmq_config::{load_config_file, RabbitMQClient, RabbitMQConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct MessageTypes {
    message_types: Vec<Category>,
}

#[derive(Debug, Deserialize)]
struct Category {
    category: String,
    types: Vec<MessageType>,
}

#[derive(Debug, Deserialize)]
struct MessageType {
    name: String,
    priority: u8,
    durable: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // --- Connect to RabbitMQ ---
    let file_config = load_config_file()?;
    let conn_info = file_config.connection;

    println!("Connecting to RabbitMQ as user: '{}'", conn_info.username);
    let password = rpassword::prompt_password("Enter password: ")?;

    let config = RabbitMQConfig {
        host: conn_info.host,
        amqp_port: conn_info.amqp_port,
        management_port: conn_info.management_port,
        username: conn_info.username,
        password,
        vhost: conn_info.vhost,
    };

    let client = RabbitMQClient::new(config).await?;
    println!("Successfully connected to RabbitMQ.");

    // --- Load Topology Definition ---
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Corrected path: Go up one level from rabbitmq-config/ to the workspace root
    path.push("../artifacts/message_types.json"); 

    println!("Loading topology from: {:?}", path);
    let topology_str = fs::read_to_string(path)?;
    let topology: MessageTypes = serde_json::from_str(&topology_str)?;

    // --- Declare Topology ---
    for category in topology.message_types {
        let exchange_name = category.category.clone();
        println!("Declaring exchange: {}", exchange_name);

        // For now, we'll assume all are topic exchanges, which is a common pattern.
        let exchange_info = rabbitmq_config::ExchangeInfo {
            name: exchange_name.clone(),
            kind: "topic".to_string(),
            durable: true, // Exchanges are almost always durable
            auto_delete: false,
            internal: false,
            arguments: HashMap::new(),
        };
        client.declare_exchange(&exchange_info).await?;

        for msg_type in category.types {
            let queue_name = msg_type.name.clone();
            println!("- Declaring queue: {}", queue_name);

            let queue_info = rabbitmq_config::QueueInfo {
                name: queue_name.clone(),
                durable: msg_type.durable,
                exclusive: false,
                auto_delete: false,
                arguments: HashMap::new(),
            };
            client.declare_queue(&queue_info).await?;

            // Bind the queue to the exchange with its name as the routing key
            println!("  - Binding queue {} to exchange {} with routing key {}", queue_name, exchange_name, queue_name);
            client.bind_queue(&queue_name, &exchange_name, &queue_name).await?;
        }
    }

    println!("Topology creation complete.");
    client.close().await?;
    Ok(())
}
