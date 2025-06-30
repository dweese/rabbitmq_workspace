// pg_vault/src/auth/mod.rs

//! Authentication module for pg_vault
//! 
//! Provides hardware token-based authentication for secure PostgreSQL connections.
//! Currently supports Yubikey devices with challenge-response authentication.

use std::error::Error;
use std::fmt;

pub mod mock;
pub mod yubikey;

// Re-export for convenience
pub use mock::MockYubikey;
pub use yubikey::YubikeyDevice;

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication errors
#[derive(Debug, Clone)]
pub enum AuthError {
    /// Hardware token not found or not accessible
    TokenNotFound,
    /// Authentication challenge failed
    ChallengeFailed(String),
    /// Hardware token requires user interaction (touch, etc.)
    UserInteractionRequired,
    /// Invalid response from hardware token
    InvalidResponse,
    /// Hardware token communication error
    CommunicationError(String),
    /// Configuration error
    ConfigurationError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::TokenNotFound => write!(f, "Hardware token not found or not accessible"),
            AuthError::ChallengeFailed(msg) => write!(f, "Authentication challenge failed: {}", msg),
            AuthError::UserInteractionRequired => write!(f, "Hardware token requires user interaction"),
            AuthError::InvalidResponse => write!(f, "Invalid response from hardware token"),
            AuthError::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
            AuthError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for AuthError {}

/// Trait for hardware token authentication providers
pub trait YubikeyAuth: Send + Sync {
    /// Check if a hardware token is present and accessible
    fn is_present(&self) -> bool;
    
    /// Check if the token requires user interaction (e.g., touch)
    fn requires_touch(&self) -> bool;
    
    /// Get the serial number of the token, if available
    fn serial_number(&self) -> Option<String>;
    
    /// Perform challenge-response authentication
    /// 
    /// # Arguments
    /// * `challenge` - The challenge bytes to send to the token
    /// 
    /// # Returns
    /// * `Ok(Vec<u8>)` - The response from the token
    /// * `Err(AuthError)` - Authentication failed
    fn challenge_response(&self, challenge: &[u8]) -> AuthResult<Vec<u8>>;
    
    /// Get token information for logging/debugging
    fn token_info(&self) -> Option<TokenInfo>;
}

/// Information about a hardware token
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub serial: Option<String>,
    pub version: Option<String>,
    pub touch_required: bool,
    pub model: String,
}

impl TokenInfo {
    pub fn new(model: String) -> Self {
        Self {
            serial: None,
            version: None,
            touch_required: false,
            model,
        }
    }
    
    pub fn with_serial(mut self, serial: String) -> Self {
        self.serial = Some(serial);
        self
    }
    
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
    
    pub fn with_touch_required(mut self, required: bool) -> Self {
        self.touch_required = required;
        self
    }
}

/// Configuration for authentication providers
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Timeout for hardware token operations (in seconds)
    pub timeout_seconds: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Whether to require user interaction for each authentication
    pub require_touch: Option<bool>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            max_retries: 3,
            require_touch: None, // Use device default
        }
    }
}

/// Factory for creating authentication providers
pub struct AuthProviderFactory;

impl AuthProviderFactory {
    /// Create a new authentication provider
    /// 
    /// This will attempt to detect and use a real Yubikey device.
    /// If no device is found, it will return an error.
    pub fn create_provider(config: AuthConfig) -> AuthResult<Box<dyn YubikeyAuth>> {
        // Try to create a real Yubikey device first
        match YubikeyDevice::new(config.clone()) {
            Ok(device) => Ok(Box::new(device)),
            Err(_) => Err(AuthError::TokenNotFound),
        }
    }
    
    /// Create a mock authentication provider for development/testing
    pub fn create_mock_provider() -> Box<dyn YubikeyAuth> {
        Box::new(MockYubikey::new())
    }
    
    /// Create a provider, falling back to mock if no real device is available
    pub fn create_provider_with_fallback(config: AuthConfig) -> Box<dyn YubikeyAuth> {
        match Self::create_provider(config) {
            Ok(provider) => provider,
            Err(_) => {
                eprintln!("Warning: No hardware token found, using mock provider");
                Self::create_mock_provider()
            }
        }
    }
}

/// Utility functions for authentication
pub mod utils {
    use super::*;
    
    /// Generate a secure random challenge
    pub fn generate_challenge(length: usize) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // For a real implementation, you'd want to use a proper CSPRNG
        // This is a simple implementation for demonstration
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        
        let hash = hasher.finish();
        let mut challenge = Vec::new();
        
        for i in 0..length {
            challenge.push(((hash >> (i % 64)) & 0xFF) as u8);
        }
        
        challenge
    }
    
    /// Validate that a response matches expected format/length
    pub fn validate_response(response: &[u8], expected_length: Option<usize>) -> AuthResult<()> {
        if response.is_empty() {
            return Err(AuthError::InvalidResponse);
        }
        
        if let Some(expected) = expected_length {
            if response.len() != expected {
                return Err(AuthError::InvalidResponse);
            }
        }
        
        Ok(())
    }
}