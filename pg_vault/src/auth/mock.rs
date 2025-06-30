// pg_vault/src/auth/mock.rs

//! Mock Yubikey implementation for development and testing
//! 
//! Provides a simulated hardware token that behaves like a real Yubikey
//! but doesn't require actual hardware. Useful for development, CI/CD,
//! and unit testing.

use super::{AuthError, AuthResult, TokenInfo, YubikeyAuth};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Mock Yubikey device for development and testing
#[derive(Debug)]
pub struct MockYubikey {
    /// Shared state for the mock device
    state: Arc<Mutex<MockState>>,
}

#[derive(Debug, Clone)]
struct MockState {
    /// Whether the device is "present"
    is_present: bool,
    /// Whether the device requires touch
    requires_touch: bool,
    /// Mock serial number
    serial_number: String,
    /// Mock version
    version: String,
    /// Secret key for generating consistent responses
    secret_key: Vec<u8>,
    /// Simulated failure rate (0.0 = never fail, 1.0 = always fail)
    failure_rate: f32,
    /// Number of authentication attempts
    auth_attempts: u32,
    /// Whether to simulate slow operations
    simulate_slow_ops: bool,
    /// Last challenge received (for testing)
    last_challenge: Option<Vec<u8>>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            is_present: true,
            requires_touch: false,
            serial_number: "MOCK-123456".to_string(),
            version: "5.4.3".to_string(),
            secret_key: b"mock_yubikey_secret_key_2024".to_vec(),
            failure_rate: 0.0,
            auth_attempts: 0,
            simulate_slow_ops: false,
            last_challenge: None,
        }
    }
}

impl MockYubikey {
    /// Create a new mock Yubikey with default settings
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState::default())),
        }
    }
    
    /// Create a mock Yubikey that requires touch interaction
    pub fn new_with_touch() -> Self {
        let mut state = MockState::default();
        state.requires_touch = true;
        
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    /// Create a mock Yubikey that's not present (for testing error cases)
    pub fn new_not_present() -> Self {
        let mut state = MockState::default();
        state.is_present = false;
        
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    /// Create a mock Yubikey with custom serial number
    pub fn new_with_serial(serial: String) -> Self {
        let mut state = MockState::default();
        state.serial_number = serial;
        
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    /// Set the failure rate for testing error conditions
    /// 
    /// # Arguments
    /// * `rate` - Failure rate between 0.0 (never fail) and 1.0 (always fail)
    pub fn set_failure_rate(&self, rate: f32) {
        if let Ok(mut state) = self.state.lock() {
            state.failure_rate = rate.clamp(0.0, 1.0);
        }
    }
    
    /// Enable/disable simulation of slow operations
    pub fn set_simulate_slow_ops(&self, enable: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.simulate_slow_ops = enable;
        }
    }
    
    /// Set whether the device should require touch
    pub fn set_requires_touch(&self, requires: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.requires_touch = requires;
        }
    }
    
    /// Set whether the device is present
    pub fn set_present(&self, present: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.is_present = present;
        }
    }
    
    /// Get the number of authentication attempts made
    pub fn auth_attempt_count(&self) -> u32 {
        self.state.lock().map(|s| s.auth_attempts).unwrap_or(0)
    }
    
    /// Get the last challenge that was sent to the device
    pub fn last_challenge(&self) -> Option<Vec<u8>> {
        self.state.lock().ok()?.last_challenge.clone()
    }
    
    /// Reset the mock device state
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.auth_attempts = 0;
            state.last_challenge = None;
        }
    }
    
    /// Generate a deterministic response based on challenge and secret key
    fn generate_response(&self, challenge: &[u8], secret_key: &[u8]) -> Vec<u8> {
        let mut hasher = DefaultHasher::new();
        
        // Hash the challenge and secret together
        challenge.hash(&mut hasher);
        secret_key.hash(&mut hasher);
        
        let hash = hasher.finish();
        
        // Generate a 20-byte response (similar to HMAC-SHA1)
        let mut response = Vec::with_capacity(20);
        for i in 0..20 {
            let byte = ((hash >> (i % 8 * 8)) & 0xFF) as u8;
            response.push(byte ^ challenge.get(i % challenge.len()).unwrap_or(&0xFF));
        }
        
        response
    }
    
    /// Simulate random failures based on failure rate
    fn should_fail(&self, failure_rate: f32) -> bool {
        if failure_rate <= 0.0 {
            return false;
        }
        if failure_rate >= 1.0 {
            return true;
        }
        
        // Simple pseudo-random based on current time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let random_value = (now % 1000) as f32 / 1000.0;
        random_value < failure_rate
    }
}

impl Default for MockYubikey {
    fn default() -> Self {
        Self::new()
    }
}

impl YubikeyAuth for MockYubikey {
    fn is_present(&self) -> bool {
        self.state.lock().map(|s| s.is_present).unwrap_or(false)
    }
    
    fn requires_touch(&self) -> bool {
        self.state.lock().map(|s| s.requires_touch).unwrap_or(false)
    }
    
    fn serial_number(&self) -> Option<String> {
        self.state.lock().ok().map(|s| s.serial_number.clone())
    }
    
    fn challenge_response(&self, challenge: &[u8]) -> AuthResult<Vec<u8>> {
        let mut state = self.state.lock().map_err(|_| {
            AuthError::CommunicationError("Failed to lock mock state".to_string())
        })?;
        
        // Check if device is present
        if !state.is_present {
            return Err(AuthError::TokenNotFound);
        }
        
        // Validate challenge
        if challenge.is_empty() {
            return Err(AuthError::InvalidResponse);
        }
        
        if challenge.len() > 255 {
            return Err(AuthError::ChallengeFailed("Challenge too long".to_string()));
        }
        
        // Update state
        state.auth_attempts += 1;
        state.last_challenge = Some(challenge.to_vec());
        
        // Simulate slow operations if enabled
        if state.simulate_slow_ops {
            drop(state); // Release lock during sleep
            std::thread::sleep(Duration::from_millis(100));
            state = self.state.lock().map_err(|_| {
                AuthError::CommunicationError("Failed to relock mock state".to_string())
            })?;
        }
        
        // Check for simulated failures
        if self.should_fail(state.failure_rate) {
            return Err(AuthError::ChallengeFailed("Simulated failure".to_string()));
        }
        
        // Simulate touch requirement
        if state.requires_touch && state.auth_attempts % 3 == 1 {
            // Occasionally require user interaction
            return Err(AuthError::UserInteractionRequired);
        }
        
        // Generate response
        let secret_key = state.secret_key.clone();
        let response = self.generate_response(challenge, &secret_key);
        
        Ok(response)
    }
    
    fn token_info(&self) -> Option<TokenInfo> {
        let state = self.state.lock().ok()?;
        
        if !state.is_present {
            return None;
        }
        
        Some(
            TokenInfo::new("Mock Yubikey".to_string())
                .with_serial(state.serial_number.clone())
                .with_version(state.version.clone())
                .with_touch_required(state.requires_touch)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_yubikey_basic() {
        let mock = MockYubikey::new();
        
        assert!(mock.is_present());
        assert!(!mock.requires_touch());
        assert_eq!(mock.serial_number(), Some("MOCK-123456".to_string()));
    }
    
    #[test]
    fn test_challenge_response() {
        let mock = MockYubikey::new();
        let challenge = b"test_challenge";
        
        let response = mock.challenge_response(challenge).unwrap();
        assert_eq!(response.len(), 20);
        
        // Same challenge should produce same response
        let response2 = mock.challenge_response(challenge).unwrap();
        assert_eq!(response, response2);
    }
    
    #[test]
    fn test_different_challenges_different_responses() {
        let mock = MockYubikey::new();
        
        let response1 = mock.challenge_response(b"challenge1").unwrap();
        let response2 = mock.challenge_response(b"challenge2").unwrap();
        
        assert_ne!(response1, response2);
    }
    
    #[test]
    fn test_not_present() {
        let mock = MockYubikey::new_not_present();
        
        assert!(!mock.is_present());
        assert!(mock.token_info().is_none());
        
        let result = mock.challenge_response(b"test");
        assert!(matches!(result, Err(AuthError::TokenNotFound)));
    }
    
    #[test]
    fn test_with_touch() {
        let mock = MockYubikey::new_with_touch();
        
        assert!(mock.requires_touch());
        
        // First attempt might require touch
        let _result = mock.challenge_response(b"test");
        assert!(mock.auth_attempt_count() > 0);
    }
    
    #[test]
    fn test_failure_rate() {
        let mock = MockYubikey::new();
        mock.set_failure_rate(1.0); // Always fail
        
        let result = mock.challenge_response(b"test");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_challenge() {
        let mock = MockYubikey::new();
        
        let result = mock.challenge_response(&[]);
        assert!(matches!(result, Err(AuthError::InvalidResponse)));
    }
    
    #[test]
    fn test_custom_serial() {
        let serial = "CUSTOM-789012".to_string();
        let mock = MockYubikey::new_with_serial(serial.clone());
        
        assert_eq!(mock.serial_number(), Some(serial));
    }
    
    #[test]
    fn test_reset() {
        let mock = MockYubikey::new();
        
        mock.challenge_response(b"test").unwrap();
        assert!(mock.auth_attempt_count() > 0);
        assert!(mock.last_challenge().is_some());
        
        mock.reset();
        assert_eq!(mock.auth_attempt_count(), 0);
        assert!(mock.last_challenge().is_none());
    }
}