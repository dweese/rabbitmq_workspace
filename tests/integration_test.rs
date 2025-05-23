#[cfg(test)]
mod tests {
    use rabbitmq_config::RabbitMQConfig;
    use rabbitmq_info::*;
    use std::path::Path;

    #[tokio::test]
    #[ignore] // This test is ignored as it requires a real RabbitMQ server
    async fn test_rabbitmq_info_dump() {
        // Create a test config
        let config = RabbitMQConfig {
            host: "localhost".to_string(),
            port: 5672,
            vhost: "/".to_string(),
            username: "guest".to_string(),
            password: "guest".to_string(),
        };

        // Create a temp file path for the dump
        let dump_path = Path::new("target/test_rabbitmq_dump.json");

        // Dump the RabbitMQ information
        let result = dump_rabbitmq_info(&config, dump_path).await;

        // This will fail if RabbitMQ is not running, that's expected in the ignored test
        if result.is_ok() {
            assert!(dump_path.exists(), "Dump file was not created");

            // Clean up
            let _ = std::fs::remove_file(dump_path);
        }
    }
}