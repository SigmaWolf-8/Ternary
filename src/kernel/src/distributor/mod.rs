// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// z=0 Turntable — the distributor plane.
// Nothing permanent. Routes, never blocks. (7, 11, 13) coprime walk.
// Requests fall down (-z). Results bubble up (+z).
//
// Import enforcement: browser at z=+1 imports ONLY from
// crate::distributor::RequestInterface. Zero imports from
// crate::layers::*, crate::crypto::*, crate::network::*.

pub mod coprime_walk;
pub mod z_router;
pub mod sponge_rekey;

use coprime_walk::CoprimeWalker;
use z_router::{ZRouter, ZLevel, ZRequest, ZResponse, RequestType, RouteDirection, RouteStatus};
use alloc::vec::Vec;

pub trait RequestInterface {
    fn submit_request(&mut self, payload_type: RequestType) -> RequestResult;
    fn query_status(&self, request_id: u64) -> Option<RouteStatus>;
    fn requests_processed(&self) -> u64;
    fn walker_position(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct RequestResult {
    pub request_id: u64,
    pub target: ZLevel,
    pub ring_position: u32,
    pub direction: RouteDirection,
    pub status: RouteStatus,
}

pub struct Distributor {
    walker: CoprimeWalker,
    router: ZRouter,
    completed_requests: Vec<(u64, RouteStatus)>,
}

impl Distributor {
    pub fn new() -> Self {
        Self {
            walker: CoprimeWalker::with_combined_stride(),
            router: ZRouter::new(),
            completed_requests: Vec::new(),
        }
    }

    pub fn dispatch(&mut self, origin: ZLevel, payload: RequestType) -> ZRequest {
        let ring_pos = self.walker.assign_request(0);
        let req = self.router.route(origin, payload, ring_pos);
        self.completed_requests.push((req.id, RouteStatus::Ok));
        req
    }

    pub fn dispatch_response(&self, request: &ZRequest) -> ZResponse {
        ZResponse {
            request_id: request.id,
            origin: request.target,
            target: request.origin,
            direction: RouteDirection::Bubbling,
            status: RouteStatus::Ok,
        }
    }
}

impl RequestInterface for Distributor {
    fn submit_request(&mut self, payload_type: RequestType) -> RequestResult {
        let req = self.dispatch(ZLevel::UI, payload_type);
        RequestResult {
            request_id: req.id,
            target: req.target,
            ring_position: req.ring_position,
            direction: req.direction,
            status: RouteStatus::Ok,
        }
    }

    fn query_status(&self, request_id: u64) -> Option<RouteStatus> {
        self.completed_requests.iter()
            .find(|(id, _)| *id == request_id)
            .map(|(_, status)| *status)
    }

    fn requests_processed(&self) -> u64 {
        self.router.requests_processed()
    }

    fn walker_position(&self) -> u32 {
        self.walker.position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distributor_dispatch() {
        let mut dist = Distributor::new();
        let req = dist.dispatch(ZLevel::UI, RequestType::DataQuery);
        assert_eq!(req.target, ZLevel::DATA_LAYER);
        assert_eq!(dist.requests_processed(), 1);
    }

    #[test]
    fn test_distributor_multiple() {
        let mut dist = Distributor::new();
        for _ in 0..100 {
            dist.dispatch(ZLevel::UI, RequestType::HttpRequest);
        }
        assert_eq!(dist.requests_processed(), 100);
    }

    #[test]
    fn test_request_interface() {
        let mut dist = Distributor::new();
        let result = dist.submit_request(RequestType::DataQuery);
        assert_eq!(result.target, ZLevel::DATA_LAYER);
        assert_eq!(result.status, RouteStatus::Ok);
        assert_eq!(result.request_id, 1);

        let status = dist.query_status(1);
        assert_eq!(status, Some(RouteStatus::Ok));

        assert_eq!(dist.requests_processed(), 1);
    }

    #[test]
    fn test_request_interface_multiple() {
        let mut dist = Distributor::new();
        let r1 = dist.submit_request(RequestType::HttpRequest);
        let r2 = dist.submit_request(RequestType::CryptoOp);
        let r3 = dist.submit_request(RequestType::FileServe);

        assert_eq!(r1.target, ZLevel::API_GATEWAY);
        assert_eq!(r2.target, ZLevel::INFRASTRUCTURE);
        assert_eq!(r3.target, ZLevel::FILE_SERVER);

        assert_eq!(dist.requests_processed(), 3);
        assert!(dist.walker_position() > 0);
    }

    #[test]
    fn test_dispatch_response() {
        let mut dist = Distributor::new();
        let req = dist.dispatch(ZLevel::UI, RequestType::DataQuery);
        let resp = dist.dispatch_response(&req);
        assert_eq!(resp.request_id, req.id);
        assert_eq!(resp.origin, ZLevel::DATA_LAYER);
        assert_eq!(resp.target, ZLevel::UI);
        assert_eq!(resp.direction, RouteDirection::Bubbling);
        assert_eq!(resp.status, RouteStatus::Ok);
    }
}
