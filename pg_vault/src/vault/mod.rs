// pg_vault/src/vault/mod.rs
//! Vault module for pg_vault
//!
//! Provides secure PostgreSQL connections with hardware token authentication.
//! This module combines authentication providers with database connection management
//! to create a secure "vault" for database access.

use crate::auth::{AuthConfig, AuthError, AuthProviderFactory, YubikeyAuth};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_postgres::{Client, Error as PgError, NoTls};

pub mod connection;
pub mod secure_connection;

// Re-export for convenience
pub use connection::Connection;
pub use secure_connection::SecureConnection;

/// Result type for vault operations
pub type VaultResult<T> = Result<T, VaultError>;

/// Vault-specific errors
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Database connection error: {0}")]
    Database(#[from] PgError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Connection pool error: {0}")]
    Pool(String),

    #[error("Timeout error: {0}")]
    Timeout(String),
}

/// Configuration for database connections
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username for database connection
    pub username: String,
    /// password
    pub password: Option<String>,

    pub connect_timeout: u64,
    /// Query timeout in seconds
    pub query_timeout: u64,
    /// SSL mode
    pub ssl_mode: SslMode,
    /// Application name for connection identification
    pub application_name: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: None,
            connect_timeout: 30,
            query_timeout: 60,
            ssl_mode: SslMode::Prefer,
            application_name: Some("pg_vault".to_string()),
        }
    }
}

/// SSL connection modes
#[derive(Debug, Clone)]
pub enum SslMode {
    /// Disable SSL
    Disable,
    /// Allow SSL but don't require it
    Allow,
    /// Prefer SSL but fall back to non-SSL
    Prefer,
    /// Require SSL
    Require,
    /// Require SSL with certificate verification
    VerifyFull,
}

/// Session information for tracking active connections
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Unique session identifier
    pub session_id: String,
    /// When the session was created
    pub created_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Hardware token serial used for authentication
    pub token_serial: Option<String>,
    /// Database user
    pub database_user: String,
    /// Database name
    pub database_name: String,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl SessionInfo {
    fn new(database_user: String, database_name: String, token_serial: Option<String>) -> Self {
        let now = SystemTime::now();
        let session_id = Self::generate_session_id();

        Self {
            session_id,
            created_at: now,
            last_activity: now,
            token_serial,
            database_user,
            database_name,
            metadata: HashMap::new(),
        }
    }

    fn generate_session_id() -> String {
        use base64::{engine::general_purpose, Engine as _};
        use sha2::{Digest, Sha256};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut hasher = Sha256::new();
        hasher.update(now.to_be_bytes());
        hasher.update(b"pg_vault_session");

        let result = hasher.finalize();
        general_purpose::STANDARD.encode(&result[..16]) // Use first 16 bytes for shorter ID
    }

    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now();
    }

    fn is_expired(&self, max_idle: Duration) -> bool {
        self.last_activity.elapsed().unwrap_or(Duration::ZERO) > max_idle
    }
}

/// Main vault structure for managing secure database connections
pub struct Vault {
    /// Authentication provider
    auth_provider: Arc<dyn YubikeyAuth>,
    /// Authentication configuration
    #[allow(dead_code)]
    auth_config: AuthConfig,
    /// Database configuration
    db_config: DatabaseConfig,
    /// Active sessions
    sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionInfo>>>,
    /// Vault configuration
    config: VaultConfig,
}

/// Configuration for the vault itself
#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Maximum session idle time before requiring re-authentication
    pub max_session_idle: Duration,
    /// Maximum total session lifetime
    pub max_session_lifetime: Duration,
    /// Whether to require hardware token for each connection
    pub require_token_per_connection: bool,
    /// Maximum number of concurrent sessions
    pub max_concurrent_sessions: usize,
    /// Challenge length for authentication
    pub challenge_length: usize,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            max_session_idle: Duration::from_secs(30 * 60), // 30 minutes
            max_session_lifetime: Duration::from_secs(8 * 60 * 60), // 8 hours
            require_token_per_connection: false,
            max_concurrent_sessions: 10,
            challenge_length: 32,
        }
    }
}

impl Vault {
    /// Create a new vault with mock authentication (for development)
    pub fn new_with_mock(db_config: DatabaseConfig) -> Self {
        let auth_provider = Arc::from(AuthProviderFactory::create_mock_provider());
        let auth_config = AuthConfig::default();
        let config = VaultConfig::default();

        Self {
            auth_provider,
            auth_config,
            db_config,
            sessions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Create a new vault with hardware authentication
    pub fn new_with_hardware(
        auth_config: AuthConfig,
        db_config: DatabaseConfig,
        vault_config: VaultConfig,
    ) -> VaultResult<Self> {
        let auth_provider = Arc::from(AuthProviderFactory::create_provider_with_fallback(
            auth_config.clone(),
        ));

        Ok(Self {
            auth_provider,
            auth_config,
            db_config,
            sessions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            config: vault_config,
        })
    }

    /// Create a new vault with custom authentication provider
    pub fn new_with_provider(
        auth_provider: Arc<dyn YubikeyAuth>,
        auth_config: AuthConfig,
        db_config: DatabaseConfig,
        vault_config: VaultConfig,
    ) -> Self {
        Self {
            auth_provider,
            auth_config,
            db_config,
            sessions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            config: vault_config,
        }
    }

    /// Authenticate and create a new secure database connection
    pub async fn connect(&self, pin: &str) -> VaultResult<SecureConnection> {
        // Clean up expired sessions first
        self.cleanup_expired_sessions().await;

        // Check session limits
        self.check_session_limits().await?;

        // Perform hardware token authentication
        let challenge = self.generate_challenge();
        let response = self.auth_provider.challenge_response(&challenge, pin)?;

        // Validate authentication response
        crate::utils::validate_response(&response, Some(20))?; // HMAC-SHA1 is 20 bytes

        // Create database connection
        let connection = self.create_database_connection().await?;

        // Create session info
        let token_serial = self.auth_provider.serial_number();
        let session_info = SessionInfo::new(
            self.db_config.username.clone(),
            self.db_config.database.clone(),
            token_serial,
        );

        // Store session
        let session_id = session_info.session_id.clone();
        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session_id.clone(), session_info);
        }

        Ok(SecureConnection::new(
            connection,
            session_id,
            self.sessions.clone(),
        ))
    }

    /// Connect with a specific challenge (for testing)
    pub async fn connect_with_challenge(&self, challenge: &[u8], pin: &str) -> VaultResult<SecureConnection> {
        // Perform authentication with provided challenge
        let response = self.auth_provider.challenge_response(challenge, pin)?;
        crate::utils::validate_response(&response, Some(20))?;

        // Create database connection
        let connection = self.create_database_connection().await?;

        // Create session
        let token_serial = self.auth_provider.serial_number();
        let session_info = SessionInfo::new(
            self.db_config.username.clone(),
            self.db_config.database.clone(),
            token_serial,
        );

        let session_id = session_info.session_id.clone();
        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session_id.clone(), session_info);
        }

        Ok(SecureConnection::new(
            connection,
            session_id,
            self.sessions.clone(),
        ))
    }

    /// Get information about the authentication provider
    pub fn auth_info(&self) -> Option<crate::auth::TokenInfo> {
        self.auth_provider.token_info()
    }

    /// Check if hardware token is present and accessible
    pub fn is_token_present(&self) -> bool {
        self.auth_provider.is_present()
    }

    /// Get current session count
    pub async fn session_count(&self) -> usize {
        self.sessions.lock().await.len()
    }

    /// Get session information
    pub async fn get_session_info(&self, session_id: &str) -> Option<SessionInfo> {
        self.sessions.lock().await.get(session_id).cloned()
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.lock().await.values().cloned().collect()
    }

    /// Manually cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.lock().await;
        let now = SystemTime::now();

        sessions.retain(|_id, session| {
            let idle_ok = !session.is_expired(self.config.max_session_idle);
            let lifetime_ok = now
                .duration_since(session.created_at)
                .unwrap_or(Duration::ZERO)
                < self.config.max_session_lifetime;

            idle_ok && lifetime_ok
        });
    }

    fn generate_challenge(&self) -> Vec<u8> {
        crate::utils::generate_challenge(self.config.challenge_length)
    }

    async fn check_session_limits(&self) -> VaultResult<()> {
        let session_count = self.session_count().await;

        if session_count >= self.config.max_concurrent_sessions {
            return Err(VaultError::Session(
                "Maximum concurrent sessions reached".to_string(),
            ));
        }

        Ok(())
    }

    async fn create_database_connection(&self) -> VaultResult<Client> {
        let connection_string = self.build_connection_string();

        // Debug: Print the connection string (remove password for security)
        let debug_string = connection_string.replace(
            &format!(
                "password={}",
                self.db_config
                    .password
                    .as_ref()
                    .unwrap_or(&"NONE".to_string())
            ),
            "password=***",
        );
        println!("🔧 Connection string: {debug_string}");

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

        // Spawn the connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {e}");
            }
        });

        Ok(client)
    }

    fn build_connection_string(&self) -> String {
        let mut parts = vec![
            format!("host={}", self.db_config.host),
            format!("port={}", self.db_config.port),
            format!("user={}", self.db_config.username),
            format!("dbname={}", self.db_config.database),
            format!("connect_timeout={}", self.db_config.connect_timeout),
        ];

        // CRITICAL: Add password if provided
        if let Some(ref password) = self.db_config.password {
            parts.push(format!("password={password}"));
        }

        if let Some(ref app_name) = self.db_config.application_name {
            parts.push(format!("application_name={app_name}"));
        }

        // Add SSL mode
        let ssl_mode = match self.db_config.ssl_mode {
            SslMode::Disable => "disable",
            SslMode::Allow => "allow",
            SslMode::Prefer => "prefer",
            SslMode::Require => "require",
            SslMode::VerifyFull => "verify-full",
        };
        parts.push(format!("sslmode={ssl_mode}"));

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "postgres");
    }

    #[test]
    fn test_session_info_creation() {
        let session = SessionInfo::new(
            "testuser".to_string(),
            "testdb".to_string(),
            Some("123456".to_string()),
        );

        assert_eq!(session.database_user, "testuser");
        assert_eq!(session.database_name, "testdb");
        assert_eq!(session.token_serial, Some("123456".to_string()));
        assert!(!session.session_id.is_empty());
    }

    #[test]
    fn test_vault_config_default() {
        let config = VaultConfig::default();
        assert_eq!(config.max_concurrent_sessions, 10);
        assert_eq!(config.challenge_length, 32);
        assert!(!config.require_token_per_connection);
    }

    #[tokio::test]
    async fn test_vault_creation_with_mock() {
        let db_config = DatabaseConfig::default();
        let vault = Vault::new_with_mock(db_config);

        assert!(vault.is_token_present());
        assert_eq!(vault.session_count().await, 0);
    }
}
