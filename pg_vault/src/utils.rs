//! Utility functions for the pg_vault crate.

use crate::auth::{AuthError, AuthResult};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a secure random challenge.
///
/// NOTE: For a real implementation, you'd want to use a proper CSPRNG
/// (e.g., from the `rand` crate). This is a simple, deterministic-enough
/// implementation for demonstration and testing.
pub fn generate_challenge(length: usize) -> Vec<u8> {
    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);

    let hash = hasher.finish();
    let mut challenge = Vec::with_capacity(length);

    for i in 0..length {
        challenge.push(((hash >> (i % 8 * 8)) & 0xFF) as u8);
    }

    challenge
}

/// Validate that a response matches expected format/length.
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