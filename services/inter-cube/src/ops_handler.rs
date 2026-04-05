// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Daemon Remote Operations Channel
//!
//! Handles operations messages received via the WebSocket relay:
//! - `exec`: Remote PowerShell script execution with sandbox constraints
//! - `tail`: Live log tailing with follow mode
//! - `telemetry`: System telemetry heartbeat (CPU, RAM, disk, GPU)
//! - `file-push` / `file-pull`: Small file transfer (< 5 MB)
//! - `chunk-init` / `chunk-data` / `chunk-complete`: Chunked transfer for large files
//! - `model-swap`: GGUF model hot-swap on running llama-server
//!
//! ## Security Model
//!
//! Every authenticated operation requires a TL-DSA signature from a registered
//! operator. The operator registry maps key fingerprints to permission scopes
//! (full, exec-only, read-only). Unsigned or out-of-scope operations are
//! rejected with structured error codes and logged as security events.
//!
//! ## Audit
//!
//! Every operation is appended to `ops-audit.jsonl` with timestamp, operator
//! identity, payload hash, result, and exit code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use ternary_math::tl_dsa;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsConfig {
    pub ops_enabled: bool,
    pub operators: Vec<OperatorEntry>,
    pub exec_timeout_seconds: u64,
    pub file_size_limit_bytes: u64,
    pub whitelisted_directories: Vec<String>,
    pub blocked_extensions: Vec<String>,
    pub chunk_size_bytes: u64,
    pub telemetry_interval_seconds: u64,
    pub audit_log_path: String,
    pub audit_log_max_size_mb: u64,
    #[serde(default)]
    pub service_account_name: Option<String>,
}

impl Default for OpsConfig {
    fn default() -> Self {
        OpsConfig {
            ops_enabled: false,
            operators: Vec::new(),
            exec_timeout_seconds: 120,
            file_size_limit_bytes: 5 * 1024 * 1024,
            whitelisted_directories: vec![
                ".plenumnet/ops/".to_string(),
                ".plenumnet/logs/".to_string(),
                ".plenumnet/configs/".to_string(),
                ".plenumnet/transfers/".to_string(),
                ".plenumnet/models/".to_string(),
            ],
            blocked_extensions: vec![
                ".exe".into(), ".dll".into(), ".sys".into(), ".bat".into(),
                ".cmd".into(), ".com".into(), ".scr".into(), ".vbs".into(),
                ".vbe".into(), ".js".into(), ".jse".into(), ".wsf".into(),
                ".wsh".into(), ".msi".into(),
            ],
            chunk_size_bytes: 512 * 1024,
            telemetry_interval_seconds: 60,
            audit_log_path: ".plenumnet/ops-audit.jsonl".to_string(),
            audit_log_max_size_mb: 50,
            service_account_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorEntry {
    pub name: String,
    pub key_fingerprint: String,
    pub public_key: String,
    pub scope: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsAuditEntry {
    pub timestamp: String,
    pub operation: String,
    pub operator_name: String,
    pub operator_fingerprint: String,
    pub node_id: String,
    pub request_id: String,
    pub payload_hash: String,
    pub script_text: Option<String>,
    pub exit_code: Option<i32>,
    pub stdout_truncated: Option<String>,
    pub stderr_truncated: Option<String>,
    pub duration_ms: Option<u64>,
    pub file_path: Option<String>,
    pub file_size: Option<u64>,
    pub result: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub cpu_pct: f64,
    pub ram_pct: f64,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub disk_pct: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub gpu_pct: Option<f64>,
    pub gpu_name: Option<String>,
    pub gpu_vram_used_mb: Option<u64>,
    pub gpu_vram_total_mb: Option<u64>,
    pub process_uptime_seconds: u64,
    pub active_model: Option<String>,
    pub llm_engine_status: String,
    pub os_version: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkTransferManifest {
    pub transfer_id: String,
    pub file_path: String,
    pub total_size_bytes: u64,
    pub chunk_count: u32,
    pub chunk_size_bytes: u64,
    pub full_hash: String,
    pub received_chunks: Vec<bool>,
    pub started_at: String,
    pub last_activity: String,
    pub temp_dir: String,
    #[serde(default)]
    pub operator_fingerprint: String,
}

pub type OpsMsgSender = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<serde_json::Value>>>>;

pub struct OpsHandler {
    config: Arc<Mutex<OpsConfig>>,
    operators: Arc<Mutex<HashMap<String, OperatorEntry>>>,
    exec_mutex: Arc<Mutex<()>>,
    active_tails: Arc<Mutex<HashMap<String, tokio::sync::watch::Sender<bool>>>>,
    active_transfers: Arc<Mutex<HashMap<String, ChunkTransferManifest>>>,
    process_start: Instant,
    node_id: String,
    base_dir: PathBuf,
    ws_sender: OpsMsgSender,
}

impl OpsHandler {
    pub fn new(node_id: String, base_dir: PathBuf) -> Self {
        let config = OpsConfig::default();
        let mut operators = HashMap::new();
        for op in &config.operators {
            operators.insert(op.key_fingerprint.clone(), op.clone());
        }

        OpsHandler {
            config: Arc::new(Mutex::new(config)),
            operators: Arc::new(Mutex::new(operators)),
            exec_mutex: Arc::new(Mutex::new(())),
            active_tails: Arc::new(Mutex::new(HashMap::new())),
            active_transfers: Arc::new(Mutex::new(HashMap::new())),
            process_start: Instant::now(),
            node_id,
            base_dir,
            ws_sender: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_ws_sender(&self, sender: tokio::sync::mpsc::UnboundedSender<serde_json::Value>) {
        *self.ws_sender.lock().await = Some(sender);
    }

    pub async fn load_config(&self, config_path: &Path) {
        if let Ok(content) = tokio::fs::read_to_string(config_path).await {
            if let Ok(config) = serde_json::from_str::<OpsConfig>(&content) {
                let mut ops = self.operators.lock().await;
                ops.clear();
                for op in &config.operators {
                    ops.insert(op.key_fingerprint.clone(), op.clone());
                }
                *self.config.lock().await = config;
                println!("[ops] Config loaded from {}", config_path.display());
            }
        }
    }

    pub async fn is_enabled(&self) -> bool {
        self.config.lock().await.ops_enabled
    }

    pub async fn set_ops_enabled(&self, enabled: bool) {
        self.config.lock().await.ops_enabled = enabled;
    }

    pub async fn add_operator(&self, fingerprint: String, name: String, public_key: String, scope: String) {
        let entry = OperatorEntry {
            name,
            key_fingerprint: fingerprint.clone(),
            public_key,
            scope,
            registered_at: chrono::Utc::now().to_rfc3339(),
        };
        self.operators.lock().await.insert(fingerprint, entry);
    }

    pub async fn remove_operator(&self, fingerprint: &str) {
        self.operators.lock().await.remove(fingerprint);
    }

    pub async fn get_operators(&self) -> HashMap<String, OperatorEntry> {
        self.operators.lock().await.clone()
    }

    async fn audit_error(&self, msg: &serde_json::Value, operation: &str, request_id: &str, error_code: &str, error_message: &str) {
        let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("");
        self.write_audit_entry(&OpsAuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: operation.to_string(),
            operator_name: self.resolve_operator_name(fp).await,
            operator_fingerprint: fp.to_string(),
            node_id: self.node_id.clone(),
            request_id: request_id.to_string(),
            payload_hash: "".to_string(),
            script_text: None, exit_code: None, stdout_truncated: None,
            stderr_truncated: None, duration_ms: None,
            file_path: None, file_size: None,
            result: "failed".to_string(),
            error_code: Some(error_code.to_string()),
            error_message: Some(error_message.to_string()),
        }).await;
    }

    pub async fn resolve_operator_name(&self, fingerprint: &str) -> String {
        let operators = self.operators.lock().await;
        operators.get(fingerprint).map(|op| op.name.clone()).unwrap_or_else(|| fingerprint.to_string())
    }

    pub async fn validate_operator(&self, fingerprint: &str, operation: &str) -> Result<OperatorEntry, (String, String)> {
        let operators = self.operators.lock().await;
        let operator = operators.get(fingerprint)
            .ok_or_else(|| ("SIGNATURE_INVALID".to_string(), format!("Unknown operator fingerprint: {}", fingerprint)))?;

        let scope = operator.scope.as_str();
        let allowed = match scope {
            "full" => true,
            "exec-only" => matches!(operation, "exec" | "tail" | "tail-stop"),
            "read-only" => matches!(operation, "tail" | "tail-stop" | "file-pull"),
            _ => false,
        };

        if !allowed {
            return Err(("SCOPE_VIOLATION".to_string(),
                format!("Operator {} (scope: {}) not authorized for {}", operator.name, scope, operation)));
        }

        Ok(operator.clone())
    }

    fn verify_tl_dsa_signature(public_key_hex: &str, message: &[u8], signature_hex: &str) -> bool {
        let pk_bytes = match hex::decode(public_key_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let sig_bytes = match hex::decode(signature_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };
        tl_dsa::verify(&pk_bytes, message, &sig_bytes, tl_dsa::TlDsaVariant::TlDsa87)
    }

    fn build_signing_payload(msg: &serde_json::Value) -> Vec<u8> {
        let mut canonical = std::collections::BTreeMap::new();
        if let Some(obj) = msg.as_object() {
            for (k, v) in obj {
                if k != "signature" && k != "operator_fingerprint" {
                    canonical.insert(k.clone(), v.clone());
                }
            }
        }
        serde_json::to_vec(&canonical).unwrap_or_default()
    }

    fn is_path_whitelisted(&self, file_path: &str, whitelist: &[String]) -> bool {
        let normalized = file_path.replace('\\', "/");
        if normalized.contains("..") || normalized.starts_with('/') || normalized.contains("://") {
            return false;
        }
        if !whitelist.iter().any(|dir| {
            let dir_normalized = dir.replace('\\', "/");
            normalized.starts_with(&dir_normalized)
        }) {
            return false;
        }
        let full_path = self.base_dir.join(&normalized);
        if let Some(parent) = full_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let canonical = match full_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                if let Some(parent) = full_path.parent() {
                    match parent.canonicalize() {
                        Ok(p) => p.join(full_path.file_name().unwrap_or_default()),
                        Err(_) => return false,
                    }
                } else {
                    return false;
                }
            }
        };
        let canonical_str = canonical.to_string_lossy().replace('\\', "/");
        let base_str = self.base_dir.to_string_lossy().replace('\\', "/");
        canonical_str.starts_with(&base_str)
    }

    fn is_extension_blocked(&self, file_path: &str, blocked: &[String]) -> bool {
        let lower = file_path.to_lowercase();
        blocked.iter().any(|ext| lower.ends_with(ext))
    }

    fn truncate_output(output: &str, max_bytes: usize) -> String {
        if output.len() <= max_bytes {
            output.to_string()
        } else {
            let truncated = &output[..max_bytes];
            format!("{}... [truncated at {} bytes]", truncated, max_bytes)
        }
    }

    pub async fn handle_exec(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let script = msg.get("script").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let timeout_secs = msg.get("timeout_seconds").and_then(|v| v.as_u64())
            .unwrap_or(self.config.lock().await.exec_timeout_seconds);

        let _guard = self.exec_mutex.lock().await;

        let start = Instant::now();

        let ops_dir = self.base_dir.join(".plenumnet/ops");
        let _ = tokio::fs::create_dir_all(&ops_dir).await;

        #[allow(unused_variables)]
        let task_name = format!("PlenumNET-Ops-{}", request_id);

        #[cfg(target_os = "windows")]
        let result = {
            let config = self.config.lock().await;
            let service_account = config.service_account_name.clone()
                .unwrap_or_else(|| "NT SERVICE\\PlenumNET-Ops".to_string());
            drop(config);

            let sandbox_preamble = [
                "$ErrorActionPreference = 'Stop'",
                "$ExecutionContext.SessionState.LanguageMode = 'ConstrainedLanguage'",
                "Set-StrictMode -Version Latest",
            ].join("\n");

            let script_path = ops_dir.join(format!("ops-exec-{}.ps1", request_id));
            let stdout_path = ops_dir.join(format!("ops-stdout-{}.txt", request_id));
            let stderr_path = ops_dir.join(format!("ops-stderr-{}.txt", request_id));
            let exitcode_path = ops_dir.join(format!("ops-exit-{}.txt", request_id));
            let wrapped_script = format!(
                "{}\n$global:LASTEXITCODE = 0\ntry {{\n{}\n  if (-not $LASTEXITCODE) {{ $global:LASTEXITCODE = 0 }}\n}} catch {{\n  $_ | Out-File -FilePath '{}' -Append -Encoding UTF8\n  $global:LASTEXITCODE = 1\n}}\n$LASTEXITCODE | Out-File -FilePath '{}' -Encoding ASCII -NoNewline",
                sandbox_preamble, script, stderr_path.display(), exitcode_path.display()
            );
            if let Err(e) = tokio::fs::write(&script_path, &wrapped_script).await {
                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "exec".to_string(),
                    operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); self.resolve_operator_name(fp).await },
                    operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    node_id: self.node_id.clone(), request_id: request_id.clone(),
                    payload_hash: Self::hash_payload(&script), script_text: Some(script.clone()),
                    exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
                    file_path: None, file_size: None, result: "failed".to_string(),
                    error_code: Some("EXEC_FAILED".to_string()),
                    error_message: Some(format!("Failed to write temp script: {}", e)),
                }).await;
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "EXEC_FAILED", "message": format!("Failed to write temp script: {}", e),
                });
            }
            let _ = tokio::process::Command::new("cmd").args(["/c", "icacls", &ops_dir.to_string_lossy(), "/deny", "Everyone:(OI)(CI)(DE,DC)"]).output().await;
            let create_result = tokio::process::Command::new("schtasks.exe")
                .args([
                    "/Create", "/F", "/TN", &task_name,
                    "/SC", "ONCE", "/ST", "00:00",
                    "/RU", &service_account,
                    "/TR", &format!(
                        "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File '{}' > '{}' 2>> '{}'",
                        script_path.display(), stdout_path.display(), stderr_path.display()
                    ),
                ])
                .output().await;
            if let Err(e) = create_result {
                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "exec".to_string(),
                    operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); self.resolve_operator_name(fp).await },
                    operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    node_id: self.node_id.clone(), request_id: request_id.clone(),
                    payload_hash: Self::hash_payload(&script), script_text: Some(script.clone()),
                    exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
                    file_path: None, file_size: None, result: "failed".to_string(),
                    error_code: Some("EXEC_FAILED".to_string()),
                    error_message: Some(format!("Failed to create scheduled task: {}", e)),
                }).await;
                let _ = tokio::fs::remove_file(&script_path).await;
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "EXEC_FAILED",
                    "message": format!("Failed to create scheduled task for service-account execution: {}. Ensure the PlenumNET-Ops service account exists.", e),
                });
            }
            let _ = tokio::process::Command::new("schtasks.exe")
                .args(["/Run", "/TN", &task_name])
                .output().await;
            let mut cmd = tokio::process::Command::new("powershell.exe");
            cmd.args([
                "-NoProfile", "-NonInteractive",
                "-Command",
                &format!(
                    "do {{ Start-Sleep -Seconds 1; $s = (schtasks /Query /TN '{}' /FO CSV /NH | ConvertFrom-Csv).'Status' }} while ($s -eq 'Running'); \
                     schtasks /Delete /TN '{}' /F; \
                     if (Test-Path '{}') {{ Get-Content '{}' }} else {{ '' }}; \
                     if (Test-Path '{}') {{ [Console]::Error.Write((Get-Content '{}' -Raw)) }}",
                    task_name, task_name,
                    stdout_path.display(), stdout_path.display(),
                    stderr_path.display(), stderr_path.display(),
                ),
            ]);
            cmd.env_clear();
            cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
            cmd.env("SYSTEMROOT", std::env::var("SYSTEMROOT").unwrap_or_else(|_| "C:\\Windows".into()));
            cmd.env("TEMP", ops_dir.to_string_lossy().to_string());
            cmd.env("TMP", ops_dir.to_string_lossy().to_string());
            cmd.env("PLENUMNET_OPS_EXEC", "1");
            cmd.env("PLENUMNET_REQUEST_ID", &request_id);
            cmd.current_dir(&ops_dir);

            tokio::time::timeout(Duration::from_secs(timeout_secs), cmd.output()).await
                .map_err(|_| ())
        };

        #[cfg(not(target_os = "windows"))]
        let result: Result<Result<std::process::Output, std::io::Error>, ()> = {
            let mut cmd = tokio::process::Command::new("sh");
            cmd.arg("-c").arg(&script);
            cmd.env_clear();
            cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
            cmd.env("TEMP", "/tmp");
            cmd.env("TMP", "/tmp");
            cmd.env("HOME", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()));
            cmd.env("PLENUMNET_OPS_EXEC", "1");
            cmd.env("PLENUMNET_REQUEST_ID", &request_id);
            cmd.current_dir(&ops_dir);

            let child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    self.write_audit_entry(&OpsAuditEntry {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        operation: "exec".to_string(),
                        operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); self.resolve_operator_name(fp).await },
                        operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        node_id: self.node_id.clone(), request_id: request_id.clone(),
                        payload_hash: Self::hash_payload(&script), script_text: Some(script.clone()),
                        exit_code: None, stdout_truncated: None, stderr_truncated: None,
                        duration_ms: Some(start.elapsed().as_millis() as u64),
                        file_path: None, file_size: None, result: "failed".to_string(),
                        error_code: Some("EXEC_FAILED".to_string()),
                        error_message: Some(format!("Failed to spawn process: {}", e)),
                    }).await;
                    return serde_json::json!({
                        "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                        "error_code": "EXEC_FAILED",
                        "message": format!("Failed to spawn process: {}", e),
                    });
                }
            };

            match tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(r) => Ok(r),
                Err(_) => {
                    Err(())
                }
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let operator_fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("");
        let resolved_name = self.resolve_operator_name(operator_fp).await;

        let response = match result {
            Ok(Ok(output)) => {
                let stdout = Self::truncate_output(
                    &String::from_utf8_lossy(&output.stdout), 10 * 1024);
                let stderr = Self::truncate_output(
                    &String::from_utf8_lossy(&output.stderr), 10 * 1024);
                #[cfg(target_os = "windows")]
                let exit_code = {
                    let ec_path = ops_dir.join(format!("ops-exit-{}.txt", request_id));
                    tokio::fs::read_to_string(&ec_path).await
                        .ok()
                        .and_then(|s| s.trim().parse::<i32>().ok())
                        .unwrap_or_else(|| output.status.code().unwrap_or(-1))
                };
                #[cfg(not(target_os = "windows"))]
                let exit_code = output.status.code().unwrap_or(-1);

                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "exec".to_string(),
                    operator_name: resolved_name.clone(),
                    operator_fingerprint: operator_fp.to_string(),
                    node_id: self.node_id.clone(),
                    request_id: request_id.clone(),
                    payload_hash: Self::hash_payload(&script),
                    script_text: Some(script),
                    exit_code: Some(exit_code),
                    stdout_truncated: Some(stdout.clone()),
                    stderr_truncated: Some(stderr.clone()),
                    duration_ms: Some(duration_ms),
                    file_path: None,
                    file_size: None,
                    result: "success".to_string(),
                    error_code: None,
                    error_message: None,
                }).await;

                serde_json::json!({
                    "type": "exec-result",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "exit_code": exit_code,
                    "stdout": stdout,
                    "stderr": stderr,
                    "duration_ms": duration_ms,
                    "timed_out": false,
                })
            }
            Ok(Err(e)) => {
                serde_json::json!({
                    "type": "ops-error",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "error_code": "EXEC_FAILED",
                    "message": format!("Execution failed: {}", e),
                })
            }
            Err(_) => {
                #[cfg(target_os = "windows")]
                {
                    let _ = tokio::process::Command::new("schtasks")
                        .args(&["/End", "/TN", &task_name])
                        .output().await;
                    let _ = tokio::process::Command::new("schtasks")
                        .args(&["/Delete", "/TN", &task_name, "/F"])
                        .output().await;
                }

                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "exec".to_string(),
                    operator_name: resolved_name.clone(),
                    operator_fingerprint: operator_fp.to_string(),
                    node_id: self.node_id.clone(),
                    request_id: request_id.clone(),
                    payload_hash: Self::hash_payload(&script),
                    script_text: Some(script),
                    exit_code: None,
                    stdout_truncated: None,
                    stderr_truncated: None,
                    duration_ms: Some(duration_ms),
                    file_path: None,
                    file_size: None,
                    result: "timeout".to_string(),
                    error_code: Some("EXEC_TIMEOUT".to_string()),
                    error_message: Some(format!("Script exceeded {}s timeout — process terminated", timeout_secs)),
                }).await;

                serde_json::json!({
                    "type": "exec-result",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "exit_code": -1,
                    "stdout": "",
                    "stderr": format!("Script exceeded {}s timeout — process terminated", timeout_secs),
                    "duration_ms": duration_ms,
                    "timed_out": true,
                })
            }
        };

        #[cfg(target_os = "windows")]
        {
            let cleanup_paths = [
                ops_dir.join(format!("ops-exec-{}.ps1", request_id)),
                ops_dir.join(format!("ops-stdout-{}.txt", request_id)),
                ops_dir.join(format!("ops-stderr-{}.txt", request_id)),
                ops_dir.join(format!("ops-exit-{}.txt", request_id)),
            ];
            for path in &cleanup_paths {
                let _ = tokio::fs::remove_file(path).await;
            }
        }

        response
    }

    pub async fn handle_tail(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let file_path = msg.get("file_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let lines = msg.get("lines").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        let follow = msg.get("follow").and_then(|v| v.as_bool()).unwrap_or(false);

        let config = self.config.lock().await;
        if !self.is_path_whitelisted(&file_path, &config.whitelisted_directories) {
            return serde_json::json!({
                "type": "ops-error",
                "node_id": self.node_id,
                "request_id": request_id,
                "error_code": "PATH_NOT_WHITELISTED",
                "message": format!("Path {} is not in the whitelist", file_path),
            });
        }
        drop(config);

        let full_path = self.base_dir.join(&file_path);
        let content = match tokio::fs::read_to_string(&full_path).await {
            Ok(c) => c,
            Err(e) => {
                return serde_json::json!({
                    "type": "ops-error",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "error_code": "TAIL_FAILED",
                    "message": format!("Cannot read file: {}", e),
                });
            }
        };

        let all_lines: Vec<&str> = content.lines().collect();
        let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
        let tail_data = all_lines[start..].join("\n");

        if follow {
            let (tx, _rx) = tokio::sync::watch::channel(false);
            let mut rx = tx.subscribe();
            self.active_tails.lock().await.insert(request_id.clone(), tx);

            let node_id = self.node_id.clone();
            let rid = request_id.clone();
            let fp = file_path.clone();
            let full_p = full_path.clone();
            let tails_ref = self.active_tails.clone();
            let ws_sender = self.ws_sender.clone();
            tokio::spawn(async move {
                let mut last_size = tokio::fs::metadata(&full_p).await
                    .map(|m| m.len()).unwrap_or(0);
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(2)) => {
                            let current_size = tokio::fs::metadata(&full_p).await
                                .map(|m| m.len()).unwrap_or(0);
                            if current_size > last_size {
                                if let Ok(content) = tokio::fs::read_to_string(&full_p).await {
                                    let all_bytes = content.as_bytes();
                                    let safe_offset = last_size.min(all_bytes.len() as u64) as usize;
                                    let new_data = String::from_utf8_lossy(&all_bytes[safe_offset..]).to_string();
                                    if !new_data.is_empty() {
                                        let msg = serde_json::json!({
                                            "type": "tail-data",
                                            "node_id": node_id,
                                            "request_id": rid,
                                            "file_path": fp,
                                            "data": new_data,
                                            "line_count": new_data.lines().count(),
                                            "eof": false,
                                            "following": true,
                                        });
                                        if let Some(sender) = ws_sender.lock().await.as_ref() {
                                            let _ = sender.send(msg);
                                        }
                                    }
                                }
                                last_size = current_size;
                            }
                        }
                        _ = rx.changed() => {
                            let stop_msg = serde_json::json!({
                                "type": "tail-data",
                                "node_id": node_id,
                                "request_id": rid,
                                "file_path": fp,
                                "data": "",
                                "line_count": 0,
                                "eof": true,
                                "following": false,
                            });
                            if let Some(sender) = ws_sender.lock().await.as_ref() {
                                let _ = sender.send(stop_msg);
                            }
                            break;
                        }
                    }
                }
                tails_ref.lock().await.remove(&rid);
            });
        }

        self.write_audit_entry(&OpsAuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "tail".to_string(),
            operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); let ops = self.operators.lock().await; ops.get(fp).map(|o| o.name.clone()).unwrap_or_else(|| fp.to_string()) },
            operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            node_id: self.node_id.clone(),
            request_id: request_id.clone(),
            payload_hash: Self::hash_payload(&file_path),
            script_text: None, exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
            file_path: Some(file_path.clone()),
            file_size: None,
            result: "success".to_string(),
            error_code: None,
            error_message: None,
        }).await;

        serde_json::json!({
            "type": "tail-data",
            "node_id": self.node_id,
            "request_id": request_id,
            "file_path": file_path,
            "data": tail_data,
            "line_count": all_lines.len().min(lines),
            "eof": !follow,
            "following": follow,
        })
    }

    pub async fn handle_tail_stop(&self, msg: &serde_json::Value) {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("");
        let original_id = msg.get("original_request_id")
            .and_then(|v| v.as_str())
            .or_else(|| msg.get("request_id").and_then(|v| v.as_str()))
            .unwrap_or("");
        let mut tails = self.active_tails.lock().await;
        let result = if let Some(sender) = tails.remove(original_id) {
            let _ = sender.send(true);
            println!("[ops] tail-stop sent for {}", original_id);
            "success"
        } else {
            println!("[ops] tail-stop: no active tail found for {}", original_id);
            "not_found"
        };

        self.write_audit_entry(&OpsAuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "tail-stop".to_string(),
            operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); let ops = self.operators.lock().await; ops.get(fp).map(|o| o.name.clone()).unwrap_or_else(|| fp.to_string()) },
            operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            node_id: self.node_id.clone(),
            request_id: request_id.to_string(),
            payload_hash: Self::hash_payload(original_id),
            script_text: None, exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
            file_path: None,
            file_size: None,
            result: result.to_string(),
            error_code: None,
            error_message: None,
        }).await;
    }

    pub async fn cancel_all_tails(&self) {
        let mut tails = self.active_tails.lock().await;
        let count = tails.len();
        for (id, sender) in tails.drain() {
            let _ = sender.send(true);
            println!("[ops] disconnect cleanup: cancelled tail-follow {}", id);
        }
        if count > 0 {
            println!("[ops] disconnect cleanup: cancelled {} active tail follows", count);
        }
    }

    pub async fn collect_telemetry(&self) -> serde_json::Value {
        let uptime = self.process_start.elapsed().as_secs();

        let (cpu_pct, ram_pct, ram_used_mb, ram_total_mb) =
            Self::get_system_metrics().await;

        let (disk_pct, disk_used_gb, disk_total_gb) =
            Self::get_disk_metrics().await;

        let (gpu_pct, gpu_name, gpu_vram_used, gpu_vram_total) =
            Self::get_gpu_metrics().await;

        let os_version = Self::get_os_version();

        let active_model = match tokio::fs::read_to_string(self.base_dir.join(".plenumnet/active-model.json")).await {
            Ok(content) => serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .and_then(|v| v.get("model_name").and_then(|n| n.as_str()).map(|s| s.to_string())),
            Err(_) => None,
        };

        let llm_engine_status = if active_model.is_some() { "loaded" } else { "idle" };

        serde_json::json!({
            "type": "telemetry",
            "node_id": self.node_id,
            "request_id": format!("telem-{}", chrono::Utc::now().timestamp_millis()),
            "cpu_pct": cpu_pct,
            "ram_pct": ram_pct,
            "ram_used_mb": ram_used_mb,
            "ram_total_mb": ram_total_mb,
            "disk_pct": disk_pct,
            "disk_used_gb": disk_used_gb,
            "disk_total_gb": disk_total_gb,
            "gpu_pct": gpu_pct,
            "gpu_name": gpu_name,
            "gpu_vram_used_mb": gpu_vram_used,
            "gpu_vram_total_mb": gpu_vram_total,
            "process_uptime_seconds": uptime,
            "active_model": active_model,
            "llm_engine_status": llm_engine_status,
            "os_version": os_version,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn handle_file_push(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let file_path = msg.get("file_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let data_b64 = msg.get("data_base64").and_then(|v| v.as_str()).unwrap_or("");
        let overwrite = msg.get("overwrite").and_then(|v| v.as_bool()).unwrap_or(false);
        let size_bytes = msg.get("size_bytes").and_then(|v| v.as_u64()).unwrap_or(0);

        let config = self.config.lock().await;
        if !self.is_path_whitelisted(&file_path, &config.whitelisted_directories) {
            drop(config);
            self.audit_error(msg, "file-push", &request_id, "PATH_NOT_WHITELISTED", &format!("Path {} is not whitelisted", file_path)).await;
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "PATH_NOT_WHITELISTED",
                "message": format!("Path {} is not whitelisted", file_path),
            });
        }
        if self.is_extension_blocked(&file_path, &config.blocked_extensions) {
            drop(config);
            self.audit_error(msg, "file-push", &request_id, "EXTENSION_BLOCKED", &format!("File extension is blocked: {}", file_path)).await;
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "EXTENSION_BLOCKED",
                "message": format!("File extension is blocked: {}", file_path),
            });
        }
        let file_size_limit = config.file_size_limit_bytes;
        drop(config);

        let data = match base64_decode(data_b64) {
            Ok(d) => d,
            Err(e) => {
                self.audit_error(msg, "file-push", &request_id, "FILE_WRITE_FAILED", &format!("Base64 decode failed: {}", e)).await;
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "FILE_WRITE_FAILED",
                    "message": format!("Base64 decode failed: {}", e),
                });
            }
        };

        let actual_size = data.len() as u64;
        if actual_size > file_size_limit {
            self.audit_error(msg, "file-push", &request_id, "FILE_TOO_LARGE", &format!("Decoded file size {} exceeds limit {}", actual_size, file_size_limit)).await;
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "FILE_TOO_LARGE",
                "message": format!("Decoded file size {} exceeds limit {} (declared: {})", actual_size, file_size_limit, size_bytes),
            });
        }

        let full_path = self.base_dir.join(&file_path);
        if full_path.exists() && !overwrite {
            self.audit_error(msg, "file-push", &request_id, "OVERWRITE_REQUIRED", "File exists; set overwrite: true to replace").await;
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "OVERWRITE_REQUIRED",
                "message": "File exists; set overwrite: true to replace",
            });
        }

        if let Some(parent) = full_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        match tokio::fs::write(&full_path, &data).await {
            Ok(()) => {
                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "file-push".to_string(),
                    operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); let ops = self.operators.lock().await; ops.get(fp).map(|o| o.name.clone()).unwrap_or_else(|| fp.to_string()) },
                    operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    node_id: self.node_id.clone(),
                    request_id: request_id.clone(),
                    payload_hash: Self::hash_payload(&file_path),
                    script_text: None, exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
                    file_path: Some(file_path.clone()),
                    file_size: Some(data.len() as u64),
                    result: "success".to_string(),
                    error_code: None, error_message: None,
                }).await;
                serde_json::json!({
                    "type": "file-push-ack",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "file_path": file_path,
                    "success": true,
                    "bytes_written": data.len(),
                })
            }
            Err(e) => {
                self.audit_error(msg, "file-push", &request_id, "FILE_WRITE_FAILED", &format!("Write failed: {}", e)).await;
                serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "FILE_WRITE_FAILED",
                    "message": format!("Write failed: {}", e),
                })
            },
        }
    }

    pub async fn handle_file_pull(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let file_path = msg.get("file_path").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let config = self.config.lock().await;
        if !self.is_path_whitelisted(&file_path, &config.whitelisted_directories) {
            drop(config);
            self.audit_error(msg, "file-pull", &request_id, "PATH_NOT_WHITELISTED", &format!("Path {} is not whitelisted", file_path)).await;
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "PATH_NOT_WHITELISTED",
                "message": format!("Path {} is not whitelisted", file_path),
            });
        }
        let file_size_limit = config.file_size_limit_bytes;
        drop(config);

        let full_path = self.base_dir.join(&file_path);
        match tokio::fs::read(&full_path).await {
            Ok(data) => {
                if (data.len() as u64) > file_size_limit {
                    self.audit_error(msg, "file-pull", &request_id, "FILE_TOO_LARGE", &format!("File size {} exceeds pull limit {}", data.len(), file_size_limit)).await;
                    return serde_json::json!({
                        "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                        "error_code": "FILE_TOO_LARGE",
                        "message": format!("File size {} exceeds pull limit {} — use chunked transfer", data.len(), file_size_limit),
                    });
                }
                let encoded = base64_encode(&data);
                let hash = Self::hash_payload(&encoded);
                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: "file-pull".to_string(),
                    operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); let ops = self.operators.lock().await; ops.get(fp).map(|o| o.name.clone()).unwrap_or_else(|| fp.to_string()) },
                    operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    node_id: self.node_id.clone(),
                    request_id: request_id.clone(),
                    payload_hash: hash.clone(),
                    script_text: None, exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
                    file_path: Some(file_path.clone()),
                    file_size: Some(data.len() as u64),
                    result: "success".to_string(),
                    error_code: None, error_message: None,
                }).await;
                serde_json::json!({
                    "type": "file-data",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "file_path": file_path,
                    "data_base64": encoded,
                    "size_bytes": data.len(),
                    "tis27_hash": hash,
                })
            }
            Err(e) => {
                self.audit_error(msg, "file-pull", &request_id, "FILE_READ_FAILED", &format!("Read failed: {}", e)).await;
                serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "FILE_READ_FAILED",
                    "message": format!("Read failed: {}", e),
                })
            },
        }
    }

    pub async fn handle_chunk_init(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let file_path = msg.get("file_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let total_size = msg.get("total_size_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
        let chunk_count = msg.get("chunk_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let chunk_size = msg.get("chunk_size_bytes").and_then(|v| v.as_u64()).unwrap_or(512 * 1024);
        let full_hash = msg.get("tis27_hash_full").and_then(|v| v.as_str()).unwrap_or("").to_string();

        {
            let config = self.config.lock().await;
            if !self.is_path_whitelisted(&file_path, &config.whitelisted_directories) {
                drop(config);
                self.audit_error(msg, "chunk-init", &request_id, "PATH_NOT_WHITELISTED", &format!("Transfer target path {} is not whitelisted", file_path)).await;
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "PATH_NOT_WHITELISTED",
                    "message": format!("Transfer target path {} is not whitelisted", file_path),
                });
            }
            if self.is_extension_blocked(&file_path, &config.blocked_extensions) {
                drop(config);
                self.audit_error(msg, "chunk-init", &request_id, "EXTENSION_BLOCKED", &format!("Transfer target extension is blocked: {}", file_path)).await;
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "EXTENSION_BLOCKED",
                    "message": format!("Transfer target extension is blocked: {}", file_path),
                });
            }
        }

        let resume_transfer_id = msg.get("resume_transfer_id").and_then(|v| v.as_str());

        if let Some(resume_id) = resume_transfer_id {
            let transfers = self.active_transfers.lock().await;
            if let Some(existing) = transfers.get(resume_id) {
                let next_chunk = existing.received_chunks.iter().position(|&c| !c).unwrap_or(existing.chunk_count as usize);
                let received = existing.received_chunks.iter().filter(|&&c| c).count();
                return serde_json::json!({
                    "type": "chunk-ack",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "transfer_id": resume_id,
                    "status": "resumed",
                    "resume_from_chunk": next_chunk,
                    "chunks_received": received,
                    "chunk_count": existing.chunk_count,
                });
            }
            drop(transfers);
            let manifest_path = self.base_dir.join(".plenumnet/transfers").join(resume_id).join("manifest.json");
            if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                if let Ok(existing) = serde_json::from_str::<ChunkTransferManifest>(&content) {
                    let next_chunk = existing.received_chunks.iter().position(|&c| !c).unwrap_or(existing.chunk_count as usize);
                    let received = existing.received_chunks.iter().filter(|&&c| c).count();
                    self.active_transfers.lock().await.insert(resume_id.to_string(), existing.clone());
                    return serde_json::json!({
                        "type": "chunk-ack",
                        "node_id": self.node_id,
                        "request_id": request_id,
                        "transfer_id": resume_id,
                        "status": "resumed",
                        "resume_from_chunk": next_chunk,
                        "chunks_received": received,
                        "chunk_count": existing.chunk_count,
                    });
                }
            }
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "TRANSFER_STALE",
                "message": format!("Cannot resume transfer {}: not found or expired", resume_id),
            });
        }

        let transfer_id = format!("xfer-{}-{}", chrono::Utc::now().timestamp_millis(),
            &request_id[..request_id.len().min(8)]);

        let temp_dir = self.base_dir.join(".plenumnet/transfers").join(&transfer_id);
        let _ = tokio::fs::create_dir_all(&temp_dir).await;

        let payload_hash = Self::hash_payload(&file_path);

        let manifest = ChunkTransferManifest {
            transfer_id: transfer_id.clone(),
            file_path: file_path.clone(),
            total_size_bytes: total_size,
            chunk_count,
            chunk_size_bytes: chunk_size,
            full_hash,
            received_chunks: vec![false; chunk_count as usize],
            started_at: chrono::Utc::now().to_rfc3339(),
            last_activity: chrono::Utc::now().to_rfc3339(),
            temp_dir: temp_dir.to_string_lossy().to_string(),
            operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        };

        self.persist_manifest(&manifest).await;
        self.active_transfers.lock().await.insert(transfer_id.clone(), manifest);

        self.write_audit_entry(&OpsAuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "chunk-init".to_string(),
            operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); let ops = self.operators.lock().await; ops.get(fp).map(|o| o.name.clone()).unwrap_or_else(|| fp.to_string()) },
            operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            node_id: self.node_id.clone(),
            request_id: request_id.clone(),
            payload_hash,
            script_text: None, exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
            file_path: Some(file_path),
            file_size: Some(total_size),
            result: "success".to_string(),
            error_code: None,
            error_message: None,
        }).await;

        serde_json::json!({
            "type": "chunk-ack",
            "node_id": self.node_id,
            "request_id": request_id,
            "transfer_id": transfer_id,
            "chunk_index": -1,
            "success": true,
        })
    }

    pub async fn handle_chunk_data(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let transfer_id = msg.get("transfer_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let chunk_index = msg.get("chunk_index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let data_b64 = msg.get("data_base64").and_then(|v| v.as_str()).unwrap_or("");
        let chunk_hash = msg.get("tis27_hash_chunk").and_then(|v| v.as_str()).unwrap_or("");

        let mut transfers = self.active_transfers.lock().await;
        let manifest = match transfers.get_mut(&transfer_id) {
            Some(m) => m,
            None => {
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "TRANSFER_STALE",
                    "message": format!("Unknown transfer: {}", transfer_id),
                });
            }
        };

        let data = match base64_decode(data_b64) {
            Ok(d) => d,
            Err(_e) => {
                return serde_json::json!({
                    "type": "chunk-ack", "node_id": self.node_id, "request_id": request_id,
                    "transfer_id": transfer_id, "chunk_index": chunk_index,
                    "success": false, "error_code": "TRANSFER_HASH_MISMATCH",
                    "message": "Base64 decode failed",
                });
            }
        };

        if !chunk_hash.is_empty() {
            let computed = Self::hash_payload(data_b64);
            if computed != chunk_hash {
                return serde_json::json!({
                    "type": "chunk-ack", "node_id": self.node_id, "request_id": request_id,
                    "transfer_id": transfer_id, "chunk_index": chunk_index,
                    "success": false, "error_code": "TRANSFER_HASH_MISMATCH",
                    "message": format!("Chunk {} hash mismatch: expected {}, got {}", chunk_index, chunk_hash, computed),
                });
            }
        }

        let chunk_path = PathBuf::from(&manifest.temp_dir).join(format!("chunk-{:06}", chunk_index));
        if let Err(e) = tokio::fs::write(&chunk_path, &data).await {
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "CHUNK_WRITE_FAILED",
                "message": format!("Failed to write chunk: {}", e),
            });
        }

        if chunk_index < manifest.received_chunks.len() {
            manifest.received_chunks[chunk_index] = true;
        }
        manifest.last_activity = chrono::Utc::now().to_rfc3339();

        let all_received = manifest.received_chunks.iter().all(|&r| r);
        let manifest_clone = manifest.clone();
        drop(transfers);

        self.persist_manifest(&manifest_clone).await;

        serde_json::json!({
            "type": "chunk-ack",
            "node_id": self.node_id,
            "request_id": request_id,
            "transfer_id": transfer_id,
            "chunk_index": chunk_index,
            "all_chunks_received": all_received,
            "success": true,
        })
    }

    pub async fn handle_chunk_complete(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let transfer_id = msg.get("transfer_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if transfer_id.is_empty() {
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "TRANSFER_STALE",
                "message": "Missing transfer_id in chunk-complete",
            });
        }

        let transfers = self.active_transfers.lock().await;
        let has_transfer = transfers.contains_key(&transfer_id);
        let all_received = transfers.get(&transfer_id)
            .map(|m| m.received_chunks.iter().all(|&r| r))
            .unwrap_or(false);
        drop(transfers);

        if !has_transfer {
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "TRANSFER_STALE",
                "message": format!("Transfer {} not found", transfer_id),
            });
        }

        if !all_received {
            let transfers = self.active_transfers.lock().await;
            let missing = transfers.get(&transfer_id)
                .map(|m| m.received_chunks.iter().filter(|&&r| !r).count())
                .unwrap_or(0);
            drop(transfers);
            return serde_json::json!({
                "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                "error_code": "CHUNK_WRITE_FAILED",
                "message": format!("Cannot finalize: {} chunks still missing", missing),
            });
        }

        self.finalize_transfer(&transfer_id, &request_id).await
    }

    async fn finalize_transfer(&self, transfer_id: &str, request_id: &str) -> serde_json::Value {
        let mut transfers = self.active_transfers.lock().await;
        let manifest = match transfers.remove(transfer_id) {
            Some(m) => m,
            None => {
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "TRANSFER_STALE",
                    "message": "Transfer not found for finalization",
                });
            }
        };
        drop(transfers);

        {
            let config = self.config.lock().await;
            if !self.is_path_whitelisted(&manifest.file_path, &config.whitelisted_directories) {
                let _ = tokio::fs::remove_dir_all(&manifest.temp_dir).await;
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "PATH_NOT_WHITELISTED",
                    "message": format!("Transfer target path {} is not whitelisted", manifest.file_path),
                });
            }
            if self.is_extension_blocked(&manifest.file_path, &config.blocked_extensions) {
                let _ = tokio::fs::remove_dir_all(&manifest.temp_dir).await;
                return serde_json::json!({
                    "type": "ops-error", "node_id": self.node_id, "request_id": request_id,
                    "error_code": "EXTENSION_BLOCKED",
                    "message": format!("Transfer target extension is blocked: {}", manifest.file_path),
                });
            }
        }

        let mut assembled = Vec::with_capacity(manifest.total_size_bytes as usize);
        for i in 0..manifest.chunk_count {
            let chunk_path = PathBuf::from(&manifest.temp_dir).join(format!("chunk-{:06}", i));
            match tokio::fs::read(&chunk_path).await {
                Ok(data) => assembled.extend_from_slice(&data),
                Err(_e) => {
                    return serde_json::json!({
                        "type": "chunk-complete", "node_id": self.node_id,
                        "request_id": request_id, "transfer_id": transfer_id,
                        "file_path": manifest.file_path, "total_bytes": 0,
                        "tis27_hash_verified": false, "success": false,
                        "error_message": format!("Failed to read chunk {}", i),
                    });
                }
            }
        }

        let computed_hash = {
            let encoded = base64_encode(&assembled);
            Self::hash_payload(&encoded)
        };
        if manifest.full_hash.is_empty() {
            let _ = tokio::fs::remove_dir_all(&manifest.temp_dir).await;
            return serde_json::json!({
                "type": "chunk-complete", "node_id": self.node_id,
                "request_id": request_id, "transfer_id": transfer_id,
                "file_path": manifest.file_path, "total_bytes": assembled.len(),
                "tis27_hash_verified": false, "success": false,
                "error_message": "Transfer rejected: full_hash is required for integrity verification",
            });
        }
        let hash_verified = computed_hash == manifest.full_hash;
        if !hash_verified {
            let _ = tokio::fs::remove_dir_all(&manifest.temp_dir).await;
            return serde_json::json!({
                "type": "chunk-complete", "node_id": self.node_id,
                "request_id": request_id, "transfer_id": transfer_id,
                "file_path": manifest.file_path, "total_bytes": assembled.len(),
                "tis27_hash_verified": false, "success": false,
                "error_message": format!("Hash mismatch: expected {}, got {}", manifest.full_hash, computed_hash),
            });
        }

        let target_path = self.base_dir.join(&manifest.file_path);
        if let Some(parent) = target_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        if let Err(e) = tokio::fs::write(&target_path, &assembled).await {
            return serde_json::json!({
                "type": "chunk-complete", "node_id": self.node_id,
                "request_id": request_id, "transfer_id": transfer_id,
                "file_path": manifest.file_path, "total_bytes": assembled.len(),
                "tis27_hash_verified": hash_verified, "success": false,
                "error_message": format!("Write failed: {}", e),
            });
        }

        let _ = tokio::fs::remove_dir_all(&manifest.temp_dir).await;

        self.write_audit_entry(&OpsAuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "chunk-complete".to_string(),
            operator_name: if manifest.operator_fingerprint.is_empty() { "transfer".to_string() } else { manifest.operator_fingerprint.clone() },
            operator_fingerprint: manifest.operator_fingerprint.clone(),
            node_id: self.node_id.clone(),
            request_id: request_id.to_string(),
            payload_hash: computed_hash.clone(),
            script_text: None, exit_code: None, stdout_truncated: None,
            stderr_truncated: None, duration_ms: None,
            file_path: Some(manifest.file_path.clone()),
            file_size: Some(assembled.len() as u64),
            result: "success".to_string(),
            error_code: None, error_message: None,
        }).await;

        serde_json::json!({
            "type": "chunk-complete",
            "node_id": self.node_id,
            "request_id": request_id,
            "transfer_id": transfer_id,
            "file_path": manifest.file_path,
            "total_bytes": assembled.len(),
            "tis27_hash_verified": hash_verified,
            "success": true,
        })
    }

    pub async fn handle_transfer_cancel(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let transfer_id = msg.get("transfer_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut transfers = self.active_transfers.lock().await;
        if let Some(manifest) = transfers.remove(&transfer_id) {
            let _ = tokio::fs::remove_dir_all(&manifest.temp_dir).await;
        }

        self.write_audit_entry(&OpsAuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "transfer-cancel".to_string(),
            operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); let ops = self.operators.lock().await; ops.get(fp).map(|o| o.name.clone()).unwrap_or_else(|| fp.to_string()) },
            operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            node_id: self.node_id.clone(),
            request_id: request_id.clone(),
            payload_hash: Self::hash_payload(&transfer_id),
            script_text: None, exit_code: None, stdout_truncated: None, stderr_truncated: None, duration_ms: None,
            file_path: None,
            file_size: None,
            result: "success".to_string(),
            error_code: None,
            error_message: None,
        }).await;

        serde_json::json!({
            "type": "chunk-ack",
            "node_id": self.node_id,
            "request_id": request_id,
            "transfer_id": transfer_id,
            "chunk_index": -1,
            "success": true,
        })
    }

    async fn get_known_good_model_path(&self) -> Option<String> {
        let persist_path = self.base_dir.join(".plenumnet").join("active-model.json");
        if let Ok(content) = tokio::fs::read_to_string(&persist_path).await {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                return data.get("model_path").and_then(|v| v.as_str()).map(|s| s.to_string());
            }
        }
        None
    }

    pub async fn handle_model_swap(&self, msg: &serde_json::Value) -> serde_json::Value {
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let model_path = msg.get("model_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let model_name = msg.get("model_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let engine_port = msg.get("engine_port").and_then(|v| v.as_u64()).unwrap_or(8080);

        let normalized_path = model_path.replace('\\', "/");
        if normalized_path.contains("..") || normalized_path.starts_with('/') || normalized_path.contains("://") {
            self.audit_error(msg, "model-swap", &request_id, "PATH_NOT_WHITELISTED", &format!("Model path traversal blocked: {}", model_path)).await;
            return serde_json::json!({
                "type": "model-swap-result",
                "node_id": self.node_id,
                "request_id": request_id,
                "success": false,
                "previous_model": null,
                "new_model": model_name,
                "engine_status": "error",
                "rollback_performed": false,
                "error_message": format!("Model path rejected: directory traversal not allowed ({})", model_path),
            });
        }
        let models_dir = ".plenumnet/models/";
        if !normalized_path.starts_with(models_dir) {
            self.audit_error(msg, "model-swap", &request_id, "PATH_NOT_WHITELISTED", &format!("Model path must be under {}: {}", models_dir, model_path)).await;
            return serde_json::json!({
                "type": "model-swap-result",
                "node_id": self.node_id,
                "request_id": request_id,
                "success": false,
                "previous_model": null,
                "new_model": model_name,
                "engine_status": "error",
                "rollback_performed": false,
                "error_message": format!("Model path must be under {} sandbox", models_dir),
            });
        }
        let full_model_path = self.base_dir.join(&model_path);
        let canonical = full_model_path.canonicalize().unwrap_or_else(|_| full_model_path.clone());
        let models_base = self.base_dir.join(models_dir).canonicalize().unwrap_or_else(|_| self.base_dir.join(models_dir));
        if !canonical.starts_with(&models_base) {
            self.audit_error(msg, "model-swap", &request_id, "PATH_NOT_WHITELISTED", "Canonical path escapes models sandbox").await;
            return serde_json::json!({
                "type": "model-swap-result",
                "node_id": self.node_id,
                "request_id": request_id,
                "success": false,
                "previous_model": null,
                "new_model": model_name,
                "engine_status": "error",
                "rollback_performed": false,
                "error_message": "Model path escapes sandbox after canonicalization",
            });
        }
        if !full_model_path.exists() {
            self.audit_error(msg, "model-swap", &request_id, "FILE_READ_FAILED", &format!("Model file not found: {}", model_path)).await;
            return serde_json::json!({
                "type": "model-swap-result",
                "node_id": self.node_id,
                "request_id": request_id,
                "success": false,
                "previous_model": null,
                "new_model": model_name,
                "engine_status": "error",
                "rollback_performed": false,
                "error_message": format!("Model file not found: {}", model_path),
            });
        }

        if !model_path.to_lowercase().ends_with(".gguf") {
            return serde_json::json!({
                "type": "model-swap-result",
                "node_id": self.node_id,
                "request_id": request_id,
                "success": false,
                "previous_model": null,
                "new_model": model_name,
                "engine_status": "error",
                "rollback_performed": false,
                "error_message": "Only .gguf model files are supported for hot-swap",
            });
        }

        let metadata = tokio::fs::metadata(&full_model_path).await;
        let file_size_mb = metadata.map(|m| m.len() / (1024 * 1024)).unwrap_or(0);

        if let Ok(mut f) = tokio::fs::File::open(&full_model_path).await {
            use tokio::io::AsyncReadExt;
            let mut magic = [0u8; 4];
            if let Ok(4) = f.read(&mut magic).await {
                if &magic != b"GGUF" {
                    return serde_json::json!({
                        "type": "model-swap-result",
                        "node_id": self.node_id,
                        "request_id": request_id,
                        "success": false,
                        "previous_model": null,
                        "new_model": model_name,
                        "engine_status": "pre-validation-failed",
                        "rollback_performed": false,
                        "error_message": "File does not have valid GGUF magic bytes — refusing swap to prevent engine disruption",
                    });
                }
            }
        }

        let health_check = reqwest::Client::new()
            .get(format!("http://127.0.0.1:{}/health", engine_port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        let previous_model = if let Ok(resp) = &health_check {
            if resp.status().is_success() {
                Some("active".to_string())
            } else {
                None
            }
        } else {
            None
        };

        let engine_running = health_check.is_ok();

        println!("[ops] model-swap: engine_status → swapping (loading {})", model_name);

        if engine_running {
            let stop_result = reqwest::Client::new()
                .post(format!("http://127.0.0.1:{}/slots/0?action=erase", engine_port))
                .timeout(Duration::from_secs(10))
                .send()
                .await;

            if let Err(e) = &stop_result {
                println!("[ops] model-swap: slot erase request failed (non-fatal): {}", e);
            }
        }

        let reload_result = reqwest::Client::new()
            .post(format!("http://127.0.0.1:{}/v1/internal/model/load", engine_port))
            .json(&serde_json::json!({
                "model_path": full_model_path.to_string_lossy(),
            }))
            .timeout(Duration::from_secs(120))
            .send()
            .await;

        let (success, engine_status, error_msg) = match reload_result {
            Ok(resp) if resp.status().is_success() => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                let verify = reqwest::Client::new()
                    .get(format!("http://127.0.0.1:{}/v1/models", engine_port))
                    .timeout(Duration::from_secs(10))
                    .send()
                    .await;
                match verify {
                    Ok(v_resp) if v_resp.status().is_success() => {
                        println!("[ops] model-swap: post-load /v1/models verification passed");
                        (true, "running".to_string(), None)
                    }
                    Ok(v_resp) => {
                        let body = v_resp.text().await.unwrap_or_default();
                        (false, "degraded".to_string(),
                            Some(format!("Model loaded but /v1/models verification failed: {}", body)))
                    }
                    Err(e) => {
                        (false, "degraded".to_string(),
                            Some(format!("Model loaded but engine unreachable for verification: {}", e)))
                    }
                }
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                (false, "error".to_string(),
                    Some(format!("Engine returned {} during model load: {}", status, body)))
            }
            Err(e) => {
                (false, "unreachable".to_string(),
                    Some(format!("Failed to contact engine for model load: {}", e)))
            }
        };

        let mut rollback_performed = false;
        let mut rollback_verified = false;
        let mut engine_final_status = engine_status.clone();
        if !success {
            let previous_model_path = msg.get("previous_model_path").and_then(|v| v.as_str()).unwrap_or("");
            let prev_normalized = previous_model_path.replace('\\', "/");
            let prev_path_safe = !prev_normalized.contains("..") && !prev_normalized.starts_with('/') && prev_normalized.starts_with(models_dir);
            if !previous_model_path.is_empty() && prev_path_safe {
                let prev_full = self.base_dir.join(previous_model_path);
                if prev_full.exists() {
                    println!("[ops] model-swap failed, attempting rollback to {}", previous_model_path);
                    let rollback_result = reqwest::Client::new()
                        .post(format!("http://127.0.0.1:{}/v1/internal/model/load", engine_port))
                        .json(&serde_json::json!({
                            "model_path": prev_full.to_string_lossy(),
                        }))
                        .timeout(Duration::from_secs(120))
                        .send()
                        .await;
                    match rollback_result {
                        Ok(resp) if resp.status().is_success() => {
                            rollback_performed = true;
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            let verify = reqwest::Client::new()
                                .get(format!("http://127.0.0.1:{}/v1/models", engine_port))
                                .timeout(Duration::from_secs(10))
                                .send()
                                .await;
                            match verify {
                                Ok(v) if v.status().is_success() => {
                                    println!("[ops] model-swap rollback verified — {} serving again", previous_model_path);
                                    rollback_verified = true;
                                    engine_final_status = "running_rollback".to_string();
                                }
                                _ => {
                                    println!("[ops] model-swap rollback load succeeded but verification failed");
                                    engine_final_status = "degraded_rollback".to_string();
                                }
                            }
                        }
                        _ => {
                            println!("[ops] model-swap rollback FAILED — engine in degraded state, attempting health recovery");
                            let health = reqwest::Client::new()
                                .get(format!("http://127.0.0.1:{}/health", engine_port))
                                .timeout(Duration::from_secs(5))
                                .send()
                                .await;
                            if health.map(|r| r.status().is_success()).unwrap_or(false) {
                                engine_final_status = "degraded_no_model".to_string();
                                println!("[ops] Engine process alive but no model loaded — manual intervention required");
                            } else {
                                engine_final_status = "dead".to_string();
                                println!("[ops] CRITICAL: Engine process unreachable — manual restart required");
                            }
                        }
                    }
                }
            }
            if !rollback_performed {
                let known_good_path = self.get_known_good_model_path().await;
                if let Some(ref good_path) = known_good_path {
                    let good_full = self.base_dir.join(good_path);
                    if good_full.exists() {
                        println!("[ops] model-swap: rollback to known-good model {}", good_path);
                        let rollback_good = reqwest::Client::new()
                            .post(format!("http://127.0.0.1:{}/v1/internal/model/load", engine_port))
                            .json(&serde_json::json!({ "model_path": good_full.to_string_lossy() }))
                            .timeout(Duration::from_secs(120))
                            .send()
                            .await;
                        match rollback_good {
                            Ok(r) if r.status().is_success() => {
                                rollback_performed = true;
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                let verify = reqwest::Client::new()
                                    .get(format!("http://127.0.0.1:{}/v1/models", engine_port))
                                    .timeout(Duration::from_secs(10))
                                    .send()
                                    .await;
                                match verify {
                                    Ok(v) if v.status().is_success() => {
                                        rollback_verified = true;
                                        engine_final_status = "running_rollback".to_string();
                                        println!("[ops] model-swap rollback to known-good model verified — {} serving", good_path);
                                    }
                                    _ => {
                                        engine_final_status = "degraded_rollback".to_string();
                                        println!("[ops] model-swap rollback load succeeded but verification failed");
                                    }
                                }
                            }
                            _ => {
                                println!("[ops] model-swap: known-good rollback FAILED — attempting engine restart");
                            }
                        }
                    }
                }

                if !rollback_performed {
                    let restart_model = known_good_path
                        .map(|p| self.base_dir.join(&p))
                        .unwrap_or_else(|| full_model_path.clone());
                    println!("[ops] CRITICAL: All rollbacks failed — attempting engine restart with {}", restart_model.display());
                    let restart = tokio::process::Command::new("llama-server")
                        .args(["--port", &engine_port.to_string(),
                               "--model", &restart_model.to_string_lossy()])
                        .spawn();
                    match restart {
                        Ok(_) => {
                            for attempt in 1..=3 {
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                let post_restart = reqwest::Client::new()
                                    .get(format!("http://127.0.0.1:{}/health", engine_port))
                                    .timeout(Duration::from_secs(10))
                                    .send()
                                    .await;
                                if post_restart.map(|r| r.status().is_success()).unwrap_or(false) {
                                    engine_final_status = "running_restarted".to_string();
                                    rollback_performed = true;
                                    println!("[ops] Engine restarted successfully on attempt {}", attempt);
                                    break;
                                }
                                println!("[ops] Engine restart attempt {}/3 — not yet healthy", attempt);
                            }
                            if !rollback_performed {
                                engine_final_status = "recovery_failed".to_string();
                                println!("[ops] CRITICAL: Engine restart failed after 3 attempts");
                            }
                        }
                        Err(e) => {
                            engine_final_status = "recovery_failed".to_string();
                            println!("[ops] CRITICAL: Engine restart spawn failed: {}", e);
                        }
                    }
                }
            }
        }

        self.write_audit_entry(&OpsAuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: "model-swap".to_string(),
            operator_name: { let fp = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or(""); let ops = self.operators.lock().await; ops.get(fp).map(|o| o.name.clone()).unwrap_or_else(|| fp.to_string()) },
            operator_fingerprint: msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            node_id: self.node_id.clone(),
            request_id: request_id.clone(),
            payload_hash: Self::hash_payload(&model_path),
            script_text: None, exit_code: None, stdout_truncated: None,
            stderr_truncated: None, duration_ms: None,
            file_path: Some(model_path.clone()),
            file_size: Some(file_size_mb * 1024 * 1024),
            result: if success { "success" } else { "failure" }.to_string(),
            error_code: if success { None } else { Some("MODEL_SWAP_FAILED".to_string()) },
            error_message: error_msg.clone(),
        }).await;

        if success {
            let persist_path = self.base_dir.join(".plenumnet").join("active-model.json");
            let _ = tokio::fs::create_dir_all(self.base_dir.join(".plenumnet")).await;
            let persist_data = serde_json::json!({
                "model_path": model_path,
                "model_name": model_name,
                "swapped_at": chrono::Utc::now().to_rfc3339(),
            });
            if let Err(e) = tokio::fs::write(&persist_path, serde_json::to_string_pretty(&persist_data).unwrap_or_default()).await {
                eprintln!("[ops] WARNING: failed to persist active model config: {}", e);
            }
        }

        let mut result = serde_json::json!({
            "type": "model-swap-result",
            "node_id": self.node_id,
            "request_id": request_id,
            "success": success,
            "previous_model": previous_model,
            "new_model": model_name,
            "engine_status": engine_final_status,
            "rollback_performed": rollback_performed,
            "rollback_verified": rollback_verified,
            "model_size_mb": file_size_mb,
        });

        if let Some(err) = error_msg {
            result.as_object_mut().unwrap().insert("error_message".to_string(), serde_json::json!(err));
        }

        result
    }

    fn requires_signature(msg_type: &str) -> bool {
        matches!(msg_type,
            "exec" | "tail" | "tail-stop" | "file-push" | "file-pull" |
            "chunk-init" | "chunk-data" | "chunk-complete" | "transfer-cancel" | "model-swap"
        )
    }

    pub async fn handle_ops_message(&self, msg: &serde_json::Value) -> Option<serde_json::Value> {
        let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let request_id = msg.get("request_id").and_then(|v| v.as_str()).unwrap_or("");

        if !self.is_enabled().await && msg_type != "telemetry" {
            return Some(serde_json::json!({
                "type": "ops-error",
                "node_id": self.node_id,
                "request_id": request_id,
                "error_code": "OPS_DISABLED",
                "message": "The operations channel is inactive on this node",
            }));
        }

        if Self::requires_signature(msg_type) {
            let signature = msg.get("signature").and_then(|v| v.as_str()).unwrap_or("");
            let fingerprint = msg.get("operator_fingerprint").and_then(|v| v.as_str()).unwrap_or("");

            if signature.is_empty() {
                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: msg_type.to_string(),
                    operator_name: self.resolve_operator_name(fingerprint).await,
                    operator_fingerprint: fingerprint.to_string(),
                    node_id: self.node_id.clone(),
                    request_id: request_id.to_string(),
                    payload_hash: Self::hash_payload(&msg.to_string()),
                    script_text: None, exit_code: None, stdout_truncated: None,
                    stderr_truncated: None, duration_ms: None, file_path: None,
                    file_size: None,
                    result: "rejected".to_string(),
                    error_code: Some("SIGNATURE_MISSING".to_string()),
                    error_message: Some("Signature required for this operation".to_string()),
                }).await;
                return Some(serde_json::json!({
                    "type": "ops-error",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "error_code": "SIGNATURE_MISSING",
                    "message": "Signature required for this operation",
                }));
            }

            if fingerprint.is_empty() {
                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: msg_type.to_string(),
                    operator_name: "unknown-no-fingerprint".to_string(),
                    operator_fingerprint: "".to_string(),
                    node_id: self.node_id.clone(),
                    request_id: request_id.to_string(),
                    payload_hash: Self::hash_payload(&msg.to_string()),
                    script_text: None, exit_code: None, stdout_truncated: None,
                    stderr_truncated: None, duration_ms: None, file_path: None,
                    file_size: None,
                    result: "rejected".to_string(),
                    error_code: Some("SIGNATURE_MISSING".to_string()),
                    error_message: Some("Operator fingerprint required".to_string()),
                }).await;
                return Some(serde_json::json!({
                    "type": "ops-error",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "error_code": "SIGNATURE_MISSING",
                    "message": "Operator fingerprint required",
                }));
            }

            let operator = match self.validate_operator(fingerprint, msg_type).await {
                Ok(op) => op,
                Err((code, message)) => {
                    self.write_audit_entry(&OpsAuditEntry {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        operation: msg_type.to_string(),
                        operator_name: self.resolve_operator_name(fingerprint).await,
                        operator_fingerprint: fingerprint.to_string(),
                        node_id: self.node_id.clone(),
                        request_id: request_id.to_string(),
                        payload_hash: Self::hash_payload(&msg.to_string()),
                        script_text: None, exit_code: None, stdout_truncated: None,
                        stderr_truncated: None, duration_ms: None, file_path: None,
                        file_size: None,
                        result: "rejected".to_string(),
                        error_code: Some(code.clone()),
                        error_message: Some(message.clone()),
                    }).await;
                    return Some(serde_json::json!({
                        "type": "ops-error",
                        "node_id": self.node_id,
                        "request_id": request_id,
                        "error_code": code,
                        "message": message,
                    }));
                }
            };

            let signing_payload = Self::build_signing_payload(msg);
            if !Self::verify_tl_dsa_signature(&operator.public_key, &signing_payload, signature) {
                self.write_audit_entry(&OpsAuditEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    operation: msg_type.to_string(),
                    operator_name: operator.name.clone(),
                    operator_fingerprint: fingerprint.to_string(),
                    node_id: self.node_id.clone(),
                    request_id: request_id.to_string(),
                    payload_hash: Self::hash_payload(&msg.to_string()),
                    script_text: None, exit_code: None, stdout_truncated: None,
                    stderr_truncated: None, duration_ms: None, file_path: None,
                    file_size: None,
                    result: "rejected".to_string(),
                    error_code: Some("SIGNATURE_INVALID".to_string()),
                    error_message: Some("TL-DSA signature verification failed".to_string()),
                }).await;
                return Some(serde_json::json!({
                    "type": "ops-error",
                    "node_id": self.node_id,
                    "request_id": request_id,
                    "error_code": "SIGNATURE_INVALID",
                    "message": "TL-DSA signature verification failed — payload integrity check did not pass",
                }));
            }

            println!("[ops] Operator {} authorized for {} (scope: {}, sig: verified)",
                operator.name, msg_type, operator.scope);
        }

        match msg_type {
            "exec" => Some(self.handle_exec(msg).await),
            "tail" => Some(self.handle_tail(msg).await),
            "tail-stop" => { self.handle_tail_stop(msg).await; None }
            "file-push" => Some(self.handle_file_push(msg).await),
            "file-pull" => Some(self.handle_file_pull(msg).await),
            "chunk-init" => Some(self.handle_chunk_init(msg).await),
            "chunk-data" => Some(self.handle_chunk_data(msg).await),
            "chunk-complete" => Some(self.handle_chunk_complete(msg).await),
            "transfer-cancel" => Some(self.handle_transfer_cancel(msg).await),
            "model-swap" => Some(self.handle_model_swap(msg).await),
            _ => None,
        }
    }

    async fn write_audit_entry(&self, entry: &OpsAuditEntry) {
        let config = self.config.lock().await;
        let audit_path = self.base_dir.join(&config.audit_log_path);
        drop(config);

        if let Some(parent) = audit_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        if let Ok(json_line) = serde_json::to_string(entry) {
            let line = format!("{}\n", json_line);
            use tokio::io::AsyncWriteExt;
            if let Ok(mut f) = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&audit_path)
                .await
            {
                let _ = f.write_all(line.as_bytes()).await;
                let _ = f.flush().await;
            }
        }
    }

    async fn persist_manifest(&self, manifest: &ChunkTransferManifest) {
        let manifest_path = PathBuf::from(&manifest.temp_dir).join("manifest.json");
        if let Ok(json) = serde_json::to_string_pretty(manifest) {
            let _ = tokio::fs::write(&manifest_path, json.as_bytes()).await;
        }
    }

    pub async fn load_persisted_transfers(&self) {
        let transfers_dir = self.base_dir.join(".plenumnet/transfers");
        if !transfers_dir.exists() {
            return;
        }
        let mut entries = match tokio::fs::read_dir(&transfers_dir).await {
            Ok(e) => e,
            Err(_) => return,
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let manifest_path = entry.path().join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                    if let Ok(manifest) = serde_json::from_str::<ChunkTransferManifest>(&content) {
                        println!("[ops] Restored transfer manifest: {} ({}/{})",
                            manifest.transfer_id,
                            manifest.received_chunks.iter().filter(|&&r| r).count(),
                            manifest.chunk_count);
                        self.active_transfers.lock().await.insert(manifest.transfer_id.clone(), manifest);
                    }
                }
            }
        }
    }

    fn hash_payload(payload: &str) -> String {
        let hash_bytes = ternary_math::tlsponge385::hash(payload.as_bytes(), 32);
        hex::encode(&hash_bytes[..16])
    }

    async fn get_system_metrics() -> (f64, f64, u64, u64) {
        #[cfg(target_os = "windows")]
        {
            let mut cpu_pct = 0.0f64;
            let mut ram_pct = 0.0f64;
            let mut ram_used = 0u64;
            let mut ram_total = 0u64;

            if let Ok(output) = tokio::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-Command",
                    "@{cpu=(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average; mem=Get-CimInstance Win32_OperatingSystem | Select-Object TotalVisibleMemorySize,FreePhysicalMemory} | ConvertTo-Json"])
                .output().await
            {
                if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    cpu_pct = parsed.get("cpu").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if let Some(mem) = parsed.get("mem") {
                        let total_kb = mem.get("TotalVisibleMemorySize").and_then(|v| v.as_u64()).unwrap_or(0);
                        let free_kb = mem.get("FreePhysicalMemory").and_then(|v| v.as_u64()).unwrap_or(0);
                        ram_total = total_kb / 1024;
                        ram_used = (total_kb - free_kb) / 1024;
                        if total_kb > 0 {
                            ram_pct = ((total_kb - free_kb) as f64 / total_kb as f64) * 100.0;
                        }
                    }
                }
            }
            (cpu_pct, ram_pct, ram_used, ram_total)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let result = tokio::task::spawn_blocking(|| {
                let mut cpu_pct = 0.0f64;
                let mut ram_pct = 0.0f64;
                let mut ram_used = 0u64;
                let mut ram_total = 0u64;

                if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                    let mut total = 0u64;
                    let mut available = 0u64;
                    for line in content.lines() {
                        if line.starts_with("MemTotal:") {
                            total = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                        } else if line.starts_with("MemAvailable:") {
                            available = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                        }
                    }
                    ram_total = total / 1024;
                    ram_used = (total - available) / 1024;
                    if total > 0 {
                        ram_pct = ((total - available) as f64 / total as f64) * 100.0;
                    }
                }

                if let Ok(stat1) = std::fs::read_to_string("/proc/stat") {
                    let parse_cpu = |s: &str| -> (u64, u64) {
                        if let Some(line) = s.lines().next() {
                            let parts: Vec<u64> = line.split_whitespace().skip(1)
                                .filter_map(|v| v.parse().ok()).collect();
                            if parts.len() >= 4 {
                                let total: u64 = parts.iter().sum();
                                return (total, parts[3]);
                            }
                        }
                        (0, 0)
                    };
                    let (total1, idle1) = parse_cpu(&stat1);
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    if let Ok(stat2) = std::fs::read_to_string("/proc/stat") {
                        let (total2, idle2) = parse_cpu(&stat2);
                        let dt = total2.saturating_sub(total1);
                        let di = idle2.saturating_sub(idle1);
                        if dt > 0 {
                            cpu_pct = ((dt - di) as f64 / dt as f64) * 100.0;
                        }
                    }
                }

                (cpu_pct, ram_pct, ram_used, ram_total)
            }).await;
            result.unwrap_or((0.0, 0.0, 0, 0))
        }
    }

    async fn get_disk_metrics() -> (f64, f64, f64) {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = tokio::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-Command",
                    "Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3' | Select-Object Size,FreeSpace | ConvertTo-Json"])
                .output().await
            {
                if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    let total = parsed.get("Size").and_then(|v| v.as_u64()).unwrap_or(0);
                    let free = parsed.get("FreeSpace").and_then(|v| v.as_u64()).unwrap_or(0);
                    let total_gb = total as f64 / (1024.0 * 1024.0 * 1024.0);
                    let used_gb = (total - free) as f64 / (1024.0 * 1024.0 * 1024.0);
                    let pct = if total > 0 { ((total - free) as f64 / total as f64) * 100.0 } else { 0.0 };
                    return (pct, used_gb, total_gb);
                }
            }
            (0.0, 0.0, 0.0)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let output = tokio::task::spawn_blocking(|| {
                std::process::Command::new("df")
                    .args(["-B1", "/"])
                    .output()
            }).await;
            if let Ok(Ok(df_out)) = output {
                let stdout = String::from_utf8_lossy(&df_out.stdout);
                if let Some(line) = stdout.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let total = parts[1].parse::<u64>().unwrap_or(0);
                        let used = parts[2].parse::<u64>().unwrap_or(0);
                        let total_gb = total as f64 / (1024.0 * 1024.0 * 1024.0);
                        let used_gb = used as f64 / (1024.0 * 1024.0 * 1024.0);
                        let pct = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
                        return (pct, used_gb, total_gb);
                    }
                }
            }
            (0.0, 0.0, 0.0)
        }
    }

    async fn get_gpu_metrics() -> (Option<f64>, Option<String>, Option<u64>, Option<u64>) {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = tokio::process::Command::new("nvidia-smi")
                .args(["--query-gpu=utilization.gpu,name,memory.used,memory.total", "--format=csv,noheader,nounits"])
                .output().await
            {
                if output.status.success() {
                    let line = String::from_utf8_lossy(&output.stdout);
                    let parts: Vec<&str> = line.trim().split(", ").collect();
                    if parts.len() >= 4 {
                        let gpu_pct = parts[0].parse::<f64>().ok();
                        let name = Some(parts[1].to_string());
                        let vram_used = parts[2].parse::<u64>().ok();
                        let vram_total = parts[3].parse::<u64>().ok();
                        return (gpu_pct, name, vram_used, vram_total);
                    }
                }
            }
            (None, None, None, None)
        }
        #[cfg(not(target_os = "windows"))]
        {
            (None, None, None, None)
        }
    }

    fn get_os_version() -> String {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("cmd")
                .args(["/C", "ver"])
                .output()
            {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !ver.is_empty() { return ver; }
            }
            "Windows".to_string()
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        return line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string();
                    }
                }
            }
            if let Ok(content) = std::fs::read_to_string("/proc/version") {
                return content.trim().to_string();
            }
            "Linux".to_string()
        }
    }

    pub async fn cleanup_stale_transfers(&self) {
        let stale_cutoff = chrono::Utc::now() - chrono::Duration::minutes(60);
        let mut transfers = self.active_transfers.lock().await;
        let stale_ids: Vec<String> = transfers.iter()
            .filter(|(_, m)| {
                chrono::DateTime::parse_from_rfc3339(&m.last_activity)
                    .map(|dt| dt < stale_cutoff)
                    .unwrap_or(true)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in stale_ids {
            if let Some(manifest) = transfers.remove(&id) {
                let _ = tokio::fs::remove_dir_all(&manifest.temp_dir).await;
                println!("[ops] Cleaned up stale transfer: {}", id);
            }
        }
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let table: [u8; 256] = {
        let mut t = [255u8; 256];
        for (i, &c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            t[c as usize] = i as u8;
        }
        t
    };
    let input = input.trim_end_matches('=');
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let bytes = input.as_bytes();
    let chunks = bytes.chunks(4);
    for chunk in chunks {
        let mut buf = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            let v = table[b as usize];
            if v == 255 { return Err(format!("Invalid base64 char: {}", b as char)); }
            buf[i] = v;
        }
        out.push((buf[0] << 2) | (buf[1] >> 4));
        if chunk.len() > 2 { out.push((buf[1] << 4) | (buf[2] >> 2)); }
        if chunk.len() > 3 { out.push((buf[2] << 6) | buf[3]); }
    }
    Ok(out)
}

fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((n >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(n & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}
