// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// z=−6: Infrastructure / Crypto — TLSponge-385, TL-DSA, TIS-27,
// TL-KEM, TDNS, key management. Only layer touching raw network.

use crate::distributor::z_router::ZLevel;
use super::{Layer, LayerStatus, LayerResult};

pub struct InfrastructureLayer {
    status: LayerStatus,
}

impl InfrastructureLayer {
    pub fn new() -> Self {
        Self { status: LayerStatus::Active }
    }
}

impl Layer for InfrastructureLayer {
    fn z_level(&self) -> ZLevel { ZLevel::INFRASTRUCTURE }
    fn name(&self) -> &str { "Infrastructure (TLSponge-385, TL-DSA, TDNS)" }
    fn status(&self) -> LayerStatus { self.status }
    fn handle_request(&mut self, payload: &[u8]) -> LayerResult {
        if self.status != LayerStatus::Active {
            return LayerResult::offline();
        }
        LayerResult::ok(payload.to_vec())
    }
}
