//! A secure vault for PostgreSQL connections using hardware token authentication.

pub mod auth;
pub mod utils;
pub mod vault;

pub mod prelude {
    //! A "prelude" for users of the `pg_vault` library.
    pub use crate::auth::{AuthConfig, AuthError, AuthResult, TokenInfo, YubikeyAuth};
    pub use crate::vault::{
        DatabaseConfig, SecureConnection, SessionInfo, Vault, VaultConfig, VaultError, VaultResult,
    };
}