use rabbitmq_config::*;

#[test]
fn test_rabbitmq_config_to_uri() {
    // Test URI generation for simple config
    let config = RabbitMQConfig {
        host: "localhost".to_string(),
        port: 5672,
        username: "guest".to_string(),
        password: "guest".to_string(),
        vhost: "/".to_string(),
    };

    let uri = config.to_uri();
    assert_eq!(uri, "amqp://guest:guest@localhost:5672/");

    // Test with non-default values
    let config2 = RabbitMQConfig {
        host: "rabbitmq.example.com".to_string(),
        port: 5673,
        username: "test-user".to_string(),
        password: "test-pass".to_string(),
        vhost: "test-vhost".to_string(),
    };

    let uri2 = config2.to_uri();
    assert_eq!(uri2, "amqp://test-user:test-pass@rabbitmq.example.com:5673/test-vhost");
}

#[test]
fn test_rabbitmq_error_debug() {
    // Test that RabbitMQError can be formatted for debugging
    let err = RabbitMQError::ConfigError("Test error".to_string());
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("ConfigError"));
    assert!(debug_str.contains("Test error"));
}