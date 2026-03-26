// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Tab isolation via kernel tasks.
// Each tab = separate kernel task with own stack/heap region.
// Panic in one tab doesn't affect others. catch_unwind within task.
// Double-panic or OOM: kernel reclaims task resources.

use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_TABS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    Loading,
    Active,
    Suspended,
    Crashed,
    Closed,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: u32,
    pub state: TabState,
    pub title: String,
    pub url: String,
    pub memory_bytes: usize,
    pub font_cache_bytes: usize,
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
        }
    }

    pub fn is_alive(&self) -> bool {
        matches!(self.state, TabState::Loading | TabState::Active | TabState::Suspended)
    }
}

pub struct TabManager {
    tabs: Vec<Tab>,
    next_id: u32,
    active_tab: Option<u32>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            next_id: 0,
            active_tab: None,
        }
    }

    pub fn open_tab(&mut self, url: String) -> Result<u32, TabError> {
        if self.tabs.len() >= MAX_TABS {
            return Err(TabError::MaxTabsReached);
        }
        let id = self.next_id;
        self.next_id += 1;
        let tab = Tab::new(id, url);
        self.tabs.push(tab);
        if self.active_tab.is_none() {
            self.active_tab = Some(id);
        }
        Ok(id)
    }

    pub fn close_tab(&mut self, id: u32) -> Result<(), TabError> {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.state = TabState::Closed;
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
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.state = TabState::Crashed;
            tab.memory_bytes = 0;
            tab.font_cache_bytes = 0;
        }
    }

    pub fn activate_tab(&mut self, id: u32) -> Result<(), TabError> {
        if let Some(tab) = self.tabs.iter().find(|t| t.id == id) {
            if tab.is_alive() {
                self.active_tab = Some(id);
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
}

#[derive(Debug, Clone)]
pub enum TabError {
    MaxTabsReached,
    TabNotFound(u32),
    TabNotAlive(u32),
}

impl core::fmt::Display for TabError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TabError::MaxTabsReached => write!(f, "Maximum tab count ({}) reached", MAX_TABS),
            TabError::TabNotFound(id) => write!(f, "Tab {} not found", id),
            TabError::TabNotAlive(id) => write!(f, "Tab {} is not alive", id),
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
}
