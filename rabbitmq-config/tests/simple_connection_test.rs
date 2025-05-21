use rabbitmq_config::{RabbitMQClient, RabbitMQConfig};
use tokio::runtime::Runtime;

#[test]
fn test_simple_rabbitmq_connection() {
    // Create a simple config
    let config = RabbitMQConfig {
        host: "localhost".to_string(),
        port: 5672,
        username: "guest".to_string(),
        password: "guest".to_string(),
        vhost: "/".to_string(),
    };

    // Create a tokio runtime
    let rt = Runtime::new().unwrap();

    // Try to connect
    let result = rt.block_on(async {
        let client = RabbitMQClient::new(config).await?;

        // If we get here, connection was successful
        // Close it properly
        client.close().await?;

        Ok::<_, rabbitmq_config::RabbitMQError>(())
    });

    assert!(result.is_ok(), "Failed to connect to RabbitMQ: {:?}", result.err());
}