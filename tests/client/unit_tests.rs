// rabbitmq-config/tests/client/unit_tests.rs

use rabbitmq_config::{
    client::{
        RabbitMQClient, RabbitMQMessage, MessageProperties,
        QueueInfo, ExchangeInfo, RabbitMQError
    },
    config::RabbitMQConfig,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

// Test helper function to create a mock config
fn create_test_config() -> RabbitMQConfig {
    RabbitMQConfig {
        host: "localhost".to_string(),
        port: 5672,
        username: "guest".to_string(),
        password: "guest".to_string(),
        vhost: "/".to_string(),
    }
}

// Test that MessageProperties::default() creates expected values
#[test]
fn test_message_properties_default() {
    let props = MessageProperties::default();

    assert_eq!(props.content_type, Some("application/json".to_string()));
    assert_eq!(props.content_encoding, None);
    assert_eq!(props.correlation_id, None);
    assert_eq!(props.reply_to, None);
    assert_eq!(props.expiration, None);
    assert_eq!(props.message_id, None);
    assert_eq!(props.timestamp, None);
    assert_eq!(props.user_id, None);
    assert_eq!(props.app_id, Some("old-rabbitmq-ui".to_string()));
    assert_eq!(props.delivery_mode, Some(2)); // Persistent by default
}

// Test actual connection to RabbitMQ server
// Note: This test requires a running RabbitMQ instance
#[test]
fn test_rabbitmq_connection() {
    // Create a config for connecting to a local RabbitMQ
    let config = create_test_config();

    // Create a tokio runtime for async code
    let rt = Runtime::new().unwrap();

    // Try to connect
    let client_result = rt.block_on(async {
        RabbitMQClient::new(config).await
    });

    // Assert that we can connect successfully
    assert!(client_result.is_ok(), "Failed to connect to RabbitMQ: {:?}", client_result.err());

    // If connection was successful, close it
    if let Ok(client) = client_result {
        let close_result = rt.block_on(async {
            client.close().await
        });
        assert!(close_result.is_ok(), "Failed to close RabbitMQ connection: {:?}", close_result.err());
    }
}



// Test RabbitMQMessage struct
#[test]
fn test_rabbitmq_message_structure() {
    let props = MessageProperties::default();
    let message = RabbitMQMessage {
        exchange: "test.exchange".to_string(),
        routing_key: "test.routing.key".to_string(),
        payload: r#"{"test":"data"}"#.to_string(),
        properties: Some(props),
    };

    assert_eq!(message.exchange, "test.exchange");
    assert_eq!(message.routing_key, "test.routing.key");
    assert_eq!(message.payload, r#"{"test":"data"}"#);
    assert!(message.properties.is_some());
}

// Test QueueInfo struct
#[test]
fn test_queue_info_structure() {
    let queue_info = QueueInfo {
        name: "test.queue".to_string(),
        durable: true,
        exclusive: false,
        auto_delete: false,
    };

    assert_eq!(queue_info.name, "test.queue");
    assert!(queue_info.durable);
    assert!(!queue_info.exclusive);
    assert!(!queue_info.auto_delete);
}

// Test ExchangeInfo struct
#[test]
fn test_exchange_info_structure() {
    let exchange_info = ExchangeInfo {
        name: "test.exchange".to_string(),
        kind: "direct".to_string(),
        durable: true,
        auto_delete: false,
    };

    assert_eq!(exchange_info.name, "test.exchange");
    assert_eq!(exchange_info.kind, "direct");
    assert!(exchange_info.durable);
    assert!(!exchange_info.auto_delete);
}

// Test RabbitMQConfig URI generation
#[test]
fn test_rabbitmq_config_to_uri() {
    let config = create_test_config();
    let uri = config.to_uri();

    assert_eq!(uri, "amqp://guest:guest@localhost:5672/%2F");
}

// Note: For the following tests, you would typically use a mock
// to avoid needing a real RabbitMQ connection. Here are some
// test skeletons that you can implement later with proper mocking:

#[test]
#[ignore] // Ignore this test until you have proper mocking set up
fn test_rabbitmq_client_new() {
    // This would test creating a new client
    // You'll need to mock the Connection and Channel
}

#[test]
#[ignore] // Ignore this test until you have proper mocking set up
fn test_publish_message() {
    // This would test publishing a message
    // You'll need to mock the Channel's basic_publish method
}

#[test]
#[ignore] // Ignore this test until you have proper mocking set up
fn test_declare_queue() {
    // This would test declaring a queue
    // You'll need to mock the Channel's queue_declare method
}

#[test]
#[ignore] // Ignore this test until you have proper mocking set up
fn test_declare_exchange() {
    // This would test declaring an exchange
    // You'll need to mock the Channel's exchange_declare method
}

#[test]
#[ignore] // Ignore this test until you have proper mocking set up
fn test_list_exchanges() {
    // This would test listing exchanges
    // You'll need to mock the appropriate method
}