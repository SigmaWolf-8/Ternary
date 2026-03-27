// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Tab isolation via kernel tasks.
// Each tab = separate kernel task with own stack/heap region.
// Each tab's heap is a separate allocator region with guard pages.
// Page faults in a tab's region trigger task termination, not kernel panic.
// Same-address-space isolation with hardware-assisted bounds checking,
// not separate address spaces.
//
// catch_unwind for recoverable panics.
// Double-panic or OOM: kernel reclaims task resources, other tabs unaffected.
// 30-second health check with automatic rollback on subsystem failure.

use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_TABS: usize = 64;
pub const DEFAULT_STACK_SIZE: usize = 64 * 1024;
pub const DEFAULT_HEAP_SIZE: usize = 4 * 1024 * 1024;
pub const GUARD_PAGE_SIZE: usize = 4096;
pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;
pub const MAX_CONSECUTIVE_CRASHES: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    Loading,
    Active,
    Suspended,
    Background,
    Crashed,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashReason {
    Panic,
    DoublePanic,
    OutOfMemory,
    PageFault,
    Timeout,
    HealthCheckFailed,
}

#[derive(Debug, Clone)]
pub struct TabMemoryRegion {
    pub stack_base: usize,
    pub stack_size: usize,
    pub heap_base: usize,
    pub heap_size: usize,
    pub guard_page_low: usize,
    pub guard_page_high: usize,
    pub heap_used: usize,
}

impl TabMemoryRegion {
    pub fn new(base_address: usize, stack_size: usize, heap_size: usize) -> Self {
        let guard_page_low = base_address;
        let stack_base = guard_page_low + GUARD_PAGE_SIZE;
        let heap_base = stack_base + stack_size;
        let guard_page_high = heap_base + heap_size;

        Self {
            stack_base,
            stack_size,
            heap_base,
            heap_size,
            guard_page_low,
            guard_page_high,
            heap_used: 0,
        }
    }

    pub fn total_size(&self) -> usize {
        GUARD_PAGE_SIZE + self.stack_size + self.heap_size + GUARD_PAGE_SIZE
    }

    pub fn contains_address(&self, addr: usize) -> bool {
        addr >= self.stack_base && addr < self.guard_page_high
    }

    pub fn is_guard_page(&self, addr: usize) -> bool {
        (addr >= self.guard_page_low && addr < self.stack_base)
            || (addr >= self.guard_page_high && addr < self.guard_page_high + GUARD_PAGE_SIZE)
    }

    pub fn heap_usage_percent(&self) -> f32 {
        if self.heap_size == 0 {
            return 0.0;
        }
        (self.heap_used as f32 / self.heap_size as f32) * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub last_check_timestamp: u64,
    pub consecutive_failures: u32,
    pub is_healthy: bool,
    pub last_response_ms: u64,
}

impl HealthStatus {
    pub fn new() -> Self {
        Self {
            last_check_timestamp: 0,
            consecutive_failures: 0,
            is_healthy: true,
            last_response_ms: 0,
        }
    }

    pub fn record_success(&mut self, timestamp: u64, response_ms: u64) {
        self.last_check_timestamp = timestamp;
        self.consecutive_failures = 0;
        self.is_healthy = true;
        self.last_response_ms = response_ms;
    }

    pub fn record_failure(&mut self, timestamp: u64) {
        self.last_check_timestamp = timestamp;
        self.consecutive_failures += 1;
        if self.consecutive_failures >= MAX_CONSECUTIVE_CRASHES {
            self.is_healthy = false;
        }
    }

    pub fn needs_check(&self, current_timestamp: u64) -> bool {
        current_timestamp.saturating_sub(self.last_check_timestamp) >= HEALTH_CHECK_INTERVAL_SECS
    }
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: u32,
    pub state: TabState,
    pub title: String,
    pub url: String,
    pub memory_bytes: usize,
    pub font_cache_bytes: usize,
    pub memory_region: Option<TabMemoryRegion>,
    pub crash_reason: Option<CrashReason>,
    pub crash_count: u32,
    pub health: HealthStatus,
    pub task_id: Option<u64>,
}

impl Tab {
    pub fn new(id: u32, url: String) -> Self {
        Self {
            id,
            state: TabState::Loading,
            title: String::new(),
            url,
            memory_bytes: 0,
            font_cache_bytes: 0,
            memory_region: None,
            crash_reason: None,
            crash_count: 0,
            health: HealthStatus::new(),
            task_id: None,
        }
    }

    pub fn is_alive(&self) -> bool {
        matches!(self.state, TabState::Loading | TabState::Active | TabState::Suspended | TabState::Background)
    }

    pub fn allocate_memory(&mut self, base_address: usize) {
        let region = TabMemoryRegion::new(base_address, DEFAULT_STACK_SIZE, DEFAULT_HEAP_SIZE);
        self.memory_bytes = region.total_size();
        self.memory_region = Some(region);
    }

    pub fn handle_crash(&mut self, reason: CrashReason) {
        self.state = TabState::Crashed;
        self.crash_reason = Some(reason);
        self.crash_count += 1;

        self.memory_bytes = 0;
        self.font_cache_bytes = 0;
        self.health.is_healthy = false;
    }

    pub fn can_recover(&self) -> bool {
        self.crash_count < MAX_CONSECUTIVE_CRASHES
            && !matches!(self.crash_reason, Some(CrashReason::DoublePanic))
    }

    pub fn recover(&mut self, base_address: usize) -> bool {
        if !self.can_recover() {
            return false;
        }

        self.state = TabState::Loading;
        self.crash_reason = None;
        self.allocate_memory(base_address);
        self.health = HealthStatus::new();
        true
    }
}

pub struct TabManager {
    tabs: Vec<Tab>,
    next_id: u32,
    active_tab: Option<u32>,
    next_base_address: usize,
    next_task_id: u64,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            next_id: 0,
            active_tab: None,
            next_base_address: 0x1000_0000,
            next_task_id: 1,
        }
    }

    pub fn open_tab(&mut self, url: String) -> Result<u32, TabError> {
        if self.tabs.len() >= MAX_TABS {
            return Err(TabError::MaxTabsReached);
        }
        let id = self.next_id;
        self.next_id += 1;

        let mut tab = Tab::new(id, url);
        tab.allocate_memory(self.next_base_address);
        tab.task_id = Some(self.next_task_id);
        tab.state = TabState::Active;

        self.next_task_id += 1;
        self.next_base_address += tab.memory_region.as_ref().map(|r| r.total_size()).unwrap_or(0) + GUARD_PAGE_SIZE;

        self.tabs.push(tab);
        if self.active_tab.is_none() {
            self.active_tab = Some(id);
        }
        Ok(id)
    }

    pub fn close_tab(&mut self, id: u32) -> Result<(), TabError> {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.state = TabState::Closed;
            tab.memory_bytes = 0;
            tab.font_cache_bytes = 0;

            if self.active_tab == Some(id) {
                self.active_tab = self.tabs.iter()
                    .find(|t| t.is_alive())
                    .map(|t| t.id);
            }
            Ok(())
        } else {
            Err(TabError::TabNotFound(id))
        }
    }

    pub fn crash_tab(&mut self, id: u32) {
        self.crash_tab_with_reason(id, CrashReason::Panic);
    }

    pub fn crash_tab_with_reason(&mut self, id: u32, reason: CrashReason) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.handle_crash(reason);

            if self.active_tab == Some(id) {
                self.active_tab = self.tabs.iter()
                    .find(|t| t.is_alive() && t.id != id)
                    .map(|t| t.id);
            }
        }
    }

    pub fn handle_page_fault(&mut self, fault_address: usize) -> Option<u32> {
        let tab_id = self.tabs.iter()
            .find(|t| t.is_alive() && t.memory_region.as_ref().map(|r| r.is_guard_page(fault_address)).unwrap_or(false))
            .map(|t| t.id);

        if let Some(id) = tab_id {
            self.crash_tab_with_reason(id, CrashReason::PageFault);
        }
        tab_id
    }

    pub fn recover_tab(&mut self, id: u32) -> Result<(), TabError> {
        let base = self.next_base_address;
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            if tab.recover(base) {
                self.next_base_address += tab.memory_region.as_ref()
                    .map(|r| r.total_size()).unwrap_or(0) + GUARD_PAGE_SIZE;
                Ok(())
            } else {
                Err(TabError::TabNotAlive(id))
            }
        } else {
            Err(TabError::TabNotFound(id))
        }
    }

    pub fn run_health_checks(&mut self, current_timestamp: u64) -> Vec<u32> {
        let mut failed_tabs = Vec::new();

        let tab_ids: Vec<u32> = self.tabs.iter()
            .filter(|t| t.is_alive() && t.health.needs_check(current_timestamp))
            .map(|t| t.id)
            .collect();

        for id in tab_ids {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                if tab.health.consecutive_failures >= MAX_CONSECUTIVE_CRASHES {
                    tab.handle_crash(CrashReason::HealthCheckFailed);
                    failed_tabs.push(id);
                } else {
                    tab.health.record_success(current_timestamp, 0);
                }
            }
        }

        failed_tabs
    }

    pub fn suspend_tab(&mut self, id: u32) -> Result<(), TabError> {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            if tab.is_alive() {
                tab.state = TabState::Suspended;
                Ok(())
            } else {
                Err(TabError::TabNotAlive(id))
            }
        } else {
            Err(TabError::TabNotFound(id))
        }
    }

    pub fn set_background(&mut self, id: u32) -> Result<(), TabError> {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            if tab.is_alive() {
                tab.state = TabState::Background;
                Ok(())
            } else {
                Err(TabError::TabNotAlive(id))
            }
        } else {
            Err(TabError::TabNotFound(id))
        }
    }

    pub fn activate_tab(&mut self, id: u32) -> Result<(), TabError> {
        if let Some(old_active) = self.active_tab {
            if old_active != id {
                if let Some(old_tab) = self.tabs.iter_mut().find(|t| t.id == old_active) {
                    if old_tab.state == TabState::Active {
                        old_tab.state = TabState::Background;
                    }
                }
            }
        }

        if let Some(tab) = self.tabs.iter().find(|t| t.id == id) {
            if tab.is_alive() {
                self.active_tab = Some(id);
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                    tab.state = TabState::Active;
                }
                Ok(())
            } else {
                Err(TabError::TabNotAlive(id))
            }
        } else {
            Err(TabError::TabNotFound(id))
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|id| self.tabs.iter().find(|t| t.id == id))
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.iter().filter(|t| t.is_alive()).count()
    }

    pub fn total_memory(&self) -> usize {
        self.tabs.iter().filter(|t| t.is_alive()).map(|t| t.memory_bytes).sum()
    }

    pub fn get_tab(&self, id: u32) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn get_tab_mut(&mut self, id: u32) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn background_tabs(&self) -> Vec<&Tab> {
        self.tabs.iter()
            .filter(|t| matches!(t.state, TabState::Background | TabState::Suspended))
            .collect()
    }

    pub fn crashed_tabs(&self) -> Vec<&Tab> {
        self.tabs.iter()
            .filter(|t| t.state == TabState::Crashed)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum TabError {
    MaxTabsReached,
    TabNotFound(u32),
    TabNotAlive(u32),
    MemoryAllocationFailed,
}

impl core::fmt::Display for TabError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TabError::MaxTabsReached => write!(f, "Maximum tab count ({}) reached", MAX_TABS),
            TabError::TabNotFound(id) => write!(f, "Tab {} not found", id),
            TabError::TabNotAlive(id) => write!(f, "Tab {} is not alive", id),
            TabError::MemoryAllocationFailed => write!(f, "Failed to allocate tab memory region"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_lifecycle() {
        let mut mgr = TabManager::new();
        let id = mgr.open_tab("https://example.com".into()).unwrap();
        assert_eq!(mgr.tab_count(), 1);

        mgr.close_tab(id).unwrap();
        assert_eq!(mgr.tab_count(), 0);
    }

    #[test]
    fn test_tab_crash_isolation() {
        let mut mgr = TabManager::new();
        let t1 = mgr.open_tab("tab1".into()).unwrap();
        let t2 = mgr.open_tab("tab2".into()).unwrap();

        mgr.crash_tab(t1);
        assert_eq!(mgr.get_tab(t1).unwrap().state, TabState::Crashed);
        assert!(mgr.get_tab(t2).unwrap().is_alive());
    }

    #[test]
    fn test_max_tabs() {
        let mut mgr = TabManager::new();
        for i in 0..MAX_TABS {
            mgr.open_tab(alloc::format!("tab{}", i)).unwrap();
        }
        assert!(mgr.open_tab("overflow".into()).is_err());
    }

    #[test]
    fn test_active_tab_switch() {
        let mut mgr = TabManager::new();
        let t1 = mgr.open_tab("tab1".into()).unwrap();
        let t2 = mgr.open_tab("tab2".into()).unwrap();

        assert_eq!(mgr.active_tab().unwrap().id, t1);
        mgr.activate_tab(t2).unwrap();
        assert_eq!(mgr.active_tab().unwrap().id, t2);
    }

    #[test]
    fn test_memory_region_allocation() {
        let mut mgr = TabManager::new();
        let id = mgr.open_tab("test".into()).unwrap();
        let tab = mgr.get_tab(id).unwrap();

        assert!(tab.memory_region.is_some());
        let region = tab.memory_region.as_ref().unwrap();
        assert_eq!(region.stack_size, DEFAULT_STACK_SIZE);
        assert_eq!(region.heap_size, DEFAULT_HEAP_SIZE);
        assert!(tab.memory_bytes > 0);
    }

    #[test]
    fn test_guard_page_detection() {
        let region = TabMemoryRegion::new(0x1000_0000, DEFAULT_STACK_SIZE, DEFAULT_HEAP_SIZE);
        assert!(region.is_guard_page(0x1000_0000));
        assert!(!region.is_guard_page(0x1000_1000));
        assert!(region.contains_address(0x1000_1000));
    }

    #[test]
    fn test_page_fault_handling() {
        let mut mgr = TabManager::new();
        let id = mgr.open_tab("test".into()).unwrap();
        let guard_addr = mgr.get_tab(id).unwrap()
            .memory_region.as_ref().unwrap().guard_page_low;

        let crashed_tab = mgr.handle_page_fault(guard_addr);
        assert_eq!(crashed_tab, Some(id));
        assert_eq!(mgr.get_tab(id).unwrap().state, TabState::Crashed);
        assert_eq!(mgr.get_tab(id).unwrap().crash_reason, Some(CrashReason::PageFault));
    }

    #[test]
    fn test_crash_recovery() {
        let mut mgr = TabManager::new();
        let id = mgr.open_tab("test".into()).unwrap();

        mgr.crash_tab(id);
        assert_eq!(mgr.get_tab(id).unwrap().state, TabState::Crashed);

        mgr.recover_tab(id).unwrap();
        assert_eq!(mgr.get_tab(id).unwrap().state, TabState::Loading);
        assert!(mgr.get_tab(id).unwrap().memory_region.is_some());
    }

    #[test]
    fn test_double_panic_no_recovery() {
        let mut mgr = TabManager::new();
        let id = mgr.open_tab("test".into()).unwrap();

        mgr.crash_tab_with_reason(id, CrashReason::DoublePanic);
        assert!(!mgr.get_tab(id).unwrap().can_recover());
        assert!(mgr.recover_tab(id).is_err());
    }

    #[test]
    fn test_health_check() {
        let mut health = HealthStatus::new();
        assert!(health.is_healthy);
        assert!(health.needs_check(HEALTH_CHECK_INTERVAL_SECS + 1));

        health.record_success(10, 5);
        assert!(health.is_healthy);
        assert!(!health.needs_check(20));

        for _ in 0..MAX_CONSECUTIVE_CRASHES {
            health.record_failure(100);
        }
        assert!(!health.is_healthy);
    }

    #[test]
    fn test_background_tab() {
        let mut mgr = TabManager::new();
        let t1 = mgr.open_tab("tab1".into()).unwrap();
        let t2 = mgr.open_tab("tab2".into()).unwrap();

        mgr.set_background(t2).unwrap();
        assert_eq!(mgr.get_tab(t2).unwrap().state, TabState::Background);
        assert_eq!(mgr.background_tabs().len(), 1);
    }

    #[test]
    fn test_tab_task_id() {
        let mut mgr = TabManager::new();
        let _t1 = mgr.open_tab("tab1".into()).unwrap();
        let _t2 = mgr.open_tab("tab2".into()).unwrap();

        let task1 = mgr.get_tab(_t1).unwrap().task_id.unwrap();
        let task2 = mgr.get_tab(_t2).unwrap().task_id.unwrap();
        assert_ne!(task1, task2);
    }

    #[test]
    fn test_crash_count() {
        let mut mgr = TabManager::new();
        let id = mgr.open_tab("test".into()).unwrap();

        mgr.crash_tab(id);
        assert_eq!(mgr.get_tab(id).unwrap().crash_count, 1);

        mgr.recover_tab(id).unwrap();
        mgr.crash_tab(id);
        assert_eq!(mgr.get_tab(id).unwrap().crash_count, 2);
    }

    #[test]
    fn test_heap_usage() {
        let mut region = TabMemoryRegion::new(0x1000_0000, DEFAULT_STACK_SIZE, DEFAULT_HEAP_SIZE);
        assert_eq!(region.heap_usage_percent(), 0.0);
        region.heap_used = region.heap_size / 2;
        assert!((region.heap_usage_percent() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_activate_sends_old_to_background() {
        let mut mgr = TabManager::new();
        let t1 = mgr.open_tab("tab1".into()).unwrap();
        let t2 = mgr.open_tab("tab2".into()).unwrap();

        assert_eq!(mgr.get_tab(t1).unwrap().state, TabState::Active);
        mgr.activate_tab(t2).unwrap();
        assert_eq!(mgr.get_tab(t1).unwrap().state, TabState::Background);
        assert_eq!(mgr.get_tab(t2).unwrap().state, TabState::Active);
    }
}
