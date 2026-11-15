// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
//! Publish actual local inventory to the same destination as the execution relay.
//! This task must not depend on a successful CRS heartbeat.
use std::time::Duration;

fn report_url(target: &str) -> Result<reqwest::Url, &'static str> {
    let mut url = reqwest::Url::parse(target).map_err(|_| "invalid relay URL")?;
    let scheme = match url.scheme() {
        "ws" | "http" => "http",
        "wss" | "https" => "https",
        _ => return Err("relay URL must use HTTP(S) or WS(S)"),
    };
    url.set_scheme(scheme).map_err(|_| "invalid relay URL scheme")?;
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("relay URL must have a host and no credentials, query or fragment");
    }
    let path = url.path().trim_end_matches('/');
    let base = path.strip_suffix("/ws/relay").unwrap_or(path);
    url.set_path(&format!("{base}/api/salvi/inter-cube/relay/slot-report"));
    Ok(url)
}

async fn publish_once(
    client: &reqwest::Client,
    node_id: u8,
    api_port: u16,
    target: &reqwest::Url,
    local_key: Option<&str>,
) -> Result<(), String> {
    let mut observations = Vec::new();
    for path in ["/api/salvi/inter-cube/slots", "/health"] {
        let mut request = client.get(format!("http://127.0.0.1:{api_port}{path}"));
        if let Some(key) = local_key {
            request = request.bearer_auth(key);
        }
        let response = request.send().await.map_err(|_| format!("local {path}: request failed or timed out"))?;
        if !response.status().is_success() {
            return Err(format!("local {path}: HTTP {}", response.status()));
        }
        let body: serde_json::Value = response.json().await
            .map_err(|_| format!("local {path}: invalid JSON"))?;
        if body["node_id"].as_u64() != Some(u64::from(node_id)) {
            return Err(format!("local {path}: node identity mismatch; report withheld"));
        }
        observations.push(body);
    }
    if !observations[0]["slots"].as_array().is_some_and(|slots| slots.len() == 27) {
        return Err("local inventory: expected 27 actual slot records; report withheld".into());
    }
    // The local daemon credential is deliberately not forwarded to the relay.
    let response = client.post(target.clone()).json(&serde_json::json!({
        "nodeId": node_id,
        "slots": observations[0],
        "health": observations[1],
    })).send().await.map_err(|_| "relay inventory upload failed or timed out".to_string())?;
    if !response.status().is_success() {
        return Err(format!("relay inventory upload: HTTP {}", response.status()));
    }
    Ok(())
}

pub fn spawn(node_id: u8, api_port: u16, relay_target: &str) {
    let target = match report_url(relay_target) {
        Ok(target) => target,
        Err(error) => {
            eprintln!("[slot-report] Node {node_id}: {error}; reporter not started");
            return;
        }
    };
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .connect_timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(format!("PlenumNET-InterCube/{}", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            eprintln!("[slot-report] Node {node_id}: could not build HTTP client");
            return;
        }
    };
    let local_key = std::env::var("PLENUM_API_KEY").ok().filter(|key| !key.is_empty());
    tokio::spawn(async move {
        // Give the local HTTP listener time to bind on startup.
        tokio::time::sleep(Duration::from_secs(5)).await;
        loop {
            match publish_once(&client, node_id, api_port, &target, local_key.as_deref()).await {
                Ok(()) => println!("[slot-report] Node {node_id}: inventory accepted by relay"),
                Err(error) => eprintln!("[slot-report] Node {node_id}: {error}"),
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::{HeaderMap, StatusCode}, routing::{get, post}, Json, Router};

    async fn serve(router: Router) -> (u16, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
        (port, task)
    }

    fn local_api(node: u8) -> Router {
        Router::new()
            .route("/health", get(move || async move {
                Json(serde_json::json!({"node_id":node,"status":"ok"}))
            }))
            .route("/api/salvi/inter-cube/slots", get(move |headers: HeaderMap| async move {
                assert_eq!(headers.get("authorization").unwrap(), "Bearer local-test-key");
                Json(serde_json::json!({"node_id":node,"slots":
                    (0..27).map(|_| serde_json::json!({"service_type":null,"status":"available"})).collect::<Vec<_>>()
                }))
            }))
    }

    #[tokio::test]
    async fn posts_unchanged_inventory_to_separate_relay_without_local_credential() {
        let (api_port, api_task) = serve(local_api(3)).await;
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let relay = Router::new().route("/api/salvi/inter-cube/relay/slot-report",
            post(move |headers: HeaderMap, Json(payload): Json<serde_json::Value>| {
                let tx = tx.clone();
                async move {
                    assert!(headers.get("authorization").is_none());
                    tx.send(payload).await.unwrap();
                    StatusCode::OK
                }
            }));
        let (relay_port, relay_task) = serve(relay).await;
        let target = report_url(&format!("ws://127.0.0.1:{relay_port}/ws/relay")).unwrap();
        publish_once(&reqwest::Client::new(), 3, api_port, &target, Some("local-test-key")).await.unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received["nodeId"], 3);
        assert_eq!(received["slots"]["node_id"], 3);
        assert_eq!(received["slots"]["slots"].as_array().unwrap().len(), 27);
        assert!(received["slots"]["slots"].as_array().unwrap().iter().all(|s| s["service_type"].is_null()));
        api_task.abort();
        relay_task.abort();
    }

    #[tokio::test]
    async fn surfaces_relay_http_failure() {
        let (api_port, api_task) = serve(local_api(2)).await;
        let (relay_port, relay_task) = serve(Router::new().route(
            "/api/salvi/inter-cube/relay/slot-report", post(|| async { StatusCode::SERVICE_UNAVAILABLE })
        )).await;
        let target = report_url(&format!("http://127.0.0.1:{relay_port}")).unwrap();
        let error = publish_once(&reqwest::Client::new(), 2, api_port, &target, Some("local-test-key"))
            .await.unwrap_err();
        assert!(error.contains("503"), "{error}");
        api_task.abort();
        relay_task.abort();
    }

    #[tokio::test]
    async fn withholds_wrong_node_inventory() {
        let (api_port, api_task) = serve(local_api(1)).await;
        let target = report_url("http://127.0.0.1:1").unwrap();
        let error = publish_once(&reqwest::Client::new(), 2, api_port, &target, Some("local-test-key"))
            .await.unwrap_err();
        assert!(error.contains("node identity mismatch"), "{error}");
        api_task.abort();
    }

    #[tokio::test]
    async fn surfaces_local_auth_failure_without_publishing() {
        let (api_port, api_task) = serve(Router::new().route(
            "/api/salvi/inter-cube/slots", get(|| async { StatusCode::UNAUTHORIZED })
        )).await;
        let target = report_url("http://127.0.0.1:1").unwrap();
        let error = publish_once(&reqwest::Client::new(), 2, api_port, &target, None)
            .await.unwrap_err();
        assert!(error.contains("401"), "{error}");
        api_task.abort();
    }

    #[tokio::test]
    async fn timeout_prevents_a_local_hang_from_blocking_future_reports() {
        let (api_port, api_task) = serve(Router::new().route(
            "/api/salvi/inter-cube/slots", get(|| async {
                tokio::time::sleep(Duration::from_secs(60)).await;
                StatusCode::OK
            })
        )).await;
        let client = reqwest::Client::builder().timeout(Duration::from_millis(30)).build().unwrap();
        let target = report_url("http://127.0.0.1:1").unwrap();
        let result = tokio::time::timeout(Duration::from_secs(2),
            publish_once(&client, 2, api_port, &target, None)).await.unwrap();
        assert!(result.unwrap_err().contains("timed out"));
        api_task.abort();
    }

    #[test]
    fn normalizes_relay_transport_and_websocket_path() {
        for target in [
            "https://relay.example", "https://relay.example/",
            "https://relay.example/ws/relay", "wss://relay.example/ws/relay/",
        ] {
            assert_eq!(report_url(target).unwrap().as_str(),
                "https://relay.example/api/salvi/inter-cube/relay/slot-report");
        }
        assert_eq!(report_url("ws://127.0.0.1:1234/ws/relay").unwrap().as_str(),
            "http://127.0.0.1:1234/api/salvi/inter-cube/relay/slot-report");
        assert_eq!(report_url("https://relay.example/prefix/ws/relay").unwrap().as_str(),
            "https://relay.example/prefix/api/salvi/inter-cube/relay/slot-report");
    }

    #[test]
    fn rejects_ambiguous_or_credentialed_targets() {
        for target in ["ftp://relay.example", "not a URL", "https://u:p@relay.example",
            "https://relay.example?token=test", "https://relay.example#fragment"] {
            assert!(report_url(target).is_err(), "{target}");
        }
    }
}
