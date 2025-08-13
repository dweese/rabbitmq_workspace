//! Secure database connection wrapper

use super::SessionInfo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::Client;

/// Secure database connection wrapper
///
/// This struct holds an active database client and its associated session ID.
/// It ensures that session activity is tracked and that the session is cleaned
/// up when the connection is dropped.
pub struct SecureConnection {
    /// The actual database client
    client: Client,
    /// Session identifier
    session_id: String,
    /// Reference to sessions for activity tracking
    sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionInfo>>>,
}

impl SecureConnection {
    pub(super) fn new(
        client: Client,
        session_id: String,
        sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionInfo>>>,
    ) -> Self {
        Self {
            client,
            session_id,
            sessions,
        }
    }

    /// Get the underlying database client, updating session activity.
    pub fn client(&self) -> &Client {
        // Update session activity in a background task to avoid blocking.
        let session_id = self.session_id.clone();
        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            let mut sessions_guard = sessions.lock().await;
            if let Some(session) = sessions_guard.get_mut(&session_id) {
                session.update_activity();
            }
        });

        &self.client
    }

    /// Get session information for this connection.
    pub async fn session_info(&self) -> Option<SessionInfo> {
        self.sessions.lock().await.get(&self.session_id).cloned()
    }

    /// Get the unique session ID for this connection.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl Drop for SecureConnection {
    fn drop(&mut self) {
        // Clean up session when connection is dropped.
        let session_id = self.session_id.clone();
        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            let mut sessions_guard = sessions.lock().await;
            sessions_guard.remove(&session_id);
        });
    }
}