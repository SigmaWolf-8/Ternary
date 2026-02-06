pub mod inode;
pub mod dir;
pub mod file;
pub mod mount;

use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    IsDirectory,
    PermissionDenied,
    NoSpace,
    InvalidName,
    NotEmpty,
    ReadOnly,
    NotMounted,
    AlreadyMounted,
    InvalidOffset,
    InvalidInode,
    LinkLimitExceeded,
    CapacityExceeded,
    NameTooLong,
    PathTooLong,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::NotFound => write!(f, "Not found"),
            FsError::AlreadyExists => write!(f, "Already exists"),
            FsError::NotADirectory => write!(f, "Not a directory"),
            FsError::NotAFile => write!(f, "Not a file"),
            FsError::IsDirectory => write!(f, "Is a directory"),
            FsError::PermissionDenied => write!(f, "Permission denied"),
            FsError::NoSpace => write!(f, "No space left"),
            FsError::InvalidName => write!(f, "Invalid name"),
            FsError::NotEmpty => write!(f, "Directory not empty"),
            FsError::ReadOnly => write!(f, "Read-only filesystem"),
            FsError::NotMounted => write!(f, "Not mounted"),
            FsError::AlreadyMounted => write!(f, "Already mounted"),
            FsError::InvalidOffset => write!(f, "Invalid offset"),
            FsError::InvalidInode => write!(f, "Invalid inode"),
            FsError::LinkLimitExceeded => write!(f, "Link limit exceeded"),
            FsError::CapacityExceeded => write!(f, "Capacity exceeded"),
            FsError::NameTooLong => write!(f, "Name too long"),
            FsError::PathTooLong => write!(f, "Path too long"),
        }
    }
}

pub type FsResult<T> = Result<T, FsError>;

pub const MAX_NAME_LEN: usize = 255;
pub const MAX_PATH_LEN: usize = 4096;

pub fn validate_name(name: &str) -> FsResult<()> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(FsError::InvalidName);
    }
    if name.len() > MAX_NAME_LEN {
        return Err(FsError::NameTooLong);
    }
    if name.contains('/') || name.contains('\0') {
        return Err(FsError::InvalidName);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(validate_name("hello").is_ok());
        assert!(validate_name("file.txt").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name(".").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name("a/b").is_err());
    }

    #[test]
    fn test_name_too_long() {
        let long_name: String = core::iter::repeat('a').take(256).collect();
        assert_eq!(validate_name(&long_name), Err(FsError::NameTooLong));
    }

    #[test]
    fn test_fs_error_display() {
        let e = FsError::NotFound;
        let s = alloc::format!("{}", e);
        assert_eq!(s, "Not found");
    }
}
