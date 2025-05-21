use rabbitmq_config::*;
use std::collections::HashMap;
use std::path::Path;

/// Creates a standard test configuration for RabbitMQFullConfig
pub fn create_test_config() -> RabbitMQFullConfig {
    let mut config = RabbitMQFullConfig::default();

    // Set some reasonable test values
    config.connection.host = "test-host".to_string();
    config.connection.port = 5672;
    config.connection.username = "test-user".to_string();
    config.connection.password = "test-pass".to_string();
    config.connection.vhost = "/".to_string();

    // Add a test exchange
    config.exchanges.push(ExchangeConfig {
        name: "test.exchange".to_string(),
        exchange_type: "topic".to_string(),
        durable: true,
        auto_delete: false,
        internal: false,
        arguments: HashMap::new(),
    });

    // Add a test queue
    config.queues.push(QueueConfig {
        name: "test.queue".to_string(),
        durable: true,
        exclusive: false,
        auto_delete: false,
        arguments: HashMap::new(),
    });

    // Add a test binding
    config.bindings.push(BindingConfig {
        exchange: "test.exchange".to_string(),
        queue: "test.queue".to_string(),
        routing_key: "test.key".to_string(),
        arguments: HashMap::new(),
    });

    config
}

/// Saves a configuration to a temporary file and returns the path
pub fn save_config_to_temp(config: &RabbitMQFullConfig) -> std::path::PathBuf {
    // Create test directory if it doesn't exist
    let test_dir = Path::new("tests/fixtures/sample_configs");
    std::fs::create_dir_all(test_dir).expect("Failed to create test directory");

    // Generate a unique filename using current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let config_path = test_dir.join(format!("temp_config_{}.json", timestamp));

    // Save the configuration
    config.save_to_file(&config_path).expect("Failed to save config");

    config_path
}