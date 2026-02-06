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
