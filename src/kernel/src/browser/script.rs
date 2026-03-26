// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// JavaScript execution — wraps boa_engine with cooperative watchdog.
// Phase 1: Executor interface with timeout/termination semantics.
// Boa runs in a dedicated kernel task with cooperative termination:
// should_terminate flag checked between bytecode instructions.

use alloc::string::String;
use core::fmt;

pub const DEFAULT_BUDGET_MS: u64 = 5000;
pub const MAX_BUDGET_MS: u64 = 30000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptStatus {
    Ready,
    Running,
    Completed,
    TimedOut,
    Terminated,
    Error,
}

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub status: ScriptStatus,
    pub value: Option<String>,
    pub error: Option<ScriptError>,
    pub execution_ms: u64,
}

#[derive(Debug, Clone)]
pub enum ScriptError {
    SyntaxError(String),
    RuntimeError(String),
    Timeout { budget_ms: u64 },
    Terminated,
    StackOverflow,
    OutOfMemory,
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            ScriptError::RuntimeError(msg) => write!(f, "RuntimeError: {}", msg),
            ScriptError::Timeout { budget_ms } => {
                write!(f, "Script exceeded {}ms budget", budget_ms)
            }
            ScriptError::Terminated => write!(f, "Script terminated by watchdog"),
            ScriptError::StackOverflow => write!(f, "Stack overflow in script execution"),
            ScriptError::OutOfMemory => write!(f, "Out of memory in script execution"),
        }
    }
}

pub struct ScriptExecutor {
    budget_ms: u64,
    should_terminate: bool,
    scripts_executed: u64,
}

impl ScriptExecutor {
    pub fn new(budget_ms: u64) -> Self {
        let budget = if budget_ms > MAX_BUDGET_MS {
            MAX_BUDGET_MS
        } else if budget_ms == 0 {
            DEFAULT_BUDGET_MS
        } else {
            budget_ms
        };

        Self {
            budget_ms: budget,
            should_terminate: false,
            scripts_executed: 0,
        }
    }

    pub fn execute(&mut self, _source: &str) -> ScriptResult {
        if self.should_terminate {
            return ScriptResult {
                status: ScriptStatus::Terminated,
                value: None,
                error: Some(ScriptError::Terminated),
                execution_ms: 0,
            };
        }

        self.scripts_executed += 1;

        ScriptResult {
            status: ScriptStatus::Completed,
            value: Some(String::from("undefined")),
            error: None,
            execution_ms: 0,
        }
    }

    pub fn terminate(&mut self) {
        self.should_terminate = true;
    }

    pub fn reset(&mut self) {
        self.should_terminate = false;
    }

    pub fn is_terminated(&self) -> bool {
        self.should_terminate
    }

    pub fn budget_ms(&self) -> u64 {
        self.budget_ms
    }

    pub fn scripts_executed(&self) -> u64 {
        self.scripts_executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let exec = ScriptExecutor::new(DEFAULT_BUDGET_MS);
        assert_eq!(exec.budget_ms(), DEFAULT_BUDGET_MS);
        assert!(!exec.is_terminated());
    }

    #[test]
    fn test_budget_clamping() {
        let exec = ScriptExecutor::new(0);
        assert_eq!(exec.budget_ms(), DEFAULT_BUDGET_MS);

        let exec = ScriptExecutor::new(999999);
        assert_eq!(exec.budget_ms(), MAX_BUDGET_MS);
    }

    #[test]
    fn test_execute_stub() {
        let mut exec = ScriptExecutor::new(5000);
        let result = exec.execute("console.log('hello')");
        assert_eq!(result.status, ScriptStatus::Completed);
        assert_eq!(exec.scripts_executed(), 1);
    }

    #[test]
    fn test_terminate() {
        let mut exec = ScriptExecutor::new(5000);
        exec.terminate();
        let result = exec.execute("anything");
        assert_eq!(result.status, ScriptStatus::Terminated);
    }

    #[test]
    fn test_reset_after_terminate() {
        let mut exec = ScriptExecutor::new(5000);
        exec.terminate();
        assert!(exec.is_terminated());
        exec.reset();
        assert!(!exec.is_terminated());
    }
}
