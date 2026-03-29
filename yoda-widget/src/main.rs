// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Yoda Widget — Ctrl+Y Chat Interface
//!
//! Separate crate and binary — NOT feature-gated within the daemon crate.
//! Prevents wry/WebView dependencies from infecting headless daemon builds.
//!
//! ## Build Variants
//!
//! - **Headless** (default): No WebView dependency, prints template info
//! - **Desktop** (`--features desktop`): Full tao+wry window with Ctrl+Y hotkey
//!
//! On headless systems (no display server), the widget binary is not installed
//! and the Ctrl+Y hotkey is not registered — this is a non-error condition.
//!
//! ## Security
//!
//! - Plain text rendering only — no HTML rendering, no script execution
//! - CSP: `default-src 'none'; style-src 'unsafe-inline'; script-src 'self'`
//! - Network access restricted to 127.0.0.1 only
//!
//! ## Typography
//!
//! All text rendered in JetBrains Mono (bundled as WOFF2).
//!
//! ## Platform Dependencies
//!
//! - Windows: WebView2 (minimum version 90)
//! - Linux: webkit2gtk-4.1

use inter_cube::yoda_chat::*;

const WIDGET_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'self'; font-src data:">
<title>Yoda Chat</title>
<style>
:root {
  --background: hsl(20, 14%, 4%);
  --foreground: hsl(60, 10%, 90%);
  --card: hsl(24, 10%, 10%);
  --primary: hsl(210, 80%, 55%);
  --muted-foreground: hsl(45, 15%, 46%);
  --destructive: hsl(356, 91%, 54%);
  --accent-foreground: hsl(48, 96%, 89%);
}
@font-face {
  font-family: 'JetBrains Mono';
  src: local('JetBrains Mono'), local('JetBrainsMono-Regular');
  font-weight: 400;
  font-style: normal;
}
@font-face {
  font-family: 'JetBrains Mono';
  src: local('JetBrains Mono Bold'), local('JetBrainsMono-Bold');
  font-weight: 700;
  font-style: normal;
}
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', 'Source Code Pro', 'Consolas', monospace;
  font-size: 13px;
  background: var(--background);
  color: var(--foreground);
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.header {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  gap: 6px;
  font-size: 11px;
  color: var(--muted-foreground);
  border-bottom: 1px solid hsl(24, 10%, 15%);
}
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--muted-foreground);
  flex-shrink: 0;
}
.status-dot.connected { background: var(--primary); }
.status-dot.error { background: var(--destructive); }
.messages {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.msg {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
}
.msg .prefix {
  font-weight: bold;
  color: var(--primary);
}
.input-area {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  gap: 6px;
  border-top: 1px solid hsl(24, 10%, 15%);
}
.input-area input {
  flex: 1;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  background: var(--card);
  color: var(--foreground);
  border: 1px solid hsl(24, 10%, 20%);
  border-radius: 4px;
  padding: 6px 8px;
  outline: none;
}
.input-area input:focus { border-color: var(--primary); }
.input-area input:disabled { opacity: 0.5; }
.signing-indicator {
  font-size: 11px;
  color: var(--accent-foreground);
  display: none;
}
.signing-indicator.visible { display: inline; }
</style>
</head>
<body>
<div class="header">
  <div class="status-dot" id="statusDot"></div>
  <span id="statusLabel">Disconnected</span>
  <span class="signing-indicator" id="signingIndicator">Signing...</span>
</div>
<div class="messages" id="messages" tabindex="0"></div>
<div class="input-area">
  <input type="text" id="input" placeholder="Send a message to Yoda..."
         autofocus data-testid="input-yoda-message">
</div>
</body>
</html>"#;

const WIDGET_JS: &str = r#"
(function() {
  const messages = document.getElementById('messages');
  const input = document.getElementById('input');
  const statusDot = document.getElementById('statusDot');
  const statusLabel = document.getElementById('statusLabel');
  const signingIndicator = document.getElementById('signingIndicator');
  let history = [];
  let historyIndex = -1;
  let isSigning = false;
  let lastDraft = '';

  function addMessage(text, isYoda) {
    const div = document.createElement('div');
    div.className = 'msg';
    if (isYoda) {
      const prefix = document.createElement('span');
      prefix.className = 'prefix';
      prefix.textContent = '[YODA] ';
      div.appendChild(prefix);
      div.appendChild(document.createTextNode(text));
    } else {
      div.textContent = '> ' + text;
    }
    messages.appendChild(div);
    messages.scrollTop = messages.scrollHeight;
  }

  function setStatus(state) {
    statusDot.className = 'status-dot ' + state;
    statusLabel.textContent = state === 'connected' ? 'Connected' :
                              state === 'error' ? 'Error' : 'Disconnected';
  }

  function setSigning(active) {
    isSigning = active;
    input.disabled = active;
    signingIndicator.className = 'signing-indicator' + (active ? ' visible' : '');
    if (!active) input.focus();
  }

  function sendMessage() {
    const text = input.value.trim();
    if (!text || isSigning) return;
    lastDraft = '';
    history.push(text);
    historyIndex = history.length;
    addMessage(text, false);
    pendingDraft = text;
    input.value = '';
    setSigning(true);
    if (window.ipc) {
      window.ipc.postMessage(JSON.stringify({ type: 'send', message: text }));
    } else {
      setTimeout(function() {
        addMessage('Widget running in preview mode — connect to daemon for live chat.', true);
        setSigning(false);
      }, 300);
    }
  }

  var pendingDraft = '';
  window.__yodaReceive = function(content) {
    addMessage(content || 'No response.', true);
    pendingDraft = '';
    setSigning(false);
  };
  window.__yodaError = function(errMsg) {
    addMessage('Error: ' + (errMsg || 'Unknown error'), true);
    if (pendingDraft) {
      input.value = pendingDraft;
      pendingDraft = '';
    }
    setSigning(false);
  };

  if (window.ipc) {
    setStatus('connected');
  } else {
    setStatus('disconnected');
  }

  input.addEventListener('keydown', function(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      sendMessage();
    } else if (e.key === 'ArrowUp' && input.value === '' && history.length > 0) {
      e.preventDefault();
      if (historyIndex > 0) historyIndex--;
      input.value = history[historyIndex] || '';
    } else if (e.key === 'Escape') {
      e.preventDefault();
      if (input.value !== '') {
        lastDraft = input.value;
        input.value = '';
      } else if (window.ipc) {
        window.ipc.postMessage(JSON.stringify({ type: 'dismiss' }));
      }
    }
  });

  document.addEventListener('keydown', function(e) {
    if (e.key === 'Tab') {
      e.preventDefault();
      if (document.activeElement === input) {
        messages.focus();
      } else {
        input.focus();
      }
    }
  });
})();
"#;

#[cfg(feature = "desktop")]
fn daemon_api_url() -> String {
    let port = std::env::var("CUBE_API_PORT")
        .or_else(|_| std::env::var("API_PORT"))
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8181);
    format!("127.0.0.1:{}", port)
}

#[cfg(feature = "desktop")]
fn widget_query_daemon_rep_c() -> Option<String> {
    let addr = daemon_api_url();
    let tcp = std::net::TcpStream::connect_timeout(
        &addr.parse().ok()?,
        std::time::Duration::from_secs(3),
    ).ok()?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
    let request = format!("GET /health HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", addr);
    use std::io::{Read, Write as IoWrite};
    let mut stream = tcp;
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).ok()?;
    let response_str = String::from_utf8_lossy(&response);
    let body_start = response_str.find("\r\n\r\n")?;
    let body = &response_str[body_start + 4..];
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    json.get("address").and_then(|v| v.as_str()).map(|s| s.to_string())
}

#[cfg(feature = "desktop")]
fn widget_submit_to_daemon(payload_json: &str) -> Result<serde_json::Value, String> {
    let addr = daemon_api_url();
    let tcp = std::net::TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("{}", e))?,
        std::time::Duration::from_secs(3),
    ).map_err(|e| format!("connect: {}", e))?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(35))).ok();
    let request = format!(
        "POST /yoda/submit HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        addr, payload_json.len(), payload_json
    );
    use std::io::{Read, Write as IoWrite};
    let mut stream = tcp;
    stream.write_all(request.as_bytes()).map_err(|e| format!("write: {}", e))?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|e| format!("read: {}", e))?;
    let response_str = String::from_utf8_lossy(&response);
    let body_start = response_str.find("\r\n\r\n").ok_or("no body")?;
    let body = &response_str[body_start + 4..];
    serde_json::from_str(body).map_err(|e| format!("parse: {}", e))
}

#[cfg(feature = "desktop")]
fn widget_ninjaexec_sign(data: &[u8]) -> Option<(String, String)> {
    let body = serde_json::json!({
        "payload_b64": base64_encode(data),
        "context": YODA_CHAT_CONTEXT,
    });
    let body_str = body.to_string();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", 21027).parse().ok()?;
    let tcp = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(3)).ok()?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();
    let request = format!(
        "POST /sign HTTP/1.1\r\nHost: 127.0.0.1:21027\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body_str.len(), body_str
    );
    use std::io::{Read, Write as IoWrite};
    let mut stream = tcp;
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).ok()?;
    let response_str = String::from_utf8_lossy(&response);
    let body_start = response_str.find("\r\n\r\n")?;
    let body = &response_str[body_start + 4..];
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let sig = json.get("signature_b64").and_then(|v| v.as_str())?.to_string();
    let pk = json.get("pubkey_b64").and_then(|v| v.as_str())?.to_string();
    Some((sig, pk))
}

#[cfg(feature = "desktop")]
#[derive(Debug)]
enum WidgetUserEvent {
    YodaResponse(String),
    YodaError(String),
    ToggleVisibility,
}

#[cfg(feature = "desktop")]
fn run_desktop_widget() {
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::window::WindowBuilder;
    use wry::WebViewBuilder;

    let session = std::sync::Arc::new(std::sync::Mutex::new(SessionFile::new()));

    let event_loop = EventLoopBuilder::<WidgetUserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let mut shortcut_manager = tao::global_shortcut::ShortcutManager::new(&event_loop);
    let hotkey_accel = tao::accelerator::Accelerator::new(
        Some(tao::keyboard::ModifiersState::CONTROL),
        tao::keyboard::KeyCode::KeyY,
    );
    let hotkey_result = shortcut_manager.register(hotkey_accel);
    if hotkey_result.is_err() {
        println!("[yoda-widget] WARNING: Could not register Ctrl+Y global hotkey");
    }

    let window = WindowBuilder::new()
        .with_title("Yoda Chat — PlenumNET")
        .with_inner_size(tao::dpi::LogicalSize::new(420.0, 600.0))
        .with_always_on_top(true)
        .with_decorations(true)
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create window");

    let session_clone = session.clone();
    let proxy_ipc = proxy.clone();
    let webview = WebViewBuilder::new(&window)
        .with_html(WIDGET_HTML)
        .with_initialization_script(WIDGET_JS)
        .with_ipc_handler(move |msg| {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg) {
                let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if msg_type == "dismiss" {
                    let _ = proxy_ipc.send_event(WidgetUserEvent::ToggleVisibility);
                    return;
                }
                if msg_type == "send" {
                    let message = parsed.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
                    if message.is_empty() || message.len() > MAX_MESSAGE_BYTES { return; }

                    let daemon_rep_c = match widget_query_daemon_rep_c() {
                        Some(r) => r,
                        None => {
                            let _ = proxy_ipc.send_event(WidgetUserEvent::YodaError(
                                "Daemon not reachable — is the Inter-Cube Daemon running?".to_string()
                            ));
                            return;
                        }
                    };

                    let (sequence, session_id) = {
                        let mut s = session_clone.lock().unwrap();
                        if s.is_expired() {
                            *s = SessionFile::new();
                        }
                        let seq = s.next_sequence();
                        (seq, s.session_id.clone())
                    };

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    let signing_payload = build_signing_payload(
                        &daemon_rep_c, &message, sequence, &session_id, timestamp,
                    );

                    let (signature, operator_pubkey) = match widget_ninjaexec_sign(&signing_payload) {
                        Some(r) => r,
                        None => {
                            let _ = proxy_ipc.send_event(WidgetUserEvent::YodaError(
                                "Signing failed — is NinjaExec running and unlocked?".to_string()
                            ));
                            return;
                        }
                    };

                    let payload = serde_json::json!({
                        "sessionId": session_id,
                        "timestamp": timestamp,
                        "sequence": sequence,
                        "message": message,
                        "operatorPubkey": operator_pubkey,
                        "daemonRepC": daemon_rep_c,
                        "signature": signature,
                    });

                    match widget_submit_to_daemon(&payload.to_string()) {
                        Ok(response) => {
                            if let Some(content) = response.get("content").and_then(|c| c.as_str()) {
                                let _ = proxy_ipc.send_event(WidgetUserEvent::YodaResponse(content.to_string()));
                            } else if let Some(err) = response.get("error") {
                                let err_msg = err.get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown error");
                                let _ = proxy_ipc.send_event(WidgetUserEvent::YodaError(err_msg.to_string()));
                            }
                        }
                        Err(e) => {
                            let _ = proxy_ipc.send_event(WidgetUserEvent::YodaError(
                                format!("Request failed: {}", e)
                            ));
                        }
                    }
                }
            }
        })
        .build()
        .expect("Failed to create WebView");

    println!("[yoda-widget] Desktop widget launched — Ctrl+Y to toggle visibility");

    event_loop.run(move |event, _, control_flow| {
        let _ = &shortcut_manager;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(WidgetUserEvent::YodaResponse(content)) => {
                let escaped = content
                    .replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r");
                let js = format!(
                    "if(window.__yodaReceive){{window.__yodaReceive('{}')}}", escaped
                );
                let _ = webview.evaluate_script(&js);
            }
            Event::UserEvent(WidgetUserEvent::YodaError(err_msg)) => {
                let escaped = err_msg
                    .replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('\n', "\\n");
                let js = format!(
                    "if(window.__yodaError){{window.__yodaError('{}')}}", escaped
                );
                let _ = webview.evaluate_script(&js);
            }
            Event::UserEvent(WidgetUserEvent::ToggleVisibility) => {
                let visible = window.is_visible();
                window.set_visible(!visible);
                if !visible {
                    window.set_focus();
                }
            }
            Event::GlobalShortcutEvent(_id) => {
                let visible = window.is_visible();
                window.set_visible(!visible);
                if !visible {
                    window.set_focus();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                window.set_visible(false);
            }
            _ => {}
        }
    });
}

fn main() {
    println!("PlenumNET Yoda Widget v{}", env!("CARGO_PKG_VERSION"));
    println!();

    #[cfg(feature = "desktop")]
    {
        run_desktop_widget();
        return;
    }

    #[cfg(not(feature = "desktop"))]
    {
        println!("The Ctrl+Y chat widget requires building with the `desktop` feature:");
        println!();
        println!("  cargo build -p yoda-widget --features desktop");
        println!();
        println!("Platform dependencies:");
        println!("  Windows: WebView2 (minimum version 90)");
        println!("  Linux:   webkit2gtk-4.1");
        println!();
        println!("This binary is an optional component in the daemon installer.");
        println!("The daemon and `y` CLI work without it.");
        println!();
        println!("Widget HTML template loaded ({} bytes, JS {} bytes)", WIDGET_HTML.len(), WIDGET_JS.len());
        println!();
        println!("On headless systems, this binary is not installed and the Ctrl+Y");
        println!("hotkey is not registered — this is a non-error condition.");
    }
}
