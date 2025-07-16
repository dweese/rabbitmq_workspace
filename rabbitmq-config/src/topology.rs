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

#[allow(dead_code)] // Future topology management feature
pub trait TopologyDataSource {
    fn get_exchanges(&self) -> Vec<Exchange>;
    fn get_queues(&self) -> Vec<Queue>;
    fn get_bindings(&self) -> Vec<Binding>;
}

// Remove the duplicate MessageProperties - it belongs in models.rs