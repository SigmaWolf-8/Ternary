// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Network requests → z=0 distributor (kernel internal call).
// The browser at z=+1 routes through z=0 to reach services.
// No IPC, no shared memory — direct function calls in kernel space.

use alloc::string::String;
use alloc::vec::Vec;
use crate::distributor::Distributor;
use crate::distributor::z_router::{ZLevel, ZRequest, RequestType, RouteDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Document,
    Script,
    Stylesheet,
    Image,
    Font,
    Xhr,
    Fetch,
    WebSocket,
    Other,
}

impl ResourceType {
    pub fn to_z_request_type(&self) -> RequestType {
        match self {
            ResourceType::Document => RequestType::HttpRequest,
            ResourceType::Script => RequestType::ScriptExec,
            ResourceType::Stylesheet => RequestType::FileServe,
            ResourceType::Image => RequestType::FileServe,
            ResourceType::Font => RequestType::FileServe,
            ResourceType::Xhr | ResourceType::Fetch => RequestType::HttpRequest,
            ResourceType::WebSocket => RequestType::HttpRequest,
            ResourceType::Other => RequestType::HttpRequest,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub id: u64,
    pub url: String,
    pub resource_type: ResourceType,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub z_request: Option<ZRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
}

#[derive(Debug, Clone)]
pub struct ResourceResponse {
    pub request_id: u64,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub content_type: String,
}

pub struct NetworkLayer {
    request_counter: u64,
    pending: Vec<u64>,
    distributor: Distributor,
}

impl NetworkLayer {
    pub fn new() -> Self {
        Self {
            request_counter: 0,
            pending: Vec::new(),
            distributor: Distributor::new(),
        }
    }

    pub fn fetch(&mut self, url: String, resource_type: ResourceType) -> ResourceRequest {
        self.request_counter += 1;

        let z_payload = resource_type.to_z_request_type();
        let z_req = self.distributor.dispatch(ZLevel::UI, z_payload);

        let req = ResourceRequest {
            id: self.request_counter,
            url,
            resource_type,
            method: HttpMethod::Get,
            headers: Vec::new(),
            z_request: Some(z_req),
        };
        self.pending.push(req.id);
        req
    }

    pub fn complete(&mut self, request_id: u64) {
        self.pending.retain(|&id| id != request_id);
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn requests_made(&self) -> u64 {
        self.request_counter
    }

    pub fn distributor_position(&self) -> u32 {
        self.distributor.walker_position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_fetch_routes_through_distributor() {
        let mut net = NetworkLayer::new();
        let req = net.fetch("https://example.com".into(), ResourceType::Document);
        assert_eq!(req.id, 1);
        assert_eq!(net.pending_count(), 1);

        let z_req = req.z_request.as_ref().unwrap();
        assert_eq!(z_req.origin, ZLevel::UI);
        assert_eq!(z_req.target, ZLevel::API_GATEWAY);
        assert_eq!(z_req.direction, RouteDirection::Falling);
    }

    #[test]
    fn test_network_fetch_fileserve_routes() {
        let mut net = NetworkLayer::new();
        let req = net.fetch("font.woff2".into(), ResourceType::Font);
        let z_req = req.z_request.as_ref().unwrap();
        assert_eq!(z_req.target, ZLevel::FILE_SERVER);
        assert_eq!(z_req.direction, RouteDirection::Bubbling);
    }

    #[test]
    fn test_network_complete() {
        let mut net = NetworkLayer::new();
        let req = net.fetch("https://example.com/style.css".into(), ResourceType::Stylesheet);
        net.complete(req.id);
        assert_eq!(net.pending_count(), 0);
    }

    #[test]
    fn test_request_counter_and_distributor() {
        let mut net = NetworkLayer::new();
        net.fetch("a".into(), ResourceType::Script);
        net.fetch("b".into(), ResourceType::Image);
        net.fetch("c".into(), ResourceType::Font);
        assert_eq!(net.requests_made(), 3);
        assert!(net.distributor_position() > 0);
    }

    #[test]
    fn test_coprime_walk_advances() {
        let mut net = NetworkLayer::new();
        let r1 = net.fetch("a".into(), ResourceType::Document);
        let r2 = net.fetch("b".into(), ResourceType::Document);
        let pos1 = r1.z_request.unwrap().ring_position;
        let pos2 = r2.z_request.unwrap().ring_position;
        assert_ne!(pos1, pos2, "coprime walk must advance to different positions");
    }
}
