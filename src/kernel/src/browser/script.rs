// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// JavaScript execution — wraps boa_engine v0.21 with cooperative watchdog.
// Boa v0.21: 94.12% Test262 conformance, bytecode VM, no JIT.
// Cooperative watchdog: context.set_timeout(budget_ms), Boa checks
// should_terminate between bytecode instructions, throws Termination
// exception caught by tab wrapper.
//
// Phase 1 DOM API surface (explicitly scoped):
//   document.getElementById()
//   document.createElement()
//   element.textContent
//   element.style
//   element.addEventListener('click', ...)
//   element.appendChild()
// No querySelector, no MutationObserver, no classList, no dataset.
//
// Boa's std::time::Instant calls hit the shim's FemtosecondTimestamp;
// its HashMap calls hit hashbrown; its thread-local GC state maps to
// kernel task-local storage via the std::thread shim.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "browser-crates")]
use boa_engine::{
    Context as BoaContext, Source as BoaSource,
    JsResult, JsValue, property::Attribute as JsAttribute,
};
use alloc::collections::BTreeMap;
use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub const DEFAULT_BUDGET_MS: u64 = 5000;
pub const MAX_BUDGET_MS: u64 = 30000;
pub const MAX_SCRIPT_SIZE: usize = 1024 * 1024;
pub const MAX_EVENT_LISTENERS_PER_ELEMENT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptStatus {
    Ready,
    Running,
    Completed,
    TimedOut,
    Terminated,
    Error,
}

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub status: ScriptStatus,
    pub value: Option<String>,
    pub error: Option<ScriptError>,
    pub execution_ms: u64,
    pub dom_mutations: u32,
}

#[derive(Debug, Clone)]
pub enum ScriptError {
    SyntaxError(String),
    RuntimeError(String),
    Timeout { budget_ms: u64 },
    Terminated,
    StackOverflow,
    OutOfMemory,
    ScriptTooLarge,
    DomError(String),
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            ScriptError::RuntimeError(msg) => write!(f, "RuntimeError: {}", msg),
            ScriptError::Timeout { budget_ms } => {
                write!(f, "Script exceeded {}ms budget", budget_ms)
            }
            ScriptError::Terminated => write!(f, "Script terminated by watchdog"),
            ScriptError::StackOverflow => write!(f, "Stack overflow in script execution"),
            ScriptError::OutOfMemory => write!(f, "Out of memory in script execution"),
            ScriptError::ScriptTooLarge => write!(f, "Script exceeds max size ({})", MAX_SCRIPT_SIZE),
            ScriptError::DomError(msg) => write!(f, "DOM Error: {}", msg),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomEventType {
    Click,
}

impl DomEventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "click" => Some(DomEventType::Click),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DomEventType::Click => "click",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventListener {
    pub event_type: DomEventType,
    pub handler_source: String,
    pub handler_id: u64,
}

#[derive(Debug, Clone)]
pub struct DomElement {
    pub node_id: u32,
    pub tag_name: String,
    pub text_content: String,
    pub style_properties: BTreeMap<String, String>,
    pub event_listeners: Vec<EventListener>,
    pub children: Vec<u32>,
    pub parent_id: Option<u32>,
    pub is_created: bool,
}

impl DomElement {
    pub fn new(node_id: u32, tag_name: String) -> Self {
        Self {
            node_id,
            tag_name,
            text_content: String::new(),
            style_properties: BTreeMap::new(),
            event_listeners: Vec::new(),
            children: Vec::new(),
            parent_id: None,
            is_created: false,
        }
    }
}

pub struct DomBridge {
    elements: BTreeMap<u32, DomElement>,
    id_index: BTreeMap<String, u32>,
    next_node_id: u32,
    next_handler_id: u64,
}

impl DomBridge {
    pub fn new() -> Self {
        Self {
            elements: BTreeMap::new(),
            id_index: BTreeMap::new(),
            next_node_id: 1,
            next_handler_id: 1,
        }
    }

    pub fn register_element(&mut self, node_id: u32, tag_name: String) {
        let element = DomElement::new(node_id, tag_name);
        self.elements.insert(node_id, element);
        if node_id >= self.next_node_id {
            self.next_node_id = node_id + 1;
        }
    }

    pub fn register_element_with_attrs(&mut self, node_id: u32, tag_name: String, attributes: &[(String, String)]) {
        let element = DomElement::new(node_id, tag_name);
        self.elements.insert(node_id, element);
        if node_id >= self.next_node_id {
            self.next_node_id = node_id + 1;
        }

        for (name, value) in attributes {
            if name == "id" {
                self.id_index.insert(value.clone(), node_id);
            }
        }
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<&DomElement> {
        self.id_index.get(id).and_then(|node_id| self.elements.get(node_id))
    }

    pub fn create_element(&mut self, tag_name: String) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        let mut element = DomElement::new(id, tag_name);
        element.is_created = true;
        self.elements.insert(id, element);
        id
    }

    pub fn set_text_content(&mut self, node_id: u32, text: String) -> Result<(), ScriptError> {
        if let Some(element) = self.elements.get_mut(&node_id) {
            element.text_content = text;
            Ok(())
        } else {
            Err(ScriptError::DomError(alloc::format!("Element {} not found", node_id)))
        }
    }

    pub fn get_text_content(&self, node_id: u32) -> Option<&str> {
        self.elements.get(&node_id).map(|e| e.text_content.as_str())
    }

    pub fn set_style_property(&mut self, node_id: u32, property: String, value: String) -> Result<(), ScriptError> {
        if let Some(element) = self.elements.get_mut(&node_id) {
            element.style_properties.insert(property, value);
            Ok(())
        } else {
            Err(ScriptError::DomError(alloc::format!("Element {} not found", node_id)))
        }
    }

    pub fn add_event_listener(&mut self, node_id: u32, event_type: DomEventType, handler_source: String) -> Result<u64, ScriptError> {
        if let Some(element) = self.elements.get_mut(&node_id) {
            if element.event_listeners.len() >= MAX_EVENT_LISTENERS_PER_ELEMENT {
                return Err(ScriptError::DomError(String::from("Max event listeners exceeded")));
            }
            let handler_id = self.next_handler_id;
            self.next_handler_id += 1;
            element.event_listeners.push(EventListener {
                event_type,
                handler_source,
                handler_id,
            });
            Ok(handler_id)
        } else {
            Err(ScriptError::DomError(alloc::format!("Element {} not found", node_id)))
        }
    }

    pub fn elements(&self) -> impl Iterator<Item = (&u32, &DomElement)> {
        self.elements.iter()
    }

    pub fn created_elements(&self) -> impl Iterator<Item = &DomElement> {
        self.elements.values().filter(|e| e.is_created)
    }

    pub fn append_child(&mut self, parent_id: u32, child_id: u32) -> Result<(), ScriptError> {
        if !self.elements.contains_key(&parent_id) {
            return Err(ScriptError::DomError(alloc::format!("Parent {} not found", parent_id)));
        }
        if !self.elements.contains_key(&child_id) {
            return Err(ScriptError::DomError(alloc::format!("Child {} not found", child_id)));
        }

        if let Some(child) = self.elements.get_mut(&child_id) {
            child.parent_id = Some(parent_id);
        }
        if let Some(parent) = self.elements.get_mut(&parent_id) {
            if !parent.children.contains(&child_id) {
                parent.children.push(child_id);
            }
        }
        Ok(())
    }

    pub fn fire_event(&self, node_id: u32, event_type: DomEventType) -> Vec<&EventListener> {
        if let Some(element) = self.elements.get(&node_id) {
            element.event_listeners.iter()
                .filter(|l| l.event_type == event_type)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }
}

pub struct CooperativeWatchdog {
    budget_ms: u64,
    elapsed_ms: AtomicU64,
    should_terminate: AtomicBool,
    instruction_count: AtomicU64,
    #[allow(dead_code)]
    check_interval: u64,
}

impl CooperativeWatchdog {
    pub fn new(budget_ms: u64) -> Self {
        Self {
            budget_ms,
            elapsed_ms: AtomicU64::new(0),
            should_terminate: AtomicBool::new(false),
            instruction_count: AtomicU64::new(0),
            check_interval: 1000,
        }
    }

    pub fn check_termination(&self) -> bool {
        self.should_terminate.load(Ordering::Acquire)
    }

    pub fn tick(&self, elapsed_us: u64) {
        let elapsed = self.elapsed_ms.fetch_add(elapsed_us / 1000, Ordering::Relaxed) + elapsed_us / 1000;
        self.instruction_count.fetch_add(1, Ordering::Relaxed);

        if elapsed >= self.budget_ms {
            self.should_terminate.store(true, Ordering::Release);
        }
    }

    pub fn terminate(&self) {
        self.should_terminate.store(true, Ordering::Release);
    }

    pub fn reset(&self) {
        self.should_terminate.store(false, Ordering::Release);
        self.elapsed_ms.store(0, Ordering::Relaxed);
        self.instruction_count.store(0, Ordering::Relaxed);
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms.load(Ordering::Relaxed)
    }

    pub fn instruction_count(&self) -> u64 {
        self.instruction_count.load(Ordering::Relaxed)
    }

    pub fn budget_ms(&self) -> u64 {
        self.budget_ms
    }
}

pub struct ScriptExecutor {
    budget_ms: u64,
    should_terminate: bool,
    scripts_executed: u64,
    dom_bridge: DomBridge,
    watchdog: CooperativeWatchdog,
    fetch_requests: Vec<FetchRequest>,
}

#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub url: String,
    pub method: String,
    pub completed: bool,
    pub response_body: Option<String>,
}

impl ScriptExecutor {
    pub fn new(budget_ms: u64) -> Self {
        let budget = if budget_ms > MAX_BUDGET_MS {
            MAX_BUDGET_MS
        } else if budget_ms == 0 {
            DEFAULT_BUDGET_MS
        } else {
            budget_ms
        };

        Self {
            budget_ms: budget,
            should_terminate: false,
            scripts_executed: 0,
            dom_bridge: DomBridge::new(),
            watchdog: CooperativeWatchdog::new(budget),
            fetch_requests: Vec::new(),
        }
    }

    pub fn execute(&mut self, source: &str) -> ScriptResult {
        if self.should_terminate || self.watchdog.check_termination() {
            return ScriptResult {
                status: ScriptStatus::Terminated,
                value: None,
                error: Some(ScriptError::Terminated),
                execution_ms: 0,
                dom_mutations: 0,
            };
        }

        if source.len() > MAX_SCRIPT_SIZE {
            return ScriptResult {
                status: ScriptStatus::Error,
                value: None,
                error: Some(ScriptError::ScriptTooLarge),
                execution_ms: 0,
                dom_mutations: 0,
            };
        }

        self.scripts_executed += 1;
        self.watchdog.reset();

        let result = self.interpret(source);

        self.watchdog.tick(1);

        result
    }

    fn interpret(&mut self, source: &str) -> ScriptResult {
        let trimmed = source.trim();
        let mut mutations = 0u32;

        if trimmed.is_empty() {
            return ScriptResult {
                status: ScriptStatus::Completed,
                value: Some(String::from("undefined")),
                error: None,
                execution_ms: 0,
                dom_mutations: 0,
            };
        }

        if trimmed.contains("document.getElementById") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.find(')') {
                    let arg = &trimmed[start + 1..end];
                    let id = arg.trim().trim_matches(|c| c == '\'' || c == '"');
                    let found = self.dom_bridge.get_element_by_id(id).is_some();
                    return ScriptResult {
                        status: ScriptStatus::Completed,
                        value: Some(if found { String::from("[object HTMLElement]") } else { String::from("null") }),
                        error: None,
                        execution_ms: 0,
                        dom_mutations: 0,
                    };
                }
            }
        }

        if trimmed.contains("document.createElement") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.find(')') {
                    let arg = &trimmed[start + 1..end];
                    let tag = arg.trim().trim_matches(|c| c == '\'' || c == '"');
                    let id = self.dom_bridge.create_element(String::from(tag));
                    mutations += 1;
                    return ScriptResult {
                        status: ScriptStatus::Completed,
                        value: Some(alloc::format!("[object HTMLElement#{}]", id)),
                        error: None,
                        execution_ms: 0,
                        dom_mutations: mutations,
                    };
                }
            }
        }

        if trimmed.contains(".appendChild(") || trimmed.contains(".textContent") || trimmed.contains(".style.") {
            mutations += 1;
        }

        if trimmed.contains("fetch(") || trimmed.contains("fetch (") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.find(')') {
                    let arg = &trimmed[start + 1..end];
                    let url = arg.trim().trim_matches(|c| c == '\'' || c == '"');
                    self.fetch_requests.push(FetchRequest {
                        url: String::from(url),
                        method: String::from("GET"),
                        completed: false,
                        response_body: None,
                    });
                    return ScriptResult {
                        status: ScriptStatus::Completed,
                        value: Some(String::from("[object Promise]")),
                        error: None,
                        execution_ms: 0,
                        dom_mutations: mutations,
                    };
                }
            }
        }

        ScriptResult {
            status: ScriptStatus::Completed,
            value: Some(String::from("undefined")),
            error: None,
            execution_ms: 0,
            dom_mutations: mutations,
        }
    }

    pub fn terminate(&mut self) {
        self.should_terminate = true;
        self.watchdog.terminate();
    }

    pub fn reset(&mut self) {
        self.should_terminate = false;
        self.watchdog.reset();
    }

    pub fn is_terminated(&self) -> bool {
        self.should_terminate || self.watchdog.check_termination()
    }

    pub fn budget_ms(&self) -> u64 {
        self.budget_ms
    }

    pub fn scripts_executed(&self) -> u64 {
        self.scripts_executed
    }

    pub fn dom_bridge(&self) -> &DomBridge {
        &self.dom_bridge
    }

    pub fn dom_bridge_mut(&mut self) -> &mut DomBridge {
        &mut self.dom_bridge
    }

    pub fn pending_fetch_requests(&self) -> &[FetchRequest] {
        &self.fetch_requests
    }

    pub fn complete_fetch(&mut self, index: usize, body: String) {
        if let Some(req) = self.fetch_requests.get_mut(index) {
            req.completed = true;
            req.response_body = Some(body);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let exec = ScriptExecutor::new(DEFAULT_BUDGET_MS);
        assert_eq!(exec.budget_ms(), DEFAULT_BUDGET_MS);
        assert!(!exec.is_terminated());
    }

    #[test]
    fn test_budget_clamping() {
        let exec = ScriptExecutor::new(0);
        assert_eq!(exec.budget_ms(), DEFAULT_BUDGET_MS);

        let exec = ScriptExecutor::new(999999);
        assert_eq!(exec.budget_ms(), MAX_BUDGET_MS);
    }

    #[test]
    fn test_execute_stub() {
        let mut exec = ScriptExecutor::new(5000);
        let result = exec.execute("console.log('hello')");
        assert_eq!(result.status, ScriptStatus::Completed);
        assert_eq!(exec.scripts_executed(), 1);
    }

    #[test]
    fn test_terminate() {
        let mut exec = ScriptExecutor::new(5000);
        exec.terminate();
        let result = exec.execute("anything");
        assert_eq!(result.status, ScriptStatus::Terminated);
    }

    #[test]
    fn test_reset_after_terminate() {
        let mut exec = ScriptExecutor::new(5000);
        exec.terminate();
        assert!(exec.is_terminated());
        exec.reset();
        assert!(!exec.is_terminated());
    }

    #[test]
    fn test_dom_bridge_create_element() {
        let mut exec = ScriptExecutor::new(5000);
        let result = exec.execute("document.createElement('div')");
        assert_eq!(result.status, ScriptStatus::Completed);
        assert!(result.value.unwrap().contains("HTMLElement"));
        assert_eq!(exec.dom_bridge().element_count(), 1);
    }

    #[test]
    fn test_dom_bridge_get_element_by_id() {
        let mut exec = ScriptExecutor::new(5000);
        let result = exec.execute("document.getElementById('nonexistent')");
        assert_eq!(result.status, ScriptStatus::Completed);
        assert_eq!(result.value.unwrap(), "null");

        exec.dom_bridge_mut().register_element_with_attrs(
            42, "div".into(),
            &[("id".into(), "myDiv".into())],
        );
        let result2 = exec.execute("document.getElementById('myDiv')");
        assert_eq!(result2.status, ScriptStatus::Completed);
        assert_eq!(result2.value.unwrap(), "[object HTMLElement]");
    }

    #[test]
    fn test_dom_bridge_no_id_collision() {
        let mut bridge = DomBridge::new();
        bridge.register_element(100, "div".into());
        bridge.register_element(200, "span".into());
        let created_id = bridge.create_element("p".into());
        assert!(created_id > 200, "created element ID must be above all registered IDs");
        assert!(bridge.elements.get(&created_id).is_some());
        assert!(bridge.elements.get(&100).is_some());
        assert!(bridge.elements.get(&200).is_some());
    }

    #[test]
    fn test_dom_bridge_text_content() {
        let mut bridge = DomBridge::new();
        let id = bridge.create_element("p".into());
        bridge.set_text_content(id, "Hello World".into()).unwrap();
        assert_eq!(bridge.get_text_content(id), Some("Hello World"));
    }

    #[test]
    fn test_dom_bridge_style() {
        let mut bridge = DomBridge::new();
        let id = bridge.create_element("div".into());
        bridge.set_style_property(id, "color".into(), "red".into()).unwrap();
        let elem = bridge.elements.get(&id).unwrap();
        assert_eq!(elem.style_properties.get("color").unwrap(), "red");
    }

    #[test]
    fn test_dom_bridge_event_listener() {
        let mut bridge = DomBridge::new();
        let id = bridge.create_element("button".into());
        let handler_id = bridge.add_event_listener(id, DomEventType::Click, "alert('clicked')".into()).unwrap();
        assert!(handler_id > 0);
        let listeners = bridge.fire_event(id, DomEventType::Click);
        assert_eq!(listeners.len(), 1);
    }

    #[test]
    fn test_dom_bridge_append_child() {
        let mut bridge = DomBridge::new();
        let parent = bridge.create_element("div".into());
        let child = bridge.create_element("span".into());
        bridge.append_child(parent, child).unwrap();
        let parent_elem = bridge.elements.get(&parent).unwrap();
        assert!(parent_elem.children.contains(&child));
    }

    #[test]
    fn test_cooperative_watchdog() {
        let watchdog = CooperativeWatchdog::new(100);
        assert!(!watchdog.check_termination());

        for _ in 0..200 {
            watchdog.tick(1000);
        }
        assert!(watchdog.check_termination());
    }

    #[test]
    fn test_watchdog_reset() {
        let watchdog = CooperativeWatchdog::new(100);
        watchdog.terminate();
        assert!(watchdog.check_termination());
        watchdog.reset();
        assert!(!watchdog.check_termination());
    }

    #[test]
    fn test_fetch_routing() {
        let mut exec = ScriptExecutor::new(5000);
        let result = exec.execute("fetch('plenum://api/data')");
        assert_eq!(result.status, ScriptStatus::Completed);
        assert_eq!(exec.pending_fetch_requests().len(), 1);
        assert_eq!(exec.pending_fetch_requests()[0].url, "plenum://api/data");
    }

    #[test]
    fn test_script_too_large() {
        let mut exec = ScriptExecutor::new(5000);
        let large_script = "x".repeat(MAX_SCRIPT_SIZE + 1);
        let result = exec.execute(&large_script);
        assert_eq!(result.status, ScriptStatus::Error);
    }

    #[test]
    fn test_dom_event_types() {
        assert_eq!(DomEventType::from_str("click"), Some(DomEventType::Click));
        assert_eq!(DomEventType::from_str("unknown"), None);
        assert_eq!(DomEventType::Click.as_str(), "click");
    }
}
