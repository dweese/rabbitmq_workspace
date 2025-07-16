// rabbitmq-config/src/models.rs
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Message-related models (already in your code)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub delivery_mode: Option<u8>,
    pub priority: Option<u8>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    pub expiration: Option<String>,
    pub message_id: Option<String>,
    pub timestamp: Option<bool>,
    pub user_id: Option<String>,
    pub app_id: Option<String>,
}

// Remove this entire impl block - it's now handled by #[derive(Default)]
// impl Default for MessageProperties {
//     fn default() -> Self {
//         Self {
//             content_type: None,
//             content_encoding: None,
//             delivery_mode: None,
//             priority: None,
//             correlation_id: None,
//             reply_to: None,
//             expiration: None,
//             message_id: None,
//             timestamp: None,
//             user_id: None,
//             app_id: None,
//         }
//     }
// }


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQMessage {
    pub exchange: String,
    pub routing_key: String,
    pub payload: Vec<u8>,
    pub properties: Option<MessageProperties>,
}


// Exchange info (already in your code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub arguments: HashMap<String, serde_json::Value>,
}

// Queue info (already in your code, but may need adjustments)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub name: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub exclusive: bool,
    pub arguments: HashMap<String, serde_json::Value>,
}

impl Default for QueueInfo {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            durable: true,
            auto_delete: false,
            exclusive: false,
            arguments: HashMap::new(),
        }
    }
}

// RabbitMQ Server Definition - just the parts we need
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQServerDefinition {
    pub rabbit_version: String,
    pub rabbitmq_version: String,
    pub product_name: String,
    pub product_version: String,
    pub rabbitmq_definition_format: String,
    pub original_cluster_name: String,
    pub explanation: Option<String>,
    pub users: Vec<UserDefinition>,
    pub vhosts: Vec<VhostDefinition>,
    pub permissions: Vec<PermissionDefinition>,
    pub topic_permissions: Vec<TopicPermissionDefinition>,
    pub parameters: Vec<serde_json::Value>,
    pub global_parameters: Vec<GlobalParameterDefinition>,
    pub policies: Vec<serde_json::Value>,
    pub queues: Vec<QueueDefinition>,
    pub exchanges: Vec<ExchangeDefinition>,
    pub bindings: Vec<BindingDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDefinition {
    pub name: String,
    pub password_hash: String,
    pub hashing_algorithm: String,
    pub tags: Vec<String>,
    pub limits: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhostDefinition {
    pub name: String,
    pub description: String,
    pub metadata: VhostMetadata,
    pub tags: Vec<String>,
    pub default_queue_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhostMetadata {
    pub description: String,
    pub tags: Vec<String>,
    pub default_queue_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDefinition {
    pub user: String,
    pub vhost: String,
    pub configure: String,
    pub write: String,
    pub read: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPermissionDefinition {
    pub user: String,
    pub vhost: String,
    pub exchange: String,
    pub write: String,
    pub read: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalParameterDefinition {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueDefinition {
    pub name: String,
    pub vhost: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeDefinition {
    pub name: String,
    pub vhost: String,
    #[serde(rename = "type")]
    pub exchange_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingDefinition {
    pub source: String,
    pub vhost: String,
    pub destination: String,
    pub destination_type: String,
    pub routing_key: String,
    pub arguments: HashMap<String, serde_json::Value>,
}