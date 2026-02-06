use alloc::vec::Vec;
use super::{HptpError, HptpResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpticalClockType {
    StrontiumLattice,
    YtterbiumLattice,
    AluminumIon,
    MercuryIon,
    GenericOptical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpticalClockStatus {
    Initializing,
    Locking,
    Locked,
    FreqRunning,
    Error,
    Offline,
}

#[derive(Debug, Clone)]
pub struct OpticalReference {
    pub clock_type: OpticalClockType,
    pub status: OpticalClockStatus,
    pub frequency_hz: f64,
    pub stability: f64,
    pub accuracy: f64,
    pub uptime_ns: u64,
    pub lock_count: u32,
    pub last_calibration_ns: u64,
}

#[derive(Debug, Clone)]
pub struct ClockComparison {
    pub clock_a: usize,
    pub clock_b: usize,
    pub offset_fs: i128,
    pub timestamp_ns: u64,
    pub agreement: bool,
}

pub struct OpticalClockManager {
    references: Vec<OpticalReference>,
    active_index: Option<usize>,
    comparison_history: Vec<ClockComparison>,
    max_history: usize,
}

impl OpticalClockManager {
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
            active_index: None,
            comparison_history: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn add_reference(&mut self, ref_clock: OpticalReference) -> usize {
        self.references.push(ref_clock);
        self.references.len() - 1
    }

    pub fn remove_reference(&mut self, index: usize) -> HptpResult<()> {
        if index >= self.references.len() {
            return Err(HptpError::ClockSourceUnavailable);
        }
        self.references.remove(index);
        match self.active_index {
            Some(active) if active == index => self.active_index = None,
            Some(active) if active > index => self.active_index = Some(active - 1),
            _ => {}
        }
        Ok(())
    }

    pub fn set_active(&mut self, index: usize) -> HptpResult<()> {
        if index >= self.references.len() {
            return Err(HptpError::ClockSourceUnavailable);
        }
        self.active_index = Some(index);
        Ok(())
    }

    pub fn active_reference(&self) -> Option<&OpticalReference> {
        self.active_index.and_then(|i| self.references.get(i))
    }

    pub fn compare_clocks(&mut self, idx_a: usize, idx_b: usize, offset_fs: i128) -> HptpResult<ClockComparison> {
        if idx_a >= self.references.len() || idx_b >= self.references.len() {
            return Err(HptpError::ClockSourceUnavailable);
        }
        let abs_offset = if offset_fs < 0 { -offset_fs } else { offset_fs };
        let comparison = ClockComparison {
            clock_a: idx_a,
            clock_b: idx_b,
            offset_fs,
            timestamp_ns: 0,
            agreement: abs_offset <= 100,
        };
        if self.comparison_history.len() >= self.max_history {
            self.comparison_history.remove(0);
        }
        self.comparison_history.push(comparison.clone());
        Ok(comparison)
    }

    pub fn best_reference(&self) -> Option<usize> {
        if self.references.is_empty() {
            return None;
        }
        let mut best_idx = 0;
        let mut best_stability = self.references[0].stability;
        for (i, r) in self.references.iter().enumerate().skip(1) {
            if r.stability < best_stability {
                best_stability = r.stability;
                best_idx = i;
            }
        }
        Some(best_idx)
    }

    pub fn reference_count(&self) -> usize {
        self.references.len()
    }

    pub fn locked_count(&self) -> usize {
        self.references.iter().filter(|r| r.status == OpticalClockStatus::Locked).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sr_reference() -> OpticalReference {
        OpticalReference {
            clock_type: OpticalClockType::StrontiumLattice,
            status: OpticalClockStatus::Locked,
            frequency_hz: 429_228_004_229_873.0,
            stability: 1e-18,
            accuracy: 2e-18,
            uptime_ns: 1_000_000_000,
            lock_count: 1,
            last_calibration_ns: 0,
        }
    }

    fn make_yb_reference() -> OpticalReference {
        OpticalReference {
            clock_type: OpticalClockType::YtterbiumLattice,
            status: OpticalClockStatus::Locked,
            frequency_hz: 518_295_836_590_863.6,
            stability: 1.5e-18,
            accuracy: 3e-18,
            uptime_ns: 500_000_000,
            lock_count: 2,
            last_calibration_ns: 100,
        }
    }

    #[test]
    fn test_optical_reference_creation() {
        let r = make_sr_reference();
        assert_eq!(r.clock_type, OpticalClockType::StrontiumLattice);
        assert_eq!(r.status, OpticalClockStatus::Locked);
        assert_eq!(r.frequency_hz, 429_228_004_229_873.0);
    }

    #[test]
    fn test_manager_new() {
        let mgr = OpticalClockManager::new();
        assert_eq!(mgr.reference_count(), 0);
        assert!(mgr.active_reference().is_none());
    }

    #[test]
    fn test_manager_add_reference() {
        let mut mgr = OpticalClockManager::new();
        let idx = mgr.add_reference(make_sr_reference());
        assert_eq!(idx, 0);
        assert_eq!(mgr.reference_count(), 1);
        let idx2 = mgr.add_reference(make_yb_reference());
        assert_eq!(idx2, 1);
        assert_eq!(mgr.reference_count(), 2);
    }

    #[test]
    fn test_manager_remove_reference() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_sr_reference());
        mgr.add_reference(make_yb_reference());
        assert!(mgr.remove_reference(0).is_ok());
        assert_eq!(mgr.reference_count(), 1);
    }

    #[test]
    fn test_manager_remove_out_of_bounds() {
        let mut mgr = OpticalClockManager::new();
        assert!(mgr.remove_reference(0).is_err());
    }

    #[test]
    fn test_manager_set_active() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_sr_reference());
        mgr.add_reference(make_yb_reference());
        assert!(mgr.set_active(1).is_ok());
        let active = mgr.active_reference().unwrap();
        assert_eq!(active.clock_type, OpticalClockType::YtterbiumLattice);
    }

    #[test]
    fn test_manager_set_active_out_of_bounds() {
        let mut mgr = OpticalClockManager::new();
        assert!(mgr.set_active(0).is_err());
    }

    #[test]
    fn test_manager_remove_active_clears() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_sr_reference());
        mgr.set_active(0).unwrap();
        assert!(mgr.active_reference().is_some());
        mgr.remove_reference(0).unwrap();
        assert!(mgr.active_reference().is_none());
    }

    #[test]
    fn test_compare_clocks_agreement() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_sr_reference());
        mgr.add_reference(make_yb_reference());
        let cmp = mgr.compare_clocks(0, 1, 50).unwrap();
        assert!(cmp.agreement);
        assert_eq!(cmp.offset_fs, 50);
    }

    #[test]
    fn test_compare_clocks_disagreement() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_sr_reference());
        mgr.add_reference(make_yb_reference());
        let cmp = mgr.compare_clocks(0, 1, 200).unwrap();
        assert!(!cmp.agreement);
    }

    #[test]
    fn test_compare_clocks_out_of_bounds() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_sr_reference());
        assert!(mgr.compare_clocks(0, 5, 0).is_err());
    }

    #[test]
    fn test_best_reference() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_yb_reference());
        mgr.add_reference(make_sr_reference());
        assert_eq!(mgr.best_reference(), Some(1));
    }

    #[test]
    fn test_best_reference_empty() {
        let mgr = OpticalClockManager::new();
        assert_eq!(mgr.best_reference(), None);
    }

    #[test]
    fn test_locked_count() {
        let mut mgr = OpticalClockManager::new();
        mgr.add_reference(make_sr_reference());
        let mut offline = make_yb_reference();
        offline.status = OpticalClockStatus::Offline;
        mgr.add_reference(offline);
        assert_eq!(mgr.locked_count(), 1);
    }

    #[test]
    fn test_clock_types() {
        assert_ne!(OpticalClockType::StrontiumLattice, OpticalClockType::YtterbiumLattice);
        assert_ne!(OpticalClockType::AluminumIon, OpticalClockType::MercuryIon);
        assert_ne!(OpticalClockType::GenericOptical, OpticalClockType::StrontiumLattice);
    }

    #[test]
    fn test_status_transitions() {
        let mut r = make_sr_reference();
        assert_eq!(r.status, OpticalClockStatus::Locked);
        r.status = OpticalClockStatus::FreqRunning;
        assert_eq!(r.status, OpticalClockStatus::FreqRunning);
        r.status = OpticalClockStatus::Error;
        assert_eq!(r.status, OpticalClockStatus::Error);
    }
}
