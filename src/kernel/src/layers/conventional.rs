// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// z=−3: Conventional Apps — Windows via Wine, Linux native, Mac via Darling.
// Sandboxed containers.

use crate::distributor::z_router::ZLevel;
use super::{Layer, LayerStatus, LayerResult};

pub struct ConventionalLayer {
    status: LayerStatus,
}

impl ConventionalLayer {
    pub fn new() -> Self {
        Self { status: LayerStatus::Active }
    }
}

impl Layer for ConventionalLayer {
    fn z_level(&self) -> ZLevel { ZLevel::CONVENTIONAL }
    fn name(&self) -> &str { "Conventional Apps (Wine/Darling sandboxed)" }
    fn status(&self) -> LayerStatus { self.status }
    fn handle_request(&mut self, payload: &[u8]) -> LayerResult {
        if self.status != LayerStatus::Active {
            return LayerResult::offline();
        }
        LayerResult::ok(payload.to_vec())
    }
}
