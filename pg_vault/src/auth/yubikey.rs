// pg_vault/src/auth/yubikey.rs
//! Real Yubikey hardware token implementation
//! This module provides integration with actual Yubikey hardware tokens
//! using the yubikey crate for PIV challenge-response authentication.

use super::{AuthConfig, AuthError, AuthResult, TokenInfo, YubikeyAuth};

#[cfg(feature = "hardware-yubikey")]
use yubikey::piv::{Key, SlotId};

/// Real Yubikey device implementation
pub struct YubikeyDevice {
    #[cfg(feature = "hardware-yubikey")]
    config: AuthConfig,
    #[cfg(not(feature = "hardware-yubikey"))]
    _phantom: std::marker::PhantomData<()>,
}

impl YubikeyDevice {
    /// Create a new Yubikey device instance
    pub fn new(config: AuthConfig) -> AuthResult<Self> {
        #[cfg(feature = "hardware-yubikey")]
        {
            // The `yubikey` crate handles device discovery, so we just store the config.
            // We will attempt to open a connection when an operation is requested.
            Ok(Self { config })
        }

        #[cfg(not(feature = "hardware-yubikey"))]
        {
            let _ = config; // Suppress unused variable warning
            Err(AuthError::ConfigurationError(
                "Yubikey support not enabled. Compile with --features hardware-yubikey".to_string(),
            ))
        }
    }

}

impl YubikeyAuth for YubikeyDevice {
    fn is_present(&self) -> bool {
        #[cfg(feature = "hardware-yubikey")]
        {
            // Attempt to open a connection to any YubiKey.
            yubikey::YubiKey::open().is_ok()
        }

        #[cfg(not(feature = "hardware-yubikey"))]
        {
            false
        }
    }

    fn serial_number(&self) -> Option<String> {
        #[cfg(feature = "hardware-yubikey")]
        {
            yubikey::YubiKey::open()
                .ok()
                .and_then(|mut yk| yk.serial().map(|s| s.to_string()))
        }

        #[cfg(not(feature = "hardware-yubikey"))]
        {
            None
        }
    }

    /// Performs a cryptographic challenge-response using the PIV application.
    /// NOTE: The `YubikeyAuth` trait should be updated to include the `pin` parameter.
    fn challenge_response(&self, challenge: &[u8], pin: &str) -> AuthResult<Vec<u8>> {
        #[cfg(feature = "hardware-yubikey")]
        {
            let mut yk = yubikey::YubiKey::open().map_err(|e| AuthError::TokenError(e.to_string()))?;

            // Verify the user's PIN. This is a required step for signing.
            yk.verify_pin(pin.as_bytes())
                .map_err(|e| AuthError::AuthenticationFailed(e.to_string()))?;

            // Use the key in the "Authentication" slot (9a), which is standard for this.
            let key = Key::read(&mut yk, SlotId::Authentication)
                .map_err(|e| AuthError::TokenError(e.to_string()))?;

            // Sign the challenge. This will require the user to touch the YubiKey if configured.
            let signature = key
                .sign(&mut yk, challenge)
                .map_err(|e| AuthError::TokenError(e.to_string()))?;

            Ok(signature.to_vec())
        }

        #[cfg(not(feature = "hardware-yubikey"))]
        {
            let _ = challenge; // Suppress unused parameter warning
            let _ = pin;
            Err(AuthError::TokenNotFound)
        }
    }

    fn token_info(&self) -> Option<TokenInfo> {
        #[cfg(feature = "hardware-yubikey")]
        {
            yubikey::YubiKey::open().ok().map(|mut yk| {
                let serial = yk.serial().map(|s| s.to_string());
                let version = yk.version().to_string();
                // The `yubikey` crate doesn't directly expose the model name in a simple way,
                // but we can get the version which is very informative.
                // Touch policy is configured per-key, so it's part of the signing operation.
                TokenInfo {
                    serial,
                    version: Some(version),
                    // This is determined by the key's policy, not a device-wide setting.
                    // We can assume true for a PIV auth key.
                    touch_required: true,
                    model: yk.name().to_string(),
                }
            })
        }

        #[cfg(not(feature = "hardware-yubikey"))]
        {
            None
        }
    }
}
