// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayEnvelope {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(rename = "publicKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(rename = "msgType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay_msg_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered: Option<bool>,
    #[serde(rename = "connectedPeers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_peers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected: Option<Vec<String>>,
}

pub type IncomingRx = mpsc::Receiver<RelayEnvelope>;
pub type OutgoingTx = mpsc::Sender<RelayEnvelope>;

pub struct WsRelayClient {
    pub outgoing_tx: OutgoingTx,
    pub connected: Arc<Mutex<bool>>,
    pub peers: Arc<Mutex<Vec<String>>>,
}

impl WsRelayClient {
    pub async fn connect(
        crs_url: &str,
        address: &str,
        public_key: &str,
    ) -> Result<(Self, IncomingRx), String> {
        Self::connect_signed(crs_url, address, public_key, None).await
    }

    pub async fn connect_signed(
        crs_url: &str,
        address: &str,
        public_key: &str,
        tl_dsa_secret_key: Option<&[u8]>,
    ) -> Result<(Self, IncomingRx), String> {
        let ws_url = crs_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let ws_url = format!("{}/ws/relay", ws_url.trim_end_matches('/'));

        println!("[ws-relay] Connecting to {}", ws_url);

        let url = url::Url::parse(&ws_url).map_err(|e| format!("Invalid URL: {}", e))?;

        let (ws_stream, _response) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {}", e))?;

        println!("[ws-relay] WebSocket connected, authenticating (challenge-response)...");

        let (mut write, mut read) = ws_stream.split();

        let challenge_response = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            read.next(),
        )
        .await
        .map_err(|_| "Challenge timeout — server did not send nonce".to_string())?
        .ok_or("Connection closed before challenge")?
        .map_err(|e| format!("Read error: {}", e))?;

        let challenge_text = challenge_response
            .to_text()
            .map_err(|e| format!("Non-text challenge: {}", e))?;
        let challenge_env: RelayEnvelope =
            serde_json::from_str(challenge_text).map_err(|e| format!("Parse error: {}", e))?;

        if challenge_env.msg_type != "challenge" {
            return Err(format!("Expected challenge, got: {}", challenge_env.msg_type));
        }

        let nonce = challenge_env.nonce.ok_or("Challenge missing nonce field")?;
        println!("[ws-relay] Received challenge nonce ({}...)", &nonce[..std::cmp::min(16, nonce.len())]);

        let challenge_payload = format!("{}||{}||{}", nonce, address, public_key);

        let auth_msg = if let Some(sk) = tl_dsa_secret_key {
            let sig_bytes = ternary_math::tl_dsa::sign(
                sk,
                challenge_payload.as_bytes(),
                ternary_math::tl_dsa::TlDsaVariant::TlDsa87,
            );
            let sig_hex: String = sig_bytes.iter().map(|b| format!("{:02x}", b)).collect();
            println!("[ws-relay] Challenge signed with TL-DSA-87 ({} bytes)", sig_bytes.len());
            serde_json::json!({
                "type": "auth",
                "address": address,
                "publicKey": public_key,
                "nonce": nonce,
                "signature": sig_hex,
            })
        } else {
            println!("[ws-relay] WARNING: No secret key — sending unsigned auth (legacy mode)");
            serde_json::json!({
                "type": "auth",
                "address": address,
                "publicKey": public_key,
                "nonce": nonce,
            })
        };

        write
            .send(Message::Text(auth_msg.to_string()))
            .await
            .map_err(|e| format!("Failed to send auth: {}", e))?;

        let auth_response = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            read.next(),
        )
        .await
        .map_err(|_| "Auth timeout".to_string())?
        .ok_or("Connection closed before auth response")?
        .map_err(|e| format!("Read error: {}", e))?;

        let auth_text = auth_response
            .to_text()
            .map_err(|e| format!("Non-text response: {}", e))?;
        let auth_env: RelayEnvelope =
            serde_json::from_str(auth_text).map_err(|e| format!("Parse error: {}", e))?;

        if auth_env.msg_type != "auth_ok" {
            return Err(format!(
                "Auth failed: {}",
                auth_env.error.unwrap_or_default()
            ));
        }

        let initial_peers = auth_env.connected_peers.unwrap_or_default();
        println!(
            "[ws-relay] Authenticated. {} peer(s) online: {:?}",
            initial_peers.len(),
            initial_peers
        );

        let connected = Arc::new(Mutex::new(true));
        let peers = Arc::new(Mutex::new(initial_peers));

        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<RelayEnvelope>(64);
        let (incoming_tx, incoming_rx) = mpsc::channel::<RelayEnvelope>(64);

        let connected_write = connected.clone();
        tokio::spawn(async move {
            while let Some(envelope) = outgoing_rx.recv().await {
                let json = match serde_json::to_string(&envelope) {
                    Ok(j) => j,
                    Err(e) => {
                        println!("[ws-relay] Serialize error: {}", e);
                        continue;
                    }
                };
                if write.send(Message::Text(json)).await.is_err() {
                    println!("[ws-relay] Write failed, connection lost");
                    *connected_write.lock().await = false;
                    break;
                }
            }
        });

        let connected_read = connected.clone();
        let peers_read = peers.clone();
        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<RelayEnvelope>(&text) {
                            Ok(env) => {
                                if env.msg_type == "peers" {
                                    if let Some(ref list) = env.connected {
                                        *peers_read.lock().await = list.clone();
                                    }
                                } else if env.msg_type == "pong" {
                                    // keepalive ack
                                } else if env.msg_type == "restart" {
                                    println!("[ws-relay] Restart command received from CRS");
                                    let _ = incoming_tx.send(env).await;
                                    *connected_read.lock().await = false;
                                    break;
                                } else if env.msg_type == "relay" {
                                    if incoming_tx.send(env).await.is_err() {
                                        println!("[ws-relay] Incoming channel closed");
                                        break;
                                    }
                                } else if env.msg_type == "relay_ack" {
                                    if let Some(delivered) = env.delivered {
                                        if !delivered {
                                            println!(
                                                "[ws-relay] Message to {} queued (peer offline)",
                                                env.to.unwrap_or_default()
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("[ws-relay] Parse error: {} — raw: {}", e, text);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("[ws-relay] Server closed connection");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        // tungstenite handles pong automatically
                        let _ = data;
                    }
                    Err(e) => {
                        println!("[ws-relay] Read error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            *connected_read.lock().await = false;
            println!("[ws-relay] Read loop ended");
        });

        let client = WsRelayClient {
            outgoing_tx,
            connected,
            peers,
        };

        Ok((client, incoming_rx))
    }

    pub async fn send_to(&self, to: &str, msg_type: &str, payload: &str) -> Result<(), String> {
        let envelope = RelayEnvelope {
            msg_type: "relay".to_string(),
            to: Some(to.to_string()),
            relay_msg_type: Some(msg_type.to_string()),
            payload: Some(payload.to_string()),
            address: None,
            public_key: None,
            nonce: None,
            signature: None,
            from: None,
            error: None,
            delivered: None,
            connected_peers: None,
            ts: None,
            connected: None,
        };
        self.outgoing_tx
            .send(envelope)
            .await
            .map_err(|e| format!("Send failed: {}", e))
    }

    pub async fn ping(&self) -> Result<(), String> {
        let envelope = RelayEnvelope {
            msg_type: "ping".to_string(),
            address: None,
            public_key: None,
            nonce: None,
            signature: None,
            to: None,
            from: None,
            relay_msg_type: None,
            payload: None,
            error: None,
            delivered: None,
            connected_peers: None,
            ts: None,
            connected: None,
        };
        self.outgoing_tx
            .send(envelope)
            .await
            .map_err(|e| format!("Ping failed: {}", e))
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    pub async fn peer_list(&self) -> Vec<String> {
        self.peers.lock().await.clone()
    }
}
