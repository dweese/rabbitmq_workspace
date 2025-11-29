#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cli_parsing_connect_command() {
        use clap::Parser;
        
        // Test basic connect command
        let args = vec![
            "messaging_commands",
            "connect",
            "--protocol", "amqp",
            "--host", "localhost",
            "--port", "5672",
            "--username", "guest",
            "--password", "guest"
        ];
        
        let cli = crate::Cli::try_parse_from(args).unwrap();
        
        match cli.command {
            crate::Commands::Connect { protocol, host, port, username, password } => {
                assert_eq!(protocol, "amqp");
                assert_eq!(host, "localhost");
                assert_eq!(port, 5672);
                assert_eq!(username, Some("guest".to_string()));
                assert_eq!(password, Some("guest".to_string()));
            }
            _ => panic!("Expected Connect command"),
        }
    }

    #[test]
    fn test_cli_parsing_connect_with_defaults() {
        use clap::Parser;
        
        // Test connect command with default host
        let args = vec![
            "messaging_commands",
            "connect",
            "--protocol", "amqp",
            "--port", "5672"
        ];
        
        let cli = crate::Cli::try_parse_from(args).unwrap();
        
        match cli.command {
            crate::Commands::Connect { protocol, host, port, username, password } => {
                assert_eq!(protocol, "amqp");
                assert_eq!(host, "localhost"); // Default value
                assert_eq!(port, 5672);
                assert_eq!(username, None);
                assert_eq!(password, None);
            }
            _ => panic!("Expected Connect command"),
        }
    }

    #[test]
    fn test_cli_parsing_reload_command() {
        use clap::Parser;
        
        // Test reload command with default config file
        let args = vec![
            "messaging_commands",
            "reload"
        ];
        
        let cli = crate::Cli::try_parse_from(args).unwrap();
        
        match cli.command {
            crate::Commands::Reload { config_file } => {
                assert_eq!(config_file, "artifacts/rabbitmq_config.json");
            }
            _ => panic!("Expected Reload command"),
        }
    }

    #[test]
    fn test_cli_parsing_reload_custom_config() {
        use clap::Parser;
        
        // Test reload command with custom config file
        let args = vec![
            "messaging_commands",
            "reload",
            "--config-file", "/custom/path/config.json"
        ];
        
        let cli = crate::Cli::try_parse_from(args).unwrap();
        
        match cli.command {
            crate::Commands::Reload { config_file } => {
                assert_eq!(config_file, "/custom/path/config.json");
            }
            _ => panic!("Expected Reload command"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        use clap::Parser;
        
        let args = vec![
            "messaging_commands",
            "--verbose",
            "connect",
            "--protocol", "amqp",
            "--port", "5672"
        ];
        
        let cli = crate::Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_config_reload_with_valid_json() {
        // Create a temporary file with valid JSON
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"{
            "exchanges": [
                {"name": "test_exchange", "type": "direct"}
            ],
            "queues": [
                {"name": "test_queue", "durable": true}
            ],
            "bindings": [
                {"source": "test_exchange", "destination": "test_queue", "routing_key": "test"}
            ],
            "users": [
                {"name": "test_user", "password": "test_pass"}
            ]
        }"#;
        
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        // This test would need to capture log output to verify proper parsing
        // For now, we just ensure it doesn't panic
        crate::reload_config(temp_file.path().to_str().unwrap());
    }

    #[test]
    fn test_config_reload_with_invalid_json() {
        // Create a temporary file with invalid JSON
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_content = r#"{ "invalid": json content }"#;
        
        temp_file.write_all(invalid_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        // This should handle the error gracefully (logged, not panicked)
        crate::reload_config(temp_file.path().to_str().unwrap());
    }

    #[test]
    fn test_config_reload_with_nonexistent_file() {
        // Test with a file that doesn't exist
        crate::reload_config("/nonexistent/path/config.json");
        // Should handle the error gracefully (logged, not panicked)
    }

    #[test]
    fn test_config_reload_with_empty_json() {
        // Create a temporary file with empty JSON object
        let mut temp_file = NamedTempFile::new().unwrap();
        let empty_content = "{}";
        
        temp_file.write_all(empty_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        crate::reload_config(temp_file.path().to_str().unwrap());
    }

    #[test]
    fn test_config_reload_with_array_json() {
        // Create a temporary file with JSON array (should trigger warning)
        let mut temp_file = NamedTempFile::new().unwrap();
        let array_content = r#"[{"key": "value"}]"#;
        
        temp_file.write_all(array_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        crate::reload_config(temp_file.path().to_str().unwrap());
    }

    #[test]
    fn test_cli_missing_required_args() {
        use clap::Parser;
        
        // Test that missing required arguments fail parsing
        let args = vec![
            "messaging_commands",
            "connect",
            "--protocol", "amqp"
            // Missing required --port argument
        ];
        
        let result = crate::Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_invalid_port() {
        use clap::Parser;
        
        // Test that invalid port values fail parsing
        let args = vec![
            "messaging_commands",
            "connect",
            "--protocol", "amqp",
            "--port", "invalid_port"
        ];
        
        let result = crate::Cli::try_parse_from(args);
        assert!(result.is_err());
    }
}