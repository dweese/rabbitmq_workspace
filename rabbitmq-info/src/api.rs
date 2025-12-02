// rabbitmq-info/src/api.rs

use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

use rabbitmq_config::RabbitMQConfig;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("HTTP error: {0}")]
    HttpError(String),
}

pub struct RabbitMQApiClient {
    client: Client,
    config: RabbitMQConfig,
}

impl RabbitMQApiClient {
    pub fn new(config: &RabbitMQConfig) -> Result<Self, ApiError> {
        Ok(Self {
            client: Client::new(),
            config: config.clone(),
        })
    }

    fn build_url(&self, path: &str) -> String {
        format!(
            "http://{}:{}{}",
            self.config.host, self.config.management_port, path
        )
    }

    async fn get_list(&self, path: &str) -> Result<Vec<Value>, ApiError> {
        let url = self.build_url(path);
        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        if response.status().is_success() {
            let items: Vec<Value> = response.json().await?;
            Ok(items)
        } else {
            Err(ApiError::HttpError(format!(
                "Failed to get list from {}: {}",
                path,
                response.status()
            )))
        }
    }

    async fn get_value(&self, path: &str) -> Result<Value, ApiError> {
        let url = self.build_url(path);
        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        if response.status().is_success() {
            let value: Value = response.json().await?;
            Ok(value)
        } else {
            Err(ApiError::HttpError(format!(
                "Failed to get value from {}: {}",
                path,
                response.status()
            )))
        }
    }

    pub async fn is_alive(&self) -> Result<bool, ApiError> {
        let url = self.build_url("/api/aliveness-test/%2F");
        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn get_overview(&self) -> Result<Value, ApiError> {
        self.get_value("/api/overview").await
    }

    pub async fn get_queues(&self) -> Result<Vec<Value>, ApiError> {
        self.get_list("/api/queues").await
    }

    pub async fn get_exchanges(&self) -> Result<Vec<Value>, ApiError> {
        self.get_list("/api/exchanges").await
    }

    pub async fn get_bindings(&self) -> Result<Vec<Value>, ApiError> {
        self.get_list("/api/bindings").await
    }

    pub async fn get_vhosts(&self) -> Result<Vec<Value>, ApiError> {
        self.get_list("/api/vhosts").await
    }

    pub async fn get_definitions(&self) -> Result<Value, ApiError> {
        self.get_value("/api/definitions").await
    }
}
