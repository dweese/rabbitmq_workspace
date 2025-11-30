// rabbitmq-config/tests/common/test_utils.rs

use rabbitmq_config::*;
use std::path::Path;

/// Creates a standard test configuration for RabbitMQ connections
pub fn create_test_config() -> RabbitMQConfig {
    RabbitMQConfig {
        host: "localhost".to_string(),
        amqp_port: 5672,
        management_port: 15672,
        username: "guest".to_string(),
        password: "guest".to_string(),
        vhost: "%2F".to_string(), // URL-encoded form of "/"
    }
}

/// Saves a configuration to a temporary file and returns the path
pub fn save_config_to_temp(config: &RabbitMQConfig) -> std::path::PathBuf {
    // Create test directory if it doesn't exist
    let test_dir = Path::new("tests/fixtures/sample_configs");
    std::fs::create_dir_all(test_dir).expect("Failed to create test directory");

    // Generate a unique filename using current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let config_path = test_dir.join(format!("temp_config_{timestamp}.json"));

    // Save the configuration
    let file = std::fs::File::create(&config_path).expect("Failed to create config file");
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, config).expect("Failed to serialize config");

    config_path
}
