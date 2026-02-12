// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

pub mod tpu;
pub mod femtoclock;

use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone)]
pub enum DriverError {
    HardwareNotPresent,
    FirmwareLoadFailed,
    DmaError(String),
    CommandQueueFull,
    CommandTimeout,
    InvalidConfiguration,
    CalibrationFailed,
    ClockDrift,
    UnsupportedOperation,
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverError::HardwareNotPresent => write!(f, "Hardware not present"),
            DriverError::FirmwareLoadFailed => write!(f, "Firmware load failed"),
            DriverError::DmaError(msg) => write!(f, "DMA error: {}", msg),
            DriverError::CommandQueueFull => write!(f, "Command queue full"),
            DriverError::CommandTimeout => write!(f, "Command timeout"),
            DriverError::InvalidConfiguration => write!(f, "Invalid configuration"),
            DriverError::CalibrationFailed => write!(f, "Calibration failed"),
            DriverError::ClockDrift => write!(f, "Clock drift detected"),
            DriverError::UnsupportedOperation => write!(f, "Unsupported operation"),
        }
    }
}

pub type DriverResult<T> = Result<T, DriverError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_not_present_error() {
        let err = DriverError::HardwareNotPresent;
        let msg = alloc::format!("{}", err);
        assert_eq!(msg, "Hardware not present");
    }

    #[test]
    fn test_firmware_load_failed_error() {
        let err = DriverError::FirmwareLoadFailed;
        let msg = alloc::format!("{}", err);
        assert_eq!(msg, "Firmware load failed");
    }

    #[test]
    fn test_dma_error_with_message() {
        let err = DriverError::DmaError(String::from("channel overflow"));
        let msg = alloc::format!("{}", err);
        assert_eq!(msg, "DMA error: channel overflow");
    }

    #[test]
    fn test_command_queue_full_error() {
        let err = DriverError::CommandQueueFull;
        let msg = alloc::format!("{}", err);
        assert_eq!(msg, "Command queue full");
    }

    #[test]
    fn test_error_debug_format() {
        let errors: alloc::vec::Vec<DriverError> = alloc::vec![
            DriverError::CommandTimeout,
            DriverError::InvalidConfiguration,
            DriverError::CalibrationFailed,
            DriverError::ClockDrift,
            DriverError::UnsupportedOperation,
        ];
        for err in &errors {
            let debug = alloc::format!("{:?}", err);
            assert!(!debug.is_empty());
        }
    }
}
