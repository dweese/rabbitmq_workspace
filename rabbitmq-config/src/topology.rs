// rabbitmq-config/src/topology.rs
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
    pub name: String,
    pub exchange_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub arguments: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub name: String,
    pub durable: bool,
    pub exclusive: bool,
    pub auto_delete: bool,
    pub arguments: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub queue: String,
    pub exchange: String,
    pub routing_key: String,
    pub arguments: HashMap<String, String>,
}

pub trait TopologyDataSource {
    fn get_exchanges(&self) -> Vec<Exchange>;
    fn get_queues(&self) -> Vec<Queue>;
    fn get_bindings(&self) -> Vec<Binding>;
}

// Add these to your topology.rs file or create a new message.rs file

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProperties {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub delivery_mode: Option<u8>,
}

impl Default for MessageProperties {
    fn default() -> Self {
        Self {
            content_type: None,
            content_encoding: None,
            delivery_mode: Some(1), // Non-persistent by default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQMessage {
    pub exchange: String,
    pub routing_key: String,
    pub payload: String,
    pub properties: Option<MessageProperties>,
}