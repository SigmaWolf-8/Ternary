pub mod protocol;
pub mod optical;
pub mod certification;

use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone)]
pub enum HptpError {
    SyncFailed(String),
    ClockDrift { measured_fs: i128, max_allowed_fs: i128 },
    CalibrationFailed,
    CertificationFailed(String),
    ProtocolViolation(String),
    TimestampOutOfRange,
    ClockSourceUnavailable,
    InvalidPrecision { requested_fs: u64, available_fs: u64 },
    NetworkTimeout,
    PeerUnreachable(String),
}

impl fmt::Display for HptpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HptpError::SyncFailed(msg) => write!(f, "Sync failed: {}", msg),
            HptpError::ClockDrift { measured_fs, max_allowed_fs } => {
                write!(f, "Clock drift: measured {}fs, max allowed {}fs", measured_fs, max_allowed_fs)
            }
            HptpError::CalibrationFailed => write!(f, "Calibration failed"),
            HptpError::CertificationFailed(msg) => write!(f, "Certification failed: {}", msg),
            HptpError::ProtocolViolation(msg) => write!(f, "Protocol violation: {}", msg),
            HptpError::TimestampOutOfRange => write!(f, "Timestamp out of range"),
            HptpError::ClockSourceUnavailable => write!(f, "Clock source unavailable"),
            HptpError::InvalidPrecision { requested_fs, available_fs } => {
                write!(f, "Invalid precision: requested {}fs, available {}fs", requested_fs, available_fs)
            }
            HptpError::NetworkTimeout => write!(f, "Network timeout"),
            HptpError::PeerUnreachable(msg) => write!(f, "Peer unreachable: {}", msg),
        }
    }
}

pub type HptpResult<T> = core::result::Result<T, HptpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hptp_error_display_sync_failed() {
        let err = HptpError::SyncFailed(String::from("timeout"));
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("Sync failed"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_hptp_error_display_clock_drift() {
        let err = HptpError::ClockDrift { measured_fs: 200, max_allowed_fs: 100 };
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("200"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_hptp_error_display_calibration_failed() {
        let err = HptpError::CalibrationFailed;
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("Calibration failed"));
    }

    #[test]
    fn test_hptp_error_display_invalid_precision() {
        let err = HptpError::InvalidPrecision { requested_fs: 1, available_fs: 1000 };
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("1"));
        assert!(msg.contains("1000"));
    }

    #[test]
    fn test_hptp_error_display_peer_unreachable() {
        let err = HptpError::PeerUnreachable(String::from("node-42"));
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("Peer unreachable"));
        assert!(msg.contains("node-42"));
    }
}
