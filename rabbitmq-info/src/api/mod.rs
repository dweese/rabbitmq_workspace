// rabbitmq-info/src/api/mod.rs

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rabbitmq_config::RabbitMQConfig;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug)]
pub struct RabbitMQApiClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("API response error: {status_code} - {message}")]
    ResponseError { status_code: u16, message: String },

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl RabbitMQApiClient {
    pub fn new(config: &RabbitMQConfig) -> Result<Self, ApiError> {
        // Create HTTP client with timeout
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        // Construct base URL for management API using the correct port
        let base_url = format!("http://{}:{}/api", config.host, config.management_port);

        // Create basic auth header
        let auth_str = format!("{}:{}", config.username, config.password);
        let auth_header = format!("Basic {}", BASE64.encode(auth_str));

        Ok(Self {
            client,
            base_url,
            auth_header,
        })
    }

    // Generic method to make API requests
    async fn get<T>(&self, endpoint: &str) -> Result<T, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        let status = response.status();

        if status == StatusCode::OK {
            let json = response.json::<T>().await?;
            Ok(json)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ApiError::ResponseError {
                status_code: status.as_u16(),
                message: error_text,
            })
        }
    }

    /// Check if the RabbitMQ server is alive and responding
    /// Returns Ok(true) if the server responds, Err otherwise
    pub async fn is_alive(&self) -> Result<bool, ApiError> {
        match self.get_overview().await {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    // Get overview information
    pub async fn get_overview(&self) -> Result<serde_json::Value, ApiError> {
        self.get("/overview").await
    }

    // Get all exchanges
    pub async fn get_exchanges(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        self.get("/exchanges").await
    }

    // Get all queues
    pub async fn get_queues(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        self.get("/queues").await
    }

    // Get all bindings
    pub async fn get_bindings(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        self.get("/bindings").await
    }

    // Get all vhosts
    pub async fn get_vhosts(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        self.get("/vhosts").await
    }

    // Get all users
    pub async fn get_users(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        self.get("/users").await
    }
}
