// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
//! PlenumNET daemon adapter for explicit logical query channels.
//! Copy alongside query_channels.rs into the daemon source tree.
use std::{sync::Arc, time::Duration};
use serde_json::{json, Value};
use crate::query_channels::{Binding, Pool};

pub struct Runtime {
    pool: Arc<Pool>,
    http: reqwest::Client,
    base: String,
}

impl Runtime {
    pub fn new(node: u8, base: String, capacity: usize) -> Result<Arc<Self>, String> {
        let pool=Pool::new(node, uuid::Uuid::new_v4().simple().to_string(), capacity)
            .map_err(str::to_owned)?;
        let http=reqwest::Client::builder().connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(120)).redirect(reqwest::redirect::Policy::none())
            .build().map_err(|e|e.to_string())?;
        Ok(Arc::new(Self {pool,http,base}))
    }

    pub async fn handle(&self, kind: &str, request: Value) -> (&'static str, Value) {
        let id=request["requestId"].as_str().unwrap_or("");
        let result=tokio::time::timeout(Duration::from_secs(120), self.execute(kind,&request)).await;
        match result {
            Ok(Ok(value)) => (if kind=="query_channels_request" {"query_channels_response"} else {"channel_inference_response"}, value),
            Ok(Err(error)) => ("channel_inference_error",json!({"requestId":id,"error":error})),
            Err(_) => ("channel_inference_error",json!({"requestId":id,"error":"Model request timed out; outcome unknown, do not automatically resend"})),
        }
    }

    async fn execute(&self, kind: &str, request: &Value) -> Result<Value,String> {
        let id=request["requestId"].as_str().filter(|s| !s.is_empty() && s.len()<=128)
            .ok_or("Missing request identity")?;
        if kind=="query_channels_request" {
            let response=self.http.get(format!("{}/v1/models",self.base))
                .timeout(Duration::from_secs(5)).send().await.map_err(|_|"Model inventory unavailable")?;
            if !response.status().is_success() {return Err("Model inventory HTTP failure".into());}
            let body:Value=response.json().await.map_err(|_|"Invalid model inventory")?;
            let models:Vec<&str>=body["data"].as_array().ok_or("Missing model inventory")?
                .iter().filter_map(|v|v["id"].as_str()).filter(|s|!s.is_empty()).collect();
            if models.is_empty() {return Err("No loaded model advertised".into());}
            return Ok(json!({
                "requestId":id,"protocol":1,"transport":"logical multiplexing over authenticated relay",
                "modelCapacity":self.pool.capacity(),"models":models,
                "channels":self.pool.snapshot().map_err(str::to_owned)?,
                "generationVerified":false
            }));
        }
        if kind!="channel_inference_request" {return Err("Unsupported channel operation".into());}
        let binding:Binding=serde_json::from_value(request["binding"].clone()).map_err(|_|"Invalid binding")?;
        let model=request["model"].as_str().filter(|s|!s.is_empty()).ok_or("Model required")?;
        let tokens=request["maxTokens"].as_u64().filter(|n|(1..=4096).contains(n)).ok_or("Token limit must be 1..4096")?;
        let messages=request["messages"].as_array().filter(|v|!v.is_empty()).ok_or("Messages required")?;
        if serde_json::to_vec(messages).map_err(|_|"Invalid messages")?.len()>1048576 {
            return Err("Input exceeds channel byte limit".into());
        }
        let mut permit=self.pool.admit(&binding,id).map_err(str::to_owned)?;
        let response=self.http.post(format!("{}/v1/chat/completions",self.base))
            .json(&json!({"model":model,"messages":messages,"max_tokens":tokens,
                "temperature":0.0,"stream":false}))
            .send().await.map_err(|_|"Model transport failed; outcome unknown")?;
        if !response.status().is_success() {return Err(format!("Model HTTP {}",response.status()));}
        let body:Value=response.json().await.map_err(|_|"Invalid model response")?;
        let actual_model=body["model"].as_str().ok_or("Model response omitted identity")?;
        if actual_model!=model {return Err("Responding model differs from requested model".into());}
        let content=body["choices"][0]["message"]["content"].as_str().ok_or("Missing model answer")?;
        if body["choices"][0]["finish_reason"].as_str()==Some("length") {return Err("Model answer truncated".into());}
        permit.complete(content).map_err(str::to_owned)?;
        Ok(json!({"requestId":id,"binding":binding,"content":content,"model":actual_model,
            "usage":body["usage"],"finish_reason":body["choices"][0]["finish_reason"]}))
    }
}
