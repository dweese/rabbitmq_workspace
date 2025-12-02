// rabbitmq-info/src/collector/mod.rs

use crate::api::{ApiError, RabbitMQApiClient};
use crate::{BindingInfo, ExchangeInfo, QueueInfo, ServerInfo};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

// Define the RabbitMQInfo struct
#[derive(Debug, Serialize, Deserialize)]
pub struct RabbitMQInfo {
    pub server: ServerInfo,
    pub exchanges: Vec<ExchangeInfo>,
    pub queues: Vec<QueueInfo>,
    pub bindings: Vec<BindingInfo>,
    pub vhosts: Vec<String>,
}

// Implement methods for RabbitMQInfo
impl RabbitMQInfo {
    pub fn new(
        server: ServerInfo,
        exchanges: Vec<ExchangeInfo>,
        queues: Vec<QueueInfo>,
        bindings: Vec<BindingInfo>,
        vhosts: Vec<String>,
    ) -> Self {
        Self {
            server,
            exchanges,
            queues,
            bindings,
            vhosts,
        }
    }
}

pub struct RabbitMQInfoCollector {
    client: RabbitMQApiClient,
}

impl RabbitMQInfoCollector {
    pub fn new(client: RabbitMQApiClient) -> Self {
        Self { client }
    }

    pub async fn collect_all(&self) -> Result<RabbitMQInfo, ApiError> {
        // Define the futures with explicit types
        let overview_future: Pin<Box<dyn Future<Output = Result<Value, ApiError>> + Send>> = Box::pin(self.client.get_overview());
        let exchanges_future: Pin<Box<dyn Future<Output = Result<Vec<Value>, ApiError>> + Send>> = Box::pin(self.client.get_exchanges());
        let queues_future: Pin<Box<dyn Future<Output = Result<Vec<Value>, ApiError>> + Send>> = Box::pin(self.client.get_queues());
        let bindings_future: Pin<Box<dyn Future<Output = Result<Vec<Value>, ApiError>> + Send>> = Box::pin(self.client.get_bindings());
        let vhosts_future: Pin<Box<dyn Future<Output = Result<Vec<Value>, ApiError>> + Send>> = Box::pin(self.client.get_vhosts());

        // Await all futures
        let (overview, exchanges, queues, bindings, vhosts) = tokio::join!(
            overview_future,
            exchanges_future,
            queues_future,
            bindings_future,
            vhosts_future
        );

        // Process the results
        let server = self.process_overview(overview?);
        let exchanges = self.process_exchanges(exchanges?);
        let queues = self.process_queues(queues?);
        let bindings = self.process_bindings(bindings?);
        let vhosts = self.process_vhosts(vhosts?);

        Ok(RabbitMQInfo::new(
            server, exchanges, queues, bindings, vhosts,
        ))
    }

    // Process the overview response from the API
    fn process_overview(&self, overview: serde_json::Value) -> ServerInfo {
        ServerInfo {
            version: self.extract_string(&overview, "rabbitmq_version"),
            erlang_version: self.extract_string(&overview, "erlang_version"),
            cluster_name: self.extract_string(&overview, "cluster_name"),
            management_version: self.extract_string(&overview, "management_version"),
            uptime: self.extract_number(&overview, "uptime").unwrap_or(0),
            node_name: self.extract_string(&overview, "node"),
        }
    }

    // Process exchanges response from the API
    fn process_exchanges(&self, exchanges: Vec<serde_json::Value>) -> Vec<ExchangeInfo> {
        exchanges
            .into_iter()
            .map(|exchange| ExchangeInfo {
                name: self.extract_string(&exchange, "name"),
                vhost: self.extract_string(&exchange, "vhost"),
                exchange_type: self.extract_string(&exchange, "type"),
                durable: self.extract_bool(&exchange, "durable").unwrap_or(false),
                auto_delete: self.extract_bool(&exchange, "auto_delete").unwrap_or(false),
                internal: self.extract_bool(&exchange, "internal").unwrap_or(false),
                arguments: self
                    .extract_value(&exchange, "arguments")
                    .unwrap_or_else(|| serde_json::json!({})),
            })
            .collect()
    }

    // Process queues response from the API
    fn process_queues(&self, queues: Vec<serde_json::Value>) -> Vec<QueueInfo> {
        queues
            .into_iter()
            .map(|queue| QueueInfo {
                name: self.extract_string(&queue, "name"),
                vhost: self.extract_string(&queue, "vhost"),
                durable: self.extract_bool(&queue, "durable").unwrap_or(false),
                auto_delete: self.extract_bool(&queue, "auto_delete").unwrap_or(false),
                exclusive: self.extract_bool(&queue, "exclusive").unwrap_or(false),
                arguments: self
                    .extract_value(&queue, "arguments")
                    .unwrap_or_else(|| serde_json::json!({})),
                messages: self.extract_number(&queue, "messages"),
                messages_ready: self.extract_number(&queue, "messages_ready"),
                messages_unacknowledged: self.extract_number(&queue, "messages_unacknowledged"),
            })
            .collect()
    }

    // Process bindings response from the API
    fn process_bindings(&self, bindings: Vec<serde_json::Value>) -> Vec<BindingInfo> {
        bindings
            .into_iter()
            .map(|binding| BindingInfo {
                source: self.extract_string(&binding, "source"),
                destination: self.extract_string(&binding, "destination"),
                destination_type: self.extract_string(&binding, "destination_type"),
                routing_key: self.extract_string(&binding, "routing_key"),
                arguments: self
                    .extract_value(&binding, "arguments")
                    .unwrap_or_else(|| serde_json::json!({})),
                vhost: self.extract_string(&binding, "vhost"),
            })
            .collect()
    }

    // Process vhosts response from the API
    fn process_vhosts(&self, vhosts: Vec<serde_json::Value>) -> Vec<String> {
        vhosts
            .into_iter()
            .filter_map(|vhost| {
                self.extract_value(&vhost, "name")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .collect()
    }

    // Helper methods to extract values from JSON
    fn extract_string(&self, value: &serde_json::Value, key: &str) -> String {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }

    fn extract_bool(&self, value: &serde_json::Value, key: &str) -> Option<bool> {
        value.get(key).and_then(|v| v.as_bool())
    }

    fn extract_number(&self, value: &serde_json::Value, key: &str) -> Option<u64> {
        value.get(key).and_then(|v| v.as_u64())
    }

    fn extract_value(&self, value: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
        value.get(key).cloned()
    }
}
