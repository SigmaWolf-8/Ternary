// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// z=0 Turntable — the distributor plane.
// Nothing permanent. Routes, never blocks. (7, 11, 13) coprime walk.
// Requests fall down (-z). Results bubble up (+z).

pub mod coprime_walk;
pub mod z_router;
pub mod sponge_rekey;

use coprime_walk::CoprimeWalker;
use z_router::{ZRouter, ZLevel, ZRequest, RequestType};

pub struct Distributor {
    walker: CoprimeWalker,
    router: ZRouter,
}

impl Distributor {
    pub fn new() -> Self {
        Self {
            walker: CoprimeWalker::with_combined_stride(),
            router: ZRouter::new(),
        }
    }

    pub fn dispatch(&mut self, origin: ZLevel, payload: RequestType) -> ZRequest {
        let ring_pos = self.walker.assign_request(0);
        self.router.route(origin, payload, ring_pos)
    }

    pub fn requests_processed(&self) -> u64 {
        self.router.requests_processed()
    }

    pub fn walker_position(&self) -> u32 {
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
        assert_eq!(req.ring_position, dist.walker_position().wrapping_sub(0));
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
}
