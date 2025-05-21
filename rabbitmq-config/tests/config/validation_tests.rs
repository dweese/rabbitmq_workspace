use rabbitmq_config::*;
use std::collections::HashMap;

#[test]
fn test_empty_fields() {
    // Test empty host
    let mut config = RabbitMQFullConfig::default();
    config.connection.host = "".to_string();

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: RabbitMQFullConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.connection.host, "");

    // Test empty username/password
    config.connection.username = "".to_string();
    config.connection.password = "".to_string();

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: RabbitMQFullConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.connection.username, "");
    assert_eq!(deserialized.connection.password, "");
}