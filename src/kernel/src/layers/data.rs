// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// z=−5: Data Layer — PlenumDB, backend databases, blockchain ledger state.

use crate::distributor::z_router::ZLevel;
use super::{Layer, LayerStatus, LayerResult};

pub struct DataLayer {
    status: LayerStatus,
}

impl DataLayer {
    pub fn new() -> Self {
        Self { status: LayerStatus::Active }
    }
}

impl Layer for DataLayer {
    fn z_level(&self) -> ZLevel { ZLevel::DATA_LAYER }
    fn name(&self) -> &str { "Data Layer (PlenumDB, blockchain state)" }
    fn status(&self) -> LayerStatus { self.status }
    fn handle_request(&mut self, payload: &[u8]) -> LayerResult {
        if self.status != LayerStatus::Active {
            return LayerResult::offline();
        }
        LayerResult::ok(payload.to_vec())
    }
}
