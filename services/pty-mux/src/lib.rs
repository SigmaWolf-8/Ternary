// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub cols: u16,
    pub rows: u16,
    pub shell: String,
    pub cwd: String,
    pub env: HashMap<String, String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            cwd: std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
            env: std::env::vars().collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: SessionId,
    pub cols: u16,
    pub rows: u16,
    pub created_at: u64,
    pub last_activity: u64,
    pub pid: u32,
}

pub struct PtyMuxService {
    next_id: u64,
    sessions: HashMap<SessionId, SessionInfo>,
    max_sessions: usize,
}

impl PtyMuxService {
    pub fn new(max_sessions: usize) -> Self {
        Self {
            next_id: 1,
            sessions: HashMap::new(),
            max_sessions,
        }
    }

    pub fn create_session(&mut self, config: SessionConfig) -> Result<SessionId, String> {
        if self.sessions.len() >= self.max_sessions {
            return Err("Maximum session limit reached".to_string());
        }

        let id = SessionId(self.next_id);
        self.next_id += 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let info = SessionInfo {
            id,
            cols: config.cols,
            rows: config.rows,
            created_at: now,
            last_activity: now,
            pid: 0,
        };

        self.sessions.insert(id, info);
        Ok(id)
    }

    pub fn destroy_session(&mut self, id: SessionId) -> bool {
        self.sessions.remove(&id).is_some()
    }

    pub fn get_session(&self, id: SessionId) -> Option<&SessionInfo> {
        self.sessions.get(&id)
    }

    pub fn list_sessions(&self) -> Vec<&SessionInfo> {
        self.sessions.values().collect()
    }

    pub fn resize_session(&mut self, id: SessionId, cols: u16, rows: u16) -> bool {
        if let Some(session) = self.sessions.get_mut(&id) {
            session.cols = cols;
            session.rows = rows;
            true
        } else {
            false
        }
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

pub type SharedPtyMux = Arc<Mutex<PtyMuxService>>;

pub fn new_shared_mux(max_sessions: usize) -> SharedPtyMux {
    Arc::new(Mutex::new(PtyMuxService::new(max_sessions)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut mux = PtyMuxService::new(10);
        let id = mux.create_session(SessionConfig::default()).unwrap();
        assert_eq!(id.0, 1);
        assert_eq!(mux.session_count(), 1);
    }

    #[test]
    fn test_destroy_session() {
        let mut mux = PtyMuxService::new(10);
        let id = mux.create_session(SessionConfig::default()).unwrap();
        assert!(mux.destroy_session(id));
        assert_eq!(mux.session_count(), 0);
        assert!(!mux.destroy_session(id));
    }

    #[test]
    fn test_max_sessions() {
        let mut mux = PtyMuxService::new(2);
        let _ = mux.create_session(SessionConfig::default()).unwrap();
        let _ = mux.create_session(SessionConfig::default()).unwrap();
        let result = mux.create_session(SessionConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_resize_session() {
        let mut mux = PtyMuxService::new(10);
        let id = mux.create_session(SessionConfig::default()).unwrap();
        assert!(mux.resize_session(id, 120, 40));
        let info = mux.get_session(id).unwrap();
        assert_eq!(info.cols, 120);
        assert_eq!(info.rows, 40);
    }

    #[test]
    fn test_list_sessions() {
        let mut mux = PtyMuxService::new(10);
        let _ = mux.create_session(SessionConfig::default()).unwrap();
        let _ = mux.create_session(SessionConfig::default()).unwrap();
        let list = mux.list_sessions();
        assert_eq!(list.len(), 2);
    }
}
