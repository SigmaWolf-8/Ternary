// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// z=+2: File server — static files, assets, media, web fonts, documents.

use crate::distributor::z_router::ZLevel;
use super::{Layer, LayerStatus, LayerResult};

pub struct FileServerLayer {
    status: LayerStatus,
}

impl FileServerLayer {
    pub fn new() -> Self {
        Self { status: LayerStatus::Active }
    }
}

impl Layer for FileServerLayer {
    fn z_level(&self) -> ZLevel { ZLevel::FILE_SERVER }
    fn name(&self) -> &str { "File Server (static assets, media, web fonts)" }
    fn status(&self) -> LayerStatus { self.status }
    fn handle_request(&mut self, payload: &[u8]) -> LayerResult {
        if self.status != LayerStatus::Active {
            return LayerResult::offline();
        }
        LayerResult::ok(payload.to_vec())
    }
}
