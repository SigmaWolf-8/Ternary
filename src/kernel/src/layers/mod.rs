// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Z-Level Layer Implementations
// +z (above ground) = presentation, -z (below ground) = processing.

pub mod gateway;
pub mod services;
pub mod conventional;
pub mod ternary_native;
pub mod data;
pub mod infrastructure;
pub mod fileserver;
pub mod snapshots;

use alloc::string::String;

use crate::distributor::z_router::ZLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerStatus {
    Active,
    Degraded,
    Offline,
    Initializing,
}

pub trait Layer {
    fn z_level(&self) -> ZLevel;
    fn name(&self) -> &str;
    fn status(&self) -> LayerStatus;
    fn handle_request(&mut self, payload: &[u8]) -> LayerResult;
}

#[derive(Debug, Clone)]
pub struct LayerResult {
    pub status: LayerStatus,
    pub response: Option<alloc::vec::Vec<u8>>,
    pub error: Option<String>,
}

impl LayerResult {
    pub fn ok(data: alloc::vec::Vec<u8>) -> Self {
        Self { status: LayerStatus::Active, response: Some(data), error: None }
    }

    pub fn err(msg: String) -> Self {
        Self { status: LayerStatus::Degraded, response: None, error: Some(msg) }
    }

    pub fn offline() -> Self {
        Self { status: LayerStatus::Offline, response: None, error: None }
    }
}
