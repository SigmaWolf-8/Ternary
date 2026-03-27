use alloc::string::String;

pub fn var(_key: &str) -> Result<String, VarError> {
    Err(VarError::NotPresent)
}

pub fn var_os(_key: &str) -> Option<String> {
    None
}

pub fn set_var(_key: &str, _value: &str) {}

pub fn remove_var(_key: &str) {}

pub fn vars() -> Vars {
    Vars
}

pub fn args() -> Args {
    Args { done: false }
}

pub fn args_os() -> Args {
    Args { done: false }
}

pub fn current_dir() -> crate::io::Result<String> {
    Ok(String::from("/"))
}

pub fn current_exe() -> crate::io::Result<String> {
    Ok(String::from("/plenumnet-kernel"))
}

pub fn temp_dir() -> String {
    String::from("/tmp")
}

pub fn home_dir() -> Option<String> {
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarError {
    NotPresent,
    NotUnicode(String),
}

impl core::fmt::Display for VarError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VarError::NotPresent => write!(f, "environment variable not found"),
            VarError::NotUnicode(s) => write!(f, "environment variable was not valid unicode: {}", s),
        }
    }
}

pub struct Vars;

impl Iterator for Vars {
    type Item = (String, String);
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct Args {
    done: bool,
}

impl Iterator for Args {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.done {
            self.done = true;
            Some(String::from("plenumnet-kernel"))
        } else {
            None
        }
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        if self.done { 0 } else { 1 }
    }
}

pub const ARCH: &str = {
    #[cfg(target_arch = "x86_64")]
    { "x86_64" }
    #[cfg(target_arch = "aarch64")]
    { "aarch64" }
    #[cfg(target_arch = "riscv64")]
    { "riscv64gc" }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    { "unknown" }
};

pub const FAMILY: &str = "plenum";
pub const OS: &str = "plenumnet";
