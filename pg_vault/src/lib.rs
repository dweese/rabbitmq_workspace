//! pg_vault - Secure PostgreSQL with Hardware Token Authentication
//!
//! A Rust library providing secure PostgreSQL connections with hardware token
//! authentication, specifically designed for Yubikey devices.

pub mod auth;
pub mod vault;

/// Re-exports for convenient importing
pub mod prelude {
    // Authentication types
    pub use crate::auth::{
        AuthConfig,
        AuthError,
        AuthResult,
        AuthProviderFactory,
        MockYubikey,
        TokenInfo,
        YubikeyAuth,
        YubikeyDevice,
    };

    // Vault types
    pub use crate::vault::{
        Connection,
        DatabaseConfig,
        SecureConnection,
        SessionInfo,
        SslMode,
        Vault,
        VaultConfig,
        VaultError,
        VaultResult,
    };

    // Connection types
    pub use crate::vault::connection::{
        ConnectionConfig,
        ConnectionMetrics,
        QueryParams,
        QueryResult,
        QueryType,
        Transaction,
    };
}