use rabbitmq_config::*;
use std::collections::HashMap;

#[test]
fn test_empty_fields() {
    // Test empty host
    let mut config = RabbitMQConfig::default();
    config.host = "".to_string();

    
    
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: RabbitMQConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.host, "");

    // Test empty username/password
    config.username = "".to_string();
    config.password = "".to_string();

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: RabbitMQConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.username, "");
    assert_eq!(deserialized.password, "");
}