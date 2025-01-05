use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::temux::TemuxClient;

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, TemuxClient>>>,
}

impl SessionManager {
    /// Creates a new SessionManager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Adds a new TemuxClient session and stores its sender
    pub fn add_session(&self, session_name: &str) {
        let client = TemuxClient::new(session_name);

        // Store the sender in the sessions map
        self.sessions
            .lock()
            .unwrap()
            .insert(session_name.to_string(), client);
    }

    /// Sends a command to a specific session by name
    pub async fn send_command(&self, session_name: &str, _command: &str) -> Result<(), String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(_sender) = sessions.get(session_name) {
            // let _ = sender.send_command(command).await;
            // .map_err(|e| format!("Failed to send command: {}", e))?;
            Ok(())
        } else {
            Err(format!("Session '{}' not found", session_name))
        }
    }

    /// Lists all active sessions
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.lock().unwrap().keys().cloned().collect()
    }

    /// Removes a session
    pub fn remove_session(&self, session_name: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if sessions.remove(session_name).is_some() {
            Ok(())
        } else {
            Err(format!("Session '{}' not found", session_name))
        }
    }
}
