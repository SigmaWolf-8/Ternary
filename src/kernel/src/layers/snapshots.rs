// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// z=+3..+n: Snapshots — monitoring and archive.
// z-trit encodes date for deep archive (+n).

use crate::distributor::z_router::ZLevel;
use super::{Layer, LayerStatus, LayerResult};

pub struct SnapshotLayer {
    z_level: i8,
    status: LayerStatus,
}

impl SnapshotLayer {
    pub fn new(z_level: i8) -> Self {
        let z = if z_level < 3 { 3 } else { z_level };
        Self {
            z_level: z,
            status: LayerStatus::Active,
        }
    }
}

impl Layer for SnapshotLayer {
    fn z_level(&self) -> ZLevel { ZLevel::new(self.z_level) }
    fn name(&self) -> &str { "Snapshot (monitoring/archive)" }
    fn status(&self) -> LayerStatus { self.status }
    fn handle_request(&mut self, payload: &[u8]) -> LayerResult {
        if self.status != LayerStatus::Active {
            return LayerResult::offline();
        }
        LayerResult::ok(payload.to_vec())
    }
}
