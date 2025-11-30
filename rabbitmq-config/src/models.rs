// rabbitmq-config/src/models.rs

use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RabbitMQServerDefinition {
    pub rabbitmq_version: String,
    pub users: Vec<UserDefinition>,
    pub vhosts: Vec<VhostDefinition>,
    pub permissions: Vec<PermissionDefinition>,
    pub topic_permissions: Vec<TopicPermissionDefinition>,
    pub parameters: Vec<GlobalParameterDefinition>,
    pub policies: Vec<serde_json::Value>, // Policies can have varied structure
    pub queues: Vec<QueueDefinition>,
    pub exchanges: Vec<ExchangeDefinition>,
    pub bindings: Vec<BindingDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDefinition {
    pub name: String,
    pub password_hash: String,
    pub hashing_algorithm: String,
    pub tags: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VhostDefinition {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PermissionDefinition {
    pub user: String,
    pub vhost: String,
    pub configure: String,
    pub write: String,
    pub read: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopicPermissionDefinition {
    pub user: String,
    pub vhost: String,
    pub exchange: String,
    pub write: String,
    pub read: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalParameterDefinition {
    pub name: String,
    pub value: serde_json::Value,
    pub vhost: String,
    pub component: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueDefinition {
    pub name: String,
    pub vhost: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub arguments: FieldTable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeDefinition {
    pub name: String,
    pub vhost: String,
    pub r#type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub arguments: FieldTable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BindingDefinition {
    pub source: String,
    pub vhost: String,
    pub destination: String,
    pub destination_type: String,
    pub routing_key: String,
    pub arguments: FieldTable,
}

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub name: String,
    pub durable: bool,
    pub exclusive: bool,
    pub auto_delete: bool,
    pub arguments: FieldTable,
}

#[derive(Debug, Clone)]
pub struct ExchangeInfo {
    pub name: String,
    pub kind: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub arguments: FieldTable,
}

#[derive(Debug, Clone)]
pub struct RabbitMQMessage {
    pub exchange: String,
    pub routing_key: String,
    pub payload: Vec<u8>,
    pub properties: Option<MessageProperties>,
}

#[derive(Debug, Clone, Default)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub delivery_mode: Option<u8>,
    pub priority: Option<u8>,
}
