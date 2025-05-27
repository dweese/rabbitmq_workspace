use rabbitmq_config::*;
use std::path::Path;
use crate::common::test_utils::create_test_config;

#[test]
fn test_basic_config_serialization() {
    // Test serialization of RabbitMQConfig (the simple config)
    let config = RabbitMQConfig {
        host: "test-host".to_string(),
        port: 5673,
        username: "test-user".to_string(),
        password: "test-password".to_string(),
        vhost: "test-vhost".to_string(),
    };

    let json = serde_json::to_string_pretty(&config).expect("Failed to serialize");
    let deserialized: RabbitMQConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(config.host, deserialized.host);
    assert_eq!(config.port, deserialized.port);
    assert_eq!(config.username, deserialized.username);
    assert_eq!(config.password, deserialized.password);
    assert_eq!(config.vhost, deserialized.vhost);
}
