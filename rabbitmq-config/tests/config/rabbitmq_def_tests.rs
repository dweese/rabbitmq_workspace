// tests/config/rabbitmq_def_tests.rs

#[cfg(test)]
mod tests {
    use rabbitmq_config::RabbitMQServerDefinition;
    
    #[test]
    fn test_parse_rabbitmq_definition() {
        let json = r#"{
            "rabbit_version": "0.0.0",
            "rabbitmq_version": "0.0.0",
            "product_name": "RabbitMQ",
            "product_version": "0.0.0",
            "rabbitmq_definition_format": "cluster",
            "original_cluster_name": "rabbit@fedora",
            "explanation": "Definitions of cluster 'rabbit@fedora' from RabbitMQ 20250523",
            "users": [
                {
                    "name": "user_rust",
                    "password_hash": "82YTUWM5YRaGgSTGjiVvA5FLbGjZOsUtFHDV9vqL6T5g72PP",
                    "hashing_algorithm": "rabbit_password_hashing_sha256",
                    "tags": ["administrator"],
                    "limits": {}
                }
            ],
            "vhosts": [
                {
                    "name": "/",
                    "description": "Default virtual host",
                    "metadata": {
                        "description": "Default virtual host",
                        "tags": [],
                        "default_queue_type": "classic"
                    },
                    "tags": [],
                    "default_queue_type": "classic"
                }
            ],
            "permissions": [],
            "topic_permissions": [],
            "parameters": [],
            "global_parameters": [],
            "policies": [],
            "queues": [],
            "exchanges": [],
            "bindings": []
        }"#;
        
        let definition: RabbitMQServerDefinition = serde_json::from_str(json)
            .expect("Failed to parse RabbitMQ server definition");
            
        assert_eq!(definition.product_name, "RabbitMQ");
        assert_eq!(definition.users.len(), 1);
        assert_eq!(definition.users[0].name, "user_rust");
        assert_eq!(definition.vhosts.len(), 1);
        assert_eq!(definition.vhosts[0].name, "/");
    }
}