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

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SyscallNumber {
    Socket = 41,
    Connect = 42,
    Accept = 43,
    SendTo = 44,
    RecvFrom = 45,
    Bind = 49,
    Listen = 50,
    GetSockOpt = 55,
    SetSockOpt = 54,
    Close = 3,
    Read = 0,
    Write = 1,
    Open = 2,
    GetRandom = 318,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShimAction {
    Redirect,
    Block,
    Allow,
    Inspect,
}

#[derive(Debug, Clone)]
pub struct SyscallInterception {
    pub syscall: SyscallNumber,
    pub action: ShimAction,
    pub process_id: u32,
    pub args: [u64; 6],
    pub return_value: i64,
}

#[derive(Debug, Clone)]
pub struct ShimPolicy {
    pub allow_local_network: bool,
    pub allow_filesystem_read: bool,
    pub allow_filesystem_write: bool,
    pub redirect_crypto: bool,
    pub redirect_network: bool,
    pub blocked_ports: Vec<u16>,
}

impl Default for ShimPolicy {
    fn default() -> Self {
        Self {
            allow_local_network: false,
            allow_filesystem_read: true,
            allow_filesystem_write: false,
            redirect_crypto: true,
            redirect_network: true,
            blocked_ports: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShimError {
    Blocked,
    Redirected,
    PolicyViolation,
    InvalidSyscall,
    GatewayUnavailable,
}

impl fmt::Display for ShimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShimError::Blocked => write!(f, "Syscall blocked by policy"),
            ShimError::Redirected => write!(f, "Syscall redirected through gateway"),
            ShimError::PolicyViolation => write!(f, "Policy violation"),
            ShimError::InvalidSyscall => write!(f, "Invalid syscall number"),
            ShimError::GatewayUnavailable => write!(f, "Gateway unavailable"),
        }
    }
}

pub type ShimResult<T> = core::result::Result<T, ShimError>;

pub struct SyscallShim {
    policy: ShimPolicy,
    interception_count: u64,
    blocked_count: u64,
    redirected_count: u64,
}

impl SyscallShim {
    pub fn new(policy: ShimPolicy) -> Self {
        Self {
            policy,
            interception_count: 0,
            blocked_count: 0,
            redirected_count: 0,
        }
    }

    pub fn classify(&self, syscall: SyscallNumber) -> ShimAction {
        match syscall {
            SyscallNumber::Socket | SyscallNumber::Connect |
            SyscallNumber::SendTo | SyscallNumber::RecvFrom |
            SyscallNumber::Bind | SyscallNumber::Listen => {
                if self.policy.redirect_network {
                    ShimAction::Redirect
                } else {
                    ShimAction::Block
                }
            }
            SyscallNumber::Accept => {
                if self.policy.allow_local_network {
                    ShimAction::Allow
                } else {
                    ShimAction::Redirect
                }
            }
            SyscallNumber::GetRandom => {
                if self.policy.redirect_crypto {
                    ShimAction::Redirect
                } else {
                    ShimAction::Allow
                }
            }
            SyscallNumber::Read => {
                if self.policy.allow_filesystem_read {
                    ShimAction::Allow
                } else {
                    ShimAction::Inspect
                }
            }
            SyscallNumber::Write => {
                if self.policy.allow_filesystem_write {
                    ShimAction::Allow
                } else {
                    ShimAction::Inspect
                }
            }
            SyscallNumber::Open => ShimAction::Inspect,
            SyscallNumber::Close => ShimAction::Allow,
            SyscallNumber::GetSockOpt | SyscallNumber::SetSockOpt => ShimAction::Allow,
        }
    }

    pub fn intercept(&mut self, syscall: SyscallNumber, args: [u64; 6], pid: u32) -> ShimResult<SyscallInterception> {
        self.interception_count += 1;
        let action = self.classify(syscall);

        let interception = SyscallInterception {
            syscall,
            action,
            process_id: pid,
            args,
            return_value: 0,
        };

        match action {
            ShimAction::Block => {
                self.blocked_count += 1;
                Err(ShimError::Blocked)
            }
            ShimAction::Redirect => {
                self.redirected_count += 1;
                Ok(interception)
            }
            ShimAction::Allow => Ok(interception),
            ShimAction::Inspect => Ok(interception),
        }
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (self.interception_count, self.blocked_count, self.redirected_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy_blocks_network() {
        let shim = SyscallShim::new(ShimPolicy::default());
        assert_eq!(shim.classify(SyscallNumber::Socket), ShimAction::Redirect);
        assert_eq!(shim.classify(SyscallNumber::Connect), ShimAction::Redirect);
        assert_eq!(shim.classify(SyscallNumber::SendTo), ShimAction::Redirect);
    }

    #[test]
    fn test_default_policy_allows_read() {
        let shim = SyscallShim::new(ShimPolicy::default());
        assert_eq!(shim.classify(SyscallNumber::Read), ShimAction::Allow);
        assert_eq!(shim.classify(SyscallNumber::Close), ShimAction::Allow);
    }

    #[test]
    fn test_default_policy_redirects_crypto() {
        let shim = SyscallShim::new(ShimPolicy::default());
        assert_eq!(shim.classify(SyscallNumber::GetRandom), ShimAction::Redirect);
    }

    #[test]
    fn test_intercept_updates_stats() {
        let mut shim = SyscallShim::new(ShimPolicy::default());
        let _ = shim.intercept(SyscallNumber::Socket, [0; 6], 1);
        let _ = shim.intercept(SyscallNumber::Read, [0; 6], 1);
        let (total, _, redirected) = shim.stats();
        assert_eq!(total, 2);
        assert_eq!(redirected, 1);
    }

    #[test]
    fn test_blocked_network_without_redirect() {
        let policy = ShimPolicy {
            redirect_network: false,
            ..ShimPolicy::default()
        };
        let mut shim = SyscallShim::new(policy);
        let result = shim.intercept(SyscallNumber::Connect, [0; 6], 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ShimError::Blocked);
    }

    #[test]
    fn test_shim_error_display() {
        let err = ShimError::Blocked;
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("blocked"));
    }
}
