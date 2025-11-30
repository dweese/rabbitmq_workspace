use rabbitmq_config::{RabbitMQClient, RabbitMQConfig};
use tokio::runtime::Runtime;

#[test]
fn test_rabbitmq_connection() {
    // Create a config for connecting to a local RabbitMQ
    let config = RabbitMQConfig {
        host: "localhost".to_string(),
        amqp_port: 5672,
        management_port: 15672,
        username: "guest".to_string(),
        password: "guest".to_string(),
        vhost: "/".to_string(),
    };

    // Create a tokio runtime for async code
    let rt = Runtime::new().unwrap();

    // Try to connect
    let client_result = rt.block_on(async { RabbitMQClient::new(config).await });

    // Assert that we can connect successfully
    assert!(
        client_result.is_ok(),
        "Failed to connect to RabbitMQ: {:?}",
        client_result.err()
    );

    // If connection was successful, close it
    if let Ok(client) = client_result {
        let close_result = rt.block_on(async { client.close().await });
        assert!(
            close_result.is_ok(),
            "Failed to close RabbitMQ connection: {:?}",
            close_result.err()
        );
    }
}
