// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::confirm::ConfirmationConfig;

pub const DEFAULT_PORT: u16 = 21027;
pub const DEFAULT_RATE_LIMIT: u32 = 30;
pub const BIND_ADDRESS: &str = "127.0.0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NinjaExecConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
    #[serde(default)]
    pub confirmation: ConfirmationConfig,
    #[serde(default)]
    pub confirm_token: Option<String>,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_rate_limit() -> u32 {
    DEFAULT_RATE_LIMIT
}

impl Default for NinjaExecConfig {
    fn default() -> Self {
        NinjaExecConfig {
            port: DEFAULT_PORT,
            rate_limit_per_minute: DEFAULT_RATE_LIMIT,
            confirmation: ConfirmationConfig::default(),
            confirm_token: None,
        }
    }
}

impl NinjaExecConfig {
    pub fn load(data_dir: &PathBuf) -> Self {
        let config_path = data_dir.join("ninja-exec.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        NinjaExecConfig::default()
    }

    pub fn save_default(data_dir: &PathBuf) {
        let config_path = data_dir.join("ninja-exec.json");
        if config_path.exists() {
            return;
        }
        let _ = std::fs::create_dir_all(data_dir);
        let config = NinjaExecConfig::default();
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::write(&config_path, json);
        }
    }

    pub fn generate_confirm_token(data_dir: &PathBuf) -> String {
        let config_path = data_dir.join("ninja-exec.json");
        let mut config = if config_path.exists() {
            std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            NinjaExecConfig::default()
        };

        if config.confirm_token.is_none() {
            let mut token_bytes = [0u8; 32];
            getrandom::getrandom(&mut token_bytes).expect("Failed to generate confirm token");
            use base64::Engine;
            let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&token_bytes);
            config.confirm_token = Some(token);
            let _ = std::fs::create_dir_all(data_dir);
            if let Ok(json) = serde_json::to_string_pretty(&config) {
                let _ = std::fs::write(&config_path, json);
            }
        }

        config.confirm_token.unwrap()
    }
}
