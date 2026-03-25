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

pub mod gateway;
pub mod adapter;
pub mod crypto_interop;
pub mod syscall_shim;

use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone)]
pub enum CompatError {
    ConversionFailed(String),
    InvalidBinaryData,
    InvalidTernaryData,
    BufferTooSmall { needed: usize, available: usize },
    UnsupportedType(String),
    GatewayError(String),
    AdapterError(String),
    OverflowError,
    PrecisionLoss { bits_lost: u32 },
    EncodingError(String),
}

impl fmt::Display for CompatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatError::ConversionFailed(msg) => write!(f, "Conversion failed: {}", msg),
            CompatError::InvalidBinaryData => write!(f, "Invalid binary data"),
            CompatError::InvalidTernaryData => write!(f, "Invalid ternary data"),
            CompatError::BufferTooSmall { needed, available } => {
                write!(f, "Buffer too small: needed {}, available {}", needed, available)
            }
            CompatError::UnsupportedType(msg) => write!(f, "Unsupported type: {}", msg),
            CompatError::GatewayError(msg) => write!(f, "Gateway error: {}", msg),
            CompatError::AdapterError(msg) => write!(f, "Adapter error: {}", msg),
            CompatError::OverflowError => write!(f, "Overflow error"),
            CompatError::PrecisionLoss { bits_lost } => {
                write!(f, "Precision loss: {} bits lost", bits_lost)
            }
            CompatError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
        }
    }
}

pub type CompatResult<T> = core::result::Result<T, CompatError>;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

    #[test]
    fn test_compat_error_display_conversion_failed() {
        let err = CompatError::ConversionFailed("test".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Conversion failed"));
        assert!(msg.contains("test"));
    }

    #[test]
    fn test_compat_error_display_buffer_too_small() {
        let err = CompatError::BufferTooSmall { needed: 10, available: 5 };
        let msg = format!("{}", err);
        assert!(msg.contains("10"));
        assert!(msg.contains("5"));
    }

    #[test]
    fn test_compat_error_display_precision_loss() {
        let err = CompatError::PrecisionLoss { bits_lost: 3 };
        let msg = format!("{}", err);
        assert!(msg.contains("3"));
    }

    #[test]
    fn test_compat_error_debug() {
        let err = CompatError::InvalidBinaryData;
        let msg = format!("{:?}", err);
        assert!(msg.contains("InvalidBinaryData"));
    }

    #[test]
    fn test_compat_result_ok_and_err() {
        let ok: CompatResult<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);
        let err: CompatResult<i32> = Err(CompatError::OverflowError);
        assert!(err.is_err());
    }
}
