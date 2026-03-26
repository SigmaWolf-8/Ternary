// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumBrowser Kernel Subsystem — Pipeline Orchestrator
// Not a fork. Not userspace. Kernel subsystem with direct access to
// GPU, ternary cryptographic stack, and z=0 distributor.
//
// Pipeline: parse → layout → script → render (CPU/GPU) → encrypt → display
//
// Phase 1: CPU rendering path via simplefb/tiny-skia fallback.
// Phase 2: GPU rendering with IOMMU-isolated VRAM.

pub mod parse;
pub mod layout;
pub mod script;
pub mod render_cpu;
pub mod tabs;
pub mod input;
pub mod net;
pub mod mesh;
pub mod color;

use alloc::string::String;
use tabs::{TabManager, TabError};
use script::ScriptExecutor;
use render_cpu::{CpuRenderer, CpuFramebuffer};
use net::NetworkLayer;

pub struct Browser {
    tab_manager: TabManager,
    renderer: CpuRenderer,
    framebuffer: CpuFramebuffer,
    network: NetworkLayer,
    pipeline_state: PipelineState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    Idle,
    Parsing,
    Layout,
    Scripting,
    Rendering,
    Encrypting,
    DisplayReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    Cpu,
    Gpu,
}

impl Browser {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            tab_manager: TabManager::new(),
            renderer: CpuRenderer::new(),
            framebuffer: CpuFramebuffer::new(width, height),
            network: NetworkLayer::new(),
            pipeline_state: PipelineState::Idle,
        }
    }

    pub fn open_tab(&mut self, url: String) -> Result<u32, TabError> {
        self.tab_manager.open_tab(url)
    }

    pub fn close_tab(&mut self, id: u32) -> Result<(), TabError> {
        self.tab_manager.close_tab(id)
    }

    pub fn tab_count(&self) -> usize {
        self.tab_manager.tab_count()
    }

    pub fn pipeline_state(&self) -> PipelineState {
        self.pipeline_state
    }

    pub fn render_backend(&self) -> RenderBackend {
        RenderBackend::Cpu
    }

    pub fn framebuffer(&self) -> &CpuFramebuffer {
        &self.framebuffer
    }

    pub fn framebuffer_mut(&mut self) -> &mut CpuFramebuffer {
        &mut self.framebuffer
    }

    pub fn flush_render(&mut self) {
        self.pipeline_state = PipelineState::Rendering;
        self.renderer.flush(&mut self.framebuffer);
        self.pipeline_state = PipelineState::DisplayReady;
    }

    pub fn encrypt_framebuffer(&mut self, keystream: &[u8]) {
        self.pipeline_state = PipelineState::Encrypting;
        render_cpu::sponge_encrypt_framebuffer(&mut self.framebuffer, keystream);
        self.pipeline_state = PipelineState::DisplayReady;
    }

    pub fn network(&mut self) -> &mut NetworkLayer {
        &mut self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_creation() {
        let browser = Browser::new(1920, 1080);
        assert_eq!(browser.pipeline_state(), PipelineState::Idle);
        assert_eq!(browser.render_backend(), RenderBackend::Cpu);
        assert_eq!(browser.tab_count(), 0);
    }

    #[test]
    fn test_browser_tabs() {
        let mut browser = Browser::new(800, 600);
        let t1 = browser.open_tab("https://example.com".into()).unwrap();
        assert_eq!(browser.tab_count(), 1);
        browser.close_tab(t1).unwrap();
        assert_eq!(browser.tab_count(), 0);
    }

    #[test]
    fn test_browser_render_pipeline() {
        let mut browser = Browser::new(100, 100);
        browser.flush_render();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_browser_encrypt() {
        let mut browser = Browser::new(4, 4);
        browser.framebuffer_mut().clear([255, 0, 0, 255]);
        let key = [0xAB; 16];
        browser.encrypt_framebuffer(&key);
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }
}
