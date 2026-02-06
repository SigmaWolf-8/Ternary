use super::{DriverError, DriverResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSource {
    InternalCrystal,
    GpsdoRubidium,
    GpsdoCesium,
    OpticalLattice,
    ChipScale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockState {
    Uninitialized,
    Warming,
    Locking,
    Locked,
    Holdover,
    Freerun,
    Error,
}

#[derive(Debug, Clone)]
pub struct ClockCapabilities {
    pub source: ClockSource,
    pub resolution_femtoseconds: u64,
    pub accuracy_ppb: u64,
    pub max_drift_fs_per_second: u64,
    pub supports_pps: bool,
    pub supports_holdover: bool,
    pub holdover_stability_hours: u32,
}

#[derive(Debug, Clone)]
pub struct ClockReading {
    pub timestamp_fs: u128,
    pub source: ClockSource,
    pub state: ClockState,
    pub uncertainty_fs: u64,
    pub sequence_number: u64,
}

#[derive(Debug, Clone)]
pub struct CalibrationData {
    pub offset_fs: i64,
    pub drift_rate_ppb: i64,
    pub last_calibration_tick: u64,
    pub calibration_count: u32,
    pub temperature_mK: i32,
}

impl ClockCapabilities {
    fn for_source(source: ClockSource) -> Self {
        match source {
            ClockSource::InternalCrystal => ClockCapabilities {
                source,
                resolution_femtoseconds: 1_000_000,
                accuracy_ppb: 1_000,
                max_drift_fs_per_second: 1_000_000_000,
                supports_pps: false,
                supports_holdover: false,
                holdover_stability_hours: 0,
            },
            ClockSource::GpsdoRubidium => ClockCapabilities {
                source,
                resolution_femtoseconds: 1_000,
                accuracy_ppb: 10,
                max_drift_fs_per_second: 10_000,
                supports_pps: true,
                supports_holdover: true,
                holdover_stability_hours: 24,
            },
            ClockSource::GpsdoCesium => ClockCapabilities {
                source,
                resolution_femtoseconds: 100,
                accuracy_ppb: 1,
                max_drift_fs_per_second: 1_000,
                supports_pps: true,
                supports_holdover: true,
                holdover_stability_hours: 72,
            },
            ClockSource::OpticalLattice => ClockCapabilities {
                source,
                resolution_femtoseconds: 1,
                accuracy_ppb: 0,
                max_drift_fs_per_second: 1,
                supports_pps: true,
                supports_holdover: false,
                holdover_stability_hours: 0,
            },
            ClockSource::ChipScale => ClockCapabilities {
                source,
                resolution_femtoseconds: 10_000,
                accuracy_ppb: 100,
                max_drift_fs_per_second: 100_000,
                supports_pps: true,
                supports_holdover: true,
                holdover_stability_hours: 4,
            },
        }
    }
}

pub struct FemtosecondClockDriver {
    state: ClockState,
    source: ClockSource,
    capabilities: ClockCapabilities,
    current_time_fs: u128,
    calibration: CalibrationData,
    readings_count: u64,
    pps_count: u64,
    holdover_start: Option<u64>,
    lock_acquisitions: u32,
}

impl FemtosecondClockDriver {
    pub fn new(source: ClockSource) -> Self {
        Self {
            state: ClockState::Uninitialized,
            source,
            capabilities: ClockCapabilities::for_source(source),
            current_time_fs: 0,
            calibration: CalibrationData {
                offset_fs: 0,
                drift_rate_ppb: 0,
                last_calibration_tick: 0,
                calibration_count: 0,
                temperature_mK: 25_000,
            },
            readings_count: 0,
            pps_count: 0,
            holdover_start: None,
            lock_acquisitions: 0,
        }
    }

    pub fn initialize(&mut self) -> DriverResult<()> {
        self.state = ClockState::Warming;
        self.state = ClockState::Locking;
        self.state = ClockState::Locked;
        self.lock_acquisitions += 1;
        Ok(())
    }

    pub fn read(&mut self) -> DriverResult<ClockReading> {
        match self.state {
            ClockState::Uninitialized | ClockState::Error => {
                return Err(DriverError::HardwareNotPresent);
            }
            _ => {}
        }
        self.readings_count += 1;
        Ok(ClockReading {
            timestamp_fs: self.current_time_fs,
            source: self.source,
            state: self.state,
            uncertainty_fs: self.uncertainty(),
            sequence_number: self.readings_count,
        })
    }

    pub fn read_raw_fs(&self) -> u128 {
        self.current_time_fs
    }

    pub fn calibrate(&mut self, reference_fs: u128) -> DriverResult<()> {
        match self.state {
            ClockState::Locked | ClockState::Holdover | ClockState::Freerun => {}
            _ => return Err(DriverError::CalibrationFailed),
        }
        let current = self.current_time_fs as i128;
        let reference = reference_fs as i128;
        let diff = reference - current;
        self.calibration.offset_fs = diff as i64;
        self.calibration.calibration_count += 1;
        self.calibration.last_calibration_tick = self.readings_count;
        Ok(())
    }

    pub fn advance_time(&mut self, femtoseconds: u128) {
        self.current_time_fs += femtoseconds;
    }

    pub fn state(&self) -> &ClockState {
        &self.state
    }

    pub fn enter_holdover(&mut self) -> DriverResult<()> {
        if !self.capabilities.supports_holdover {
            return Err(DriverError::UnsupportedOperation);
        }
        match self.state {
            ClockState::Locked => {
                self.state = ClockState::Holdover;
                self.holdover_start = Some(self.readings_count);
                Ok(())
            }
            _ => Err(DriverError::InvalidConfiguration),
        }
    }

    pub fn exit_holdover(&mut self) -> DriverResult<()> {
        match self.state {
            ClockState::Holdover => {
                self.state = ClockState::Locking;
                self.state = ClockState::Locked;
                self.holdover_start = None;
                self.lock_acquisitions += 1;
                Ok(())
            }
            _ => Err(DriverError::InvalidConfiguration),
        }
    }

    pub fn pps_pulse(&mut self) {
        self.pps_count += 1;
    }

    pub fn drift_estimate(&self) -> i64 {
        self.calibration.drift_rate_ppb
    }

    pub fn uncertainty(&self) -> u64 {
        match self.state {
            ClockState::Locked => self.capabilities.resolution_femtoseconds,
            ClockState::Holdover => {
                self.capabilities.resolution_femtoseconds * 10
            }
            ClockState::Freerun => {
                self.capabilities.resolution_femtoseconds * 100
            }
            ClockState::Warming | ClockState::Locking => {
                self.capabilities.resolution_femtoseconds * 1000
            }
            _ => u64::MAX,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.state == ClockState::Locked
    }

    pub fn reset(&mut self) -> DriverResult<()> {
        self.state = ClockState::Uninitialized;
        self.current_time_fs = 0;
        self.calibration = CalibrationData {
            offset_fs: 0,
            drift_rate_ppb: 0,
            last_calibration_tick: 0,
            calibration_count: 0,
            temperature_mK: 25_000,
        };
        self.readings_count = 0;
        self.pps_count = 0;
        self.holdover_start = None;
        self.lock_acquisitions = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_new_crystal() {
        let clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        assert_eq!(*clock.state(), ClockState::Uninitialized);
        assert_eq!(clock.read_raw_fs(), 0);
        assert!(!clock.is_locked());
    }

    #[test]
    fn test_clock_new_cesium() {
        let clock = FemtosecondClockDriver::new(ClockSource::GpsdoCesium);
        assert_eq!(clock.capabilities.accuracy_ppb, 1);
        assert!(clock.capabilities.supports_pps);
        assert!(clock.capabilities.supports_holdover);
    }

    #[test]
    fn test_clock_initialize() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        assert!(clock.initialize().is_ok());
        assert_eq!(*clock.state(), ClockState::Locked);
        assert!(clock.is_locked());
    }

    #[test]
    fn test_clock_read_uninitialized_fails() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        assert!(clock.read().is_err());
    }

    #[test]
    fn test_clock_read_after_init() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        clock.initialize().unwrap();
        let reading = clock.read().unwrap();
        assert_eq!(reading.timestamp_fs, 0);
        assert_eq!(reading.source, ClockSource::InternalCrystal);
        assert_eq!(reading.state, ClockState::Locked);
        assert_eq!(reading.sequence_number, 1);
    }

    #[test]
    fn test_clock_read_increments_sequence() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        clock.initialize().unwrap();
        clock.read().unwrap();
        let reading = clock.read().unwrap();
        assert_eq!(reading.sequence_number, 2);
    }

    #[test]
    fn test_clock_advance_time() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        clock.initialize().unwrap();
        clock.advance_time(1_000_000_000_000_000);
        let reading = clock.read().unwrap();
        assert_eq!(reading.timestamp_fs, 1_000_000_000_000_000);
    }

    #[test]
    fn test_clock_read_raw_fs() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        clock.advance_time(42);
        assert_eq!(clock.read_raw_fs(), 42);
    }

    #[test]
    fn test_clock_calibrate() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        clock.advance_time(1000);
        assert!(clock.calibrate(1050).is_ok());
    }

    #[test]
    fn test_clock_calibrate_uninitialized_fails() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        assert!(clock.calibrate(0).is_err());
    }

    #[test]
    fn test_clock_holdover_enter_exit() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        assert!(clock.enter_holdover().is_ok());
        assert_eq!(*clock.state(), ClockState::Holdover);
        assert!(!clock.is_locked());
        assert!(clock.exit_holdover().is_ok());
        assert_eq!(*clock.state(), ClockState::Locked);
        assert!(clock.is_locked());
    }

    #[test]
    fn test_clock_holdover_unsupported() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        clock.initialize().unwrap();
        assert!(clock.enter_holdover().is_err());
    }

    #[test]
    fn test_clock_holdover_wrong_state() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        assert!(clock.enter_holdover().is_err());
    }

    #[test]
    fn test_clock_exit_holdover_wrong_state() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        assert!(clock.exit_holdover().is_err());
    }

    #[test]
    fn test_clock_pps_pulse() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.pps_pulse();
        clock.pps_pulse();
        clock.pps_pulse();
    }

    #[test]
    fn test_clock_drift_estimate() {
        let clock = FemtosecondClockDriver::new(ClockSource::InternalCrystal);
        assert_eq!(clock.drift_estimate(), 0);
    }

    #[test]
    fn test_clock_uncertainty_locked() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoCesium);
        clock.initialize().unwrap();
        assert_eq!(clock.uncertainty(), clock.capabilities.resolution_femtoseconds);
    }

    #[test]
    fn test_clock_uncertainty_holdover() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        let locked_uncertainty = clock.uncertainty();
        clock.enter_holdover().unwrap();
        assert!(clock.uncertainty() > locked_uncertainty);
    }

    #[test]
    fn test_clock_reset() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        clock.advance_time(1000);
        clock.read().unwrap();
        clock.pps_pulse();
        assert!(clock.reset().is_ok());
        assert_eq!(*clock.state(), ClockState::Uninitialized);
        assert_eq!(clock.read_raw_fs(), 0);
        assert!(!clock.is_locked());
    }

    #[test]
    fn test_clock_optical_lattice_capabilities() {
        let clock = FemtosecondClockDriver::new(ClockSource::OpticalLattice);
        assert_eq!(clock.capabilities.resolution_femtoseconds, 1);
        assert_eq!(clock.capabilities.accuracy_ppb, 0);
        assert!(!clock.capabilities.supports_holdover);
    }

    #[test]
    fn test_clock_chip_scale_capabilities() {
        let clock = FemtosecondClockDriver::new(ClockSource::ChipScale);
        assert_eq!(clock.capabilities.resolution_femtoseconds, 10_000);
        assert_eq!(clock.capabilities.holdover_stability_hours, 4);
    }

    #[test]
    fn test_clock_reading_during_holdover() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        clock.advance_time(500);
        clock.enter_holdover().unwrap();
        let reading = clock.read().unwrap();
        assert_eq!(reading.state, ClockState::Holdover);
        assert_eq!(reading.timestamp_fs, 500);
    }

    #[test]
    fn test_clock_calibrate_during_holdover() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        clock.enter_holdover().unwrap();
        assert!(clock.calibrate(100).is_ok());
    }

    #[test]
    fn test_clock_lock_acquisitions() {
        let mut clock = FemtosecondClockDriver::new(ClockSource::GpsdoRubidium);
        clock.initialize().unwrap();
        clock.enter_holdover().unwrap();
        clock.exit_holdover().unwrap();
        clock.enter_holdover().unwrap();
        clock.exit_holdover().unwrap();
    }
}
