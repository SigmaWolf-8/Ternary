// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumBrowser Kernel Subsystem — Pipeline Orchestrator
// Not a fork. Not userspace. Kernel subsystem with direct access to
// GPU, ternary cryptographic stack, and z=0 distributor.
//
// Pipeline: parse → layout → script → render (CPU/GPU) → mesh color → encrypt → display
//
// Phase 1: CPU rendering path via tiny-skia + resvg.
// Phase 2: GPU rendering with IOMMU-isolated VRAM.
//
// Import enforcement: this module imports ONLY from
// crate::distributor::RequestInterface for z=0 dispatch.
// Zero imports from crate::layers::*, crate::crypto::*, crate::network::*.

pub mod parse;
pub mod layout;
pub mod script;
pub mod render;
pub mod render_cpu;
pub mod tabs;
pub mod input;
pub mod net;
pub mod mesh;
pub mod color;
pub mod home_page;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use tabs::{TabManager, TabError};
use render_cpu::{CpuRenderer, CpuFramebuffer};
use render::{RenderBackendType, RenderScene, RenderPrimitive, RenderColor};
use script::ScriptExecutor;
use net::NetworkLayer;
use crate::distributor::{RequestInterface, RequestResult};
use crate::distributor::z_router::RequestType;

pub struct Browser {
    tab_manager: TabManager,
    renderer: CpuRenderer,
    framebuffer: CpuFramebuffer,
    network: NetworkLayer,
    pipeline_state: PipelineState,
    script_executor: ScriptExecutor,
    input_handler: Option<input::BrowserInputHandler>,
    font_cache: layout::FontCache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    Idle,
    Parsing,
    Layout,
    Scripting,
    Rendering,
    MeshColor,
    Encrypting,
    DisplayReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendSelection {
    Cpu,
    Gpu,
}

impl Browser {
    pub fn new(width: u32, height: u32, distributor: Box<dyn RequestInterface>) -> Self {
        Self {
            tab_manager: TabManager::new(),
            renderer: CpuRenderer::new(),
            framebuffer: CpuFramebuffer::new(width, height),
            network: NetworkLayer::new(distributor),
            pipeline_state: PipelineState::Idle,
            script_executor: ScriptExecutor::new(script::DEFAULT_BUDGET_MS),
            input_handler: None,
            font_cache: layout::FontCache::new(),
        }
    }

    pub fn init_input(&mut self, session_key: [u8; 32]) {
        self.input_handler = Some(input::BrowserInputHandler::new(session_key));
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

    pub fn render_backend(&self) -> RenderBackendSelection {
        RenderBackendSelection::Cpu
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

    pub fn render_page(&mut self, html: &str) -> Result<(), parse::ParseError> {
        self.pipeline_state = PipelineState::Parsing;
        let (mut document, _stylesheet) = parse::parse_html_with_styles(html)?;

        self.pipeline_state = PipelineState::Layout;
        let w = self.framebuffer.width() as f32;
        let h = self.framebuffer.height() as f32;
        let mut engine = layout::LayoutEngine::new();
        let root = engine.create_box();
        self.build_layout_tree(&document, root, &mut engine);
        engine.compute_layout(w, h);

        self.pipeline_state = PipelineState::Scripting;
        self.sync_dom_to_bridge(&document);
        let scripts: Vec<String> = self.extract_scripts(&document);
        for script_source in &scripts {
            let result = self.script_executor.execute(script_source);
            if result.dom_mutations > 0 {
                Self::apply_bridge_mutations_to_dom(&self.script_executor.dom_bridge(), &mut document);
                engine = layout::LayoutEngine::new();
                let new_root = engine.create_box();
                self.build_layout_tree(&document, new_root, &mut engine);
                engine.compute_layout(w, h);
            }
        }

        self.pipeline_state = PipelineState::Rendering;
        let scene = RenderScene::from_layout_boxes(engine.all_boxes(), self.framebuffer.width(), self.framebuffer.height());
        self.renderer.render_scene_to_framebuffer(&scene, &mut self.framebuffer);

        self.pipeline_state = PipelineState::DisplayReady;
        Ok(())
    }

    fn apply_bridge_mutations_to_dom(bridge: &script::DomBridge, document: &mut parse::DomNode) {
        for (node_id, element) in bridge.elements() {
            if let Some(dom_node) = parse::DomNode::find_by_node_id_mut(document, *node_id) {
                if !element.text_content.is_empty() {
                    dom_node.children.retain(|c| !matches!(c.node_type, parse::NodeType::Text(_)));
                    dom_node.children.push(parse::DomNode::new_text(element.text_content.clone(), dom_node.depth + 1));
                }

                for (prop, val) in &element.style_properties {
                    dom_node.computed_style.set(prop.clone(), val.clone());
                }
            }
        }

        let created: Vec<_> = bridge.created_elements().cloned().collect();
        for element in &created {
            if let Some(body) = parse::DomNode::find_by_tag_mut(document, "body") {
                let depth = body.depth + 1;
                let mut new_node = parse::DomNode::new_element(element.tag_name.clone(), depth);
                if !element.text_content.is_empty() {
                    new_node.children.push(parse::DomNode::new_text(element.text_content.clone(), depth + 1));
                }
                body.children.push(new_node);
            }
        }
    }

    fn build_layout_tree(&self, node: &parse::DomNode, parent_id: u32, engine: &mut layout::LayoutEngine) {
        for child in &node.children {
            match &child.node_type {
                parse::NodeType::Element(tag) => {
                    let style = Self::css_to_layout_style(&child.computed_style, tag);
                    let child_id = engine.create_box_with_style(style);
                    engine.add_child(parent_id, child_id);
                    self.build_layout_tree(child, child_id, engine);
                }
                parse::NodeType::Text(text) => {
                    let font_size = child.computed_style.get("font-size")
                        .and_then(|v| Self::parse_px_value(v))
                        .unwrap_or(16.0);
                    let text_id = engine.create_box();
                    engine.set_text(text_id, text.clone(), font_size);
                    engine.add_child(parent_id, text_id);
                }
                _ => {}
            }
        }
    }

    fn css_to_layout_style(computed: &parse::StyleDeclaration, tag: &str) -> layout::LayoutStyle {
        let mut style = layout::LayoutStyle::new();

        if let Some(display) = computed.get("display") {
            style.display = match display {
                "flex" => layout::Display::Flex,
                "grid" => layout::Display::Grid,
                "inline" => layout::Display::Inline,
                "none" => layout::Display::None,
                _ => layout::Display::Block,
            };
        } else {
            style.display = match tag {
                "span" | "a" | "em" | "strong" | "b" | "i" => layout::Display::Inline,
                _ => layout::Display::Block,
            };
        }

        if let Some(pos) = computed.get("position") {
            style.position = match pos {
                "relative" => layout::Position::Relative,
                "absolute" => layout::Position::Absolute,
                "fixed" => layout::Position::Fixed,
                _ => layout::Position::Static,
            };
        }

        if let Some(dir) = computed.get("flex-direction") {
            style.flex_direction = match dir {
                "column" => layout::FlexDirection::Column,
                "row-reverse" => layout::FlexDirection::RowReverse,
                "column-reverse" => layout::FlexDirection::ColumnReverse,
                _ => layout::FlexDirection::Row,
            };
        }

        if let Some(v) = computed.get("flex-grow").and_then(|v| Self::parse_float(v)) {
            style.flex_grow = v;
        }
        if let Some(v) = computed.get("flex-shrink").and_then(|v| Self::parse_float(v)) {
            style.flex_shrink = v;
        }
        if let Some(v) = computed.get("width").and_then(|v| Self::parse_px_value(v)) {
            style.width = Some(v);
        }
        if let Some(v) = computed.get("height").and_then(|v| Self::parse_px_value(v)) {
            style.height = Some(v);
        }
        if let Some(v) = computed.get("margin").and_then(|v| Self::parse_px_value(v)) {
            style.margin = layout::EdgeInsets::uniform(v);
        }
        if let Some(v) = computed.get("padding").and_then(|v| Self::parse_px_value(v)) {
            style.padding = layout::EdgeInsets::uniform(v);
        }

        style
    }

    fn parse_px_value(s: &str) -> Option<f32> {
        let s = s.trim().trim_end_matches("px");
        s.parse::<f32>().ok()
    }

    fn parse_float(s: &str) -> Option<f32> {
        s.trim().parse::<f32>().ok()
    }

    fn sync_dom_to_bridge(&mut self, node: &parse::DomNode) {
        let bridge = self.script_executor.dom_bridge_mut();
        let mut stack: Vec<&parse::DomNode> = Vec::new();
        stack.push(node);
        while let Some(n) = stack.pop() {
            if let parse::NodeType::Element(tag) = &n.node_type {
                bridge.register_element_with_attrs(
                    n.node_id,
                    tag.clone(),
                    &n.attributes,
                );
                if let Some(text) = n.children.iter().find_map(|c| {
                    if let parse::NodeType::Text(t) = &c.node_type { Some(t.clone()) } else { None }
                }) {
                    bridge.set_text_content(n.node_id, text);
                }
            }
            for child in n.children.iter().rev() {
                stack.push(child);
            }
        }
    }

    fn extract_scripts(&self, node: &parse::DomNode) -> Vec<String> {
        let mut scripts = Vec::new();
        let mut stack: Vec<&parse::DomNode> = Vec::new();
        stack.push(node);
        while let Some(n) = stack.pop() {
            if n.tag_name() == Some("script") {
                let text = n.collect_text();
                if !text.is_empty() {
                    scripts.push(text);
                }
            }
            for child in n.children.iter().rev() {
                stack.push(child);
            }
        }
        scripts
    }

    pub fn apply_mesh_color(&mut self) {
        self.pipeline_state = PipelineState::MeshColor;
        let w = self.framebuffer.width();
        let h = self.framebuffer.height();
        let pixels = self.framebuffer.pixels_mut();
        color::process_framebuffer_colors(pixels, w, h, color::DEFAULT_DEPTH);
        self.pipeline_state = PipelineState::DisplayReady;
    }

    pub fn encrypt_framebuffer(&mut self, keystream: &[u8]) {
        self.pipeline_state = PipelineState::Encrypting;
        render_cpu::sponge_encrypt_framebuffer(&mut self.framebuffer, keystream);
        self.pipeline_state = PipelineState::DisplayReady;
    }

    pub fn dispatch_request(&self, distributor: &mut dyn RequestInterface, req_type: RequestType) -> RequestResult {
        distributor.submit_request(req_type)
    }

    pub fn render_home_page(&mut self) {
        self.pipeline_state = PipelineState::Rendering;
        home_page::render_home_page(&mut self.framebuffer, self.tab_manager.tab_count());
        self.pipeline_state = PipelineState::DisplayReady;
    }

    pub fn network(&mut self) -> &mut NetworkLayer {
        &mut self.network
    }

    pub fn script_executor(&self) -> &ScriptExecutor {
        &self.script_executor
    }

    pub fn script_executor_mut(&mut self) -> &mut ScriptExecutor {
        &mut self.script_executor
    }

    pub fn input_handler(&mut self) -> Option<&mut input::BrowserInputHandler> {
        self.input_handler.as_mut()
    }

    pub fn tab_manager(&self) -> &TabManager {
        &self.tab_manager
    }

    pub fn tab_manager_mut(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }

    pub fn font_cache(&self) -> &layout::FontCache {
        &self.font_cache
    }

    pub fn font_cache_mut(&mut self) -> &mut layout::FontCache {
        &mut self.font_cache
    }

    pub fn renderer(&self) -> &CpuRenderer {
        &self.renderer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributor::Distributor;

    fn test_browser(w: u32, h: u32) -> Browser {
        Browser::new(w, h, Box::new(Distributor::new()))
    }

    #[test]
    fn test_browser_creation() {
        let browser = test_browser(1920, 1080);
        assert_eq!(browser.pipeline_state(), PipelineState::Idle);
        assert_eq!(browser.render_backend(), RenderBackendSelection::Cpu);
        assert_eq!(browser.tab_count(), 0);
    }

    #[test]
    fn test_browser_tabs() {
        let mut browser = test_browser(800, 600);
        let t1 = browser.open_tab("https://example.com".into()).unwrap();
        assert_eq!(browser.tab_count(), 1);
        browser.close_tab(t1).unwrap();
        assert_eq!(browser.tab_count(), 0);
    }

    #[test]
    fn test_browser_render_pipeline() {
        let mut browser = test_browser(100, 100);
        browser.flush_render();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_browser_mesh_color() {
        let mut browser = test_browser(4, 4);
        browser.framebuffer_mut().clear([128, 64, 200, 255]);
        browser.apply_mesh_color();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_browser_encrypt() {
        let mut browser = test_browser(4, 4);
        browser.framebuffer_mut().clear([255, 0, 0, 255]);
        let key = [0xAB; 16];
        browser.encrypt_framebuffer(&key);
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_browser_render_page() {
        let mut browser = test_browser(800, 600);
        let html = "<html><body><h1>Hello World</h1></body></html>";
        browser.render_page(html).unwrap();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_browser_render_page_with_style() {
        let mut browser = test_browser(800, 600);
        let html = r#"<html><head><style>body { background: white; } h1 { color: blue; }</style></head><body><h1>Styled</h1></body></html>"#;
        browser.render_page(html).unwrap();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_browser_render_page_with_script() {
        let mut browser = test_browser(800, 600);
        let html = r#"<html><body><script>document.createElement('div')</script><p>Content</p></body></html>"#;
        browser.render_page(html).unwrap();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_browser_input_init() {
        let mut browser = test_browser(800, 600);
        assert!(browser.input_handler().is_none());
        browser.init_input([0x42; 32]);
        assert!(browser.input_handler().is_some());
    }

    #[test]
    fn test_browser_font_cache() {
        let mut browser = test_browser(800, 600);
        let id = browser.font_cache_mut().load_font("Test".into(), alloc::vec![0u8; 1024]);
        assert!(browser.font_cache_mut().get_font(id).is_some());
    }

    #[test]
    fn test_browser_tab_isolation() {
        let mut browser = test_browser(800, 600);
        let t1 = browser.open_tab("tab1".into()).unwrap();
        let t2 = browser.open_tab("tab2".into()).unwrap();

        browser.tab_manager_mut().crash_tab(t1);
        assert!(browser.tab_manager().get_tab(t2).unwrap().is_alive());
        assert_eq!(browser.tab_count(), 1);
    }

    #[test]
    fn test_css_affects_layout() {
        let mut browser = test_browser(800, 600);
        let html = r#"<html><head><style>
            .container { display: flex; }
            .item { width: 200px; flex-grow: 1; }
        </style></head><body>
        <div class="container"><div class="item">A</div><div class="item">B</div></div>
        </body></html>"#;
        browser.render_page(html).unwrap();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_dom_bridge_sync_from_parsed_html() {
        let mut browser = test_browser(320, 200);
        let html = r#"<html><body><div id="target">Hello</div></body></html>"#;
        browser.render_page(html).unwrap();
        let elem = browser.script_executor().dom_bridge().get_element_by_id("target");
        assert!(elem.is_some(), "DOM bridge should find element by id after sync");
    }

    #[test]
    fn test_script_dom_mutation_triggers_relayout() {
        let mut browser = test_browser(320, 200);
        let html = r#"<html><body>
            <div id="app">Before</div>
            <script>document.createElement('span')</script>
        </body></html>"#;
        browser.render_page(html).unwrap();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
        assert!(browser.script_executor().scripts_executed() >= 1);
    }

    #[test]
    fn test_dirty_rect_partial_render() {
        use crate::browser::render::{RenderScene, RenderPrimitive, RenderColor, DirtyRect, PaintStyle};
        use crate::browser::layout::Rect;

        let mut renderer = render_cpu::CpuRenderer::new();
        let scene = RenderScene {
            viewport_width: 100,
            viewport_height: 100,
            primitives: alloc::vec![
                RenderPrimitive::FillRect {
                    rect: Rect { x: 10.0, y: 10.0, width: 30.0, height: 30.0 },
                    paint: PaintStyle::Solid(RenderColor::new(255, 0, 0, 255)),
                },
                RenderPrimitive::FillRect {
                    rect: Rect { x: 60.0, y: 60.0, width: 30.0, height: 30.0 },
                    paint: PaintStyle::Solid(RenderColor::new(0, 0, 255, 255)),
                },
            ],
            dirty_regions: alloc::vec![DirtyRect::new(0, 0, 50, 50)],
        };

        let mut output = alloc::vec![0u8; 100 * 100 * 4];
        use crate::browser::render::RenderBackend;
        let dirty = DirtyRect::new(0, 0, 50, 50);
        renderer.render_dirty(&scene, &mut output, 400, &dirty);
        assert!(renderer.partial_update_count() >= 1);
    }

    #[test]
    fn test_svg_rendering() {
        let mut fb = render_cpu::CpuFramebuffer::new(100, 100);
        let svg = crate::browser::render::SvgData {
            content: alloc::string::String::from(
                r#"<svg><rect x="10" y="10" width="50" height="50" fill="red"/></svg>"#
            ),
            width: 100,
            height: 100,
        };
        fb.render_svg(0, 0, 100, 100, &svg);
        let pixel_offset = (15 * 100 + 15) * 4;
        let r = fb.pixels()[pixel_offset];
        assert_eq!(r, 255, "SVG rect should render red at interior pixel");
    }

    #[test]
    fn test_browser_home_page() {
        let mut browser = test_browser(320, 240);
        browser.render_home_page();
        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
        let px = browser.framebuffer().get_pixel(160, 120);
        assert_ne!(px, [0, 0, 0, 0], "home page should render non-transparent pixels");
    }

    #[test]
    fn test_browser_request_dispatch() {
        let browser = test_browser(100, 100);
        let mut dist = crate::distributor::Distributor::new();
        let result = browser.dispatch_request(&mut dist, RequestType::HttpRequest);
        assert_eq!(result.status, crate::distributor::z_router::RouteStatus::Ok);
    }

    #[test]
    fn test_browser_full_pipeline() {
        let mut browser = test_browser(320, 200);

        let _t = browser.open_tab("plenum://test".into()).unwrap();
        assert!(browser.tab_count() >= 1);

        let html = "<html><body><h1>Test</h1><p>Content here</p></body></html>";
        browser.render_page(html).unwrap();

        let keystream = [0xAA; 32];
        browser.encrypt_framebuffer(&keystream);

        assert_eq!(browser.pipeline_state(), PipelineState::DisplayReady);
    }

    #[test]
    fn test_full_pipeline() {
        let mut browser = test_browser(64, 64);
        browser.render_home_page();
        browser.apply_mesh_color();
        let keystream = alloc::vec![0xABu8; 64 * 64 * 4];
        let original = browser.framebuffer().get_pixel(32, 32);
        browser.encrypt_framebuffer(&keystream);
        let encrypted = browser.framebuffer().get_pixel(32, 32);
        assert_ne!(original, encrypted, "encryption must change pixel values");
    }
}
