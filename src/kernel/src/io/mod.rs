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

pub mod scheduler;
pub mod buffer;
pub mod block;
pub mod chardev;
pub mod poll;

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoError {
    DeviceNotFound,
    BufferFull,
    BufferEmpty,
    InvalidOffset,
    InvalidSize,
    ReadOnly,
    WriteOnly,
    NotReady,
    Timeout,
    EndOfDevice,
    QueueFull,
    InvalidRequest,
    CacheMiss,
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::DeviceNotFound => write!(f, "Device not found"),
            IoError::BufferFull => write!(f, "Buffer full"),
            IoError::BufferEmpty => write!(f, "Buffer empty"),
            IoError::InvalidOffset => write!(f, "Invalid offset"),
            IoError::InvalidSize => write!(f, "Invalid size"),
            IoError::ReadOnly => write!(f, "Read-only"),
            IoError::WriteOnly => write!(f, "Write-only"),
            IoError::NotReady => write!(f, "Not ready"),
            IoError::Timeout => write!(f, "Timeout"),
            IoError::EndOfDevice => write!(f, "End of device"),
            IoError::QueueFull => write!(f, "Queue full"),
            IoError::InvalidRequest => write!(f, "Invalid request"),
            IoError::CacheMiss => write!(f, "Cache miss"),
        }
    }
}

pub type IoResult<T> = Result<T, IoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let e = IoError::BufferFull;
        let s = alloc::format!("{}", e);
        assert_eq!(s, "Buffer full");
    }
}
