// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! FIPS 140-3 Module Finite State Machine
//!
//! Implements the deterministic state machine required by ISO/IEC 19790
//! Section 7.2. All state transitions are explicit, logged, and enforced.
//!
//! # States
//! - PowerOff: Module not loaded
//! - Uninitialized: Loaded, no self-tests run
//! - SelfTest: POST in progress
//! - Operational: All tests passed, crypto services available
//! - ApprovedMode: CNSA 2.0 only algorithms (FIPS approved)
//! - NonApprovedMode: Non-CNSA algorithms permitted
//! - Error: Self-test failed or critical error
//! - Zeroization: Key destruction in progress
//! - Shutdown: Module unloaded, all state cleared
//!
//! # FIPS 140-3 Requirement
//! The module MUST operate in a defined state at all times.
//! All crypto entry points check state == Operational (or sub-state)
//! before proceeding. Error state is terminal until module restart.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    PowerOff,
    Uninitialized,
    SelfTest,
    Operational,
    ApprovedMode,
    NonApprovedMode,
    Error,
    Zeroization,
    Shutdown,
}

impl core::fmt::Display for ModuleState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModuleState::PowerOff => write!(f, "Power Off"),
            ModuleState::Uninitialized => write!(f, "Uninitialized"),
            ModuleState::SelfTest => write!(f, "Self-Test"),
            ModuleState::Operational => write!(f, "Operational"),
            ModuleState::ApprovedMode => write!(f, "Approved Mode (CNSA 2.0)"),
            ModuleState::NonApprovedMode => write!(f, "Non-Approved Mode"),
            ModuleState::Error => write!(f, "Error"),
            ModuleState::Zeroization => write!(f, "Zeroization"),
            ModuleState::Shutdown => write!(f, "Shutdown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleEvent {
    Load,
    StartSelfTest,
    SelfTestPassed,
    SelfTestFailed,
    SetApprovedMode,
    SetNonApprovedMode,
    ConditionalTestFailed,
    IntegrityFailure,
    BeginZeroization,
    ZeroizationComplete,
    Unload,
    Reload,
}

impl core::fmt::Display for ModuleEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModuleEvent::Load => write!(f, "Module Load"),
            ModuleEvent::StartSelfTest => write!(f, "Start Self-Test"),
            ModuleEvent::SelfTestPassed => write!(f, "Self-Test Passed"),
            ModuleEvent::SelfTestFailed => write!(f, "Self-Test Failed"),
            ModuleEvent::SetApprovedMode => write!(f, "Set Approved Mode"),
            ModuleEvent::SetNonApprovedMode => write!(f, "Set Non-Approved Mode"),
            ModuleEvent::ConditionalTestFailed => write!(f, "Conditional Test Failed"),
            ModuleEvent::IntegrityFailure => write!(f, "Integrity Failure"),
            ModuleEvent::BeginZeroization => write!(f, "Begin Zeroization"),
            ModuleEvent::ZeroizationComplete => write!(f, "Zeroization Complete"),
            ModuleEvent::Unload => write!(f, "Module Unload"),
            ModuleEvent::Reload => write!(f, "Module Reload"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InvalidTransition {
    pub from: ModuleState,
    pub event: ModuleEvent,
    pub reason: String,
}

impl core::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid transition: {} --[{}]--> ?: {}", self.from, self.event, self.reason)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeIndicator {
    Approved,
    NonApproved,
    Error,
    NotOperational,
}

impl core::fmt::Display for ModeIndicator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModeIndicator::Approved => write!(f, "FIPS Approved Mode"),
            ModeIndicator::NonApproved => write!(f, "Non-Approved Mode"),
            ModeIndicator::Error => write!(f, "Error State"),
            ModeIndicator::NotOperational => write!(f, "Not Operational"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransitionRecord {
    pub from: ModuleState,
    pub to: ModuleState,
    pub event: ModuleEvent,
    pub sequence: u64,
}

pub struct ModuleStateMachine {
    state: ModuleState,
    transition_log: Vec<TransitionRecord>,
    sequence_counter: u64,
}

impl ModuleStateMachine {
    pub fn new() -> Self {
        Self {
            state: ModuleState::PowerOff,
            transition_log: Vec::new(),
            sequence_counter: 0,
        }
    }

    pub fn current_state(&self) -> ModuleState {
        self.state
    }

    pub fn transition(&mut self, event: ModuleEvent) -> Result<ModuleState, InvalidTransition> {
        let new_state = match (self.state, event) {
            (ModuleState::PowerOff, ModuleEvent::Load) => ModuleState::Uninitialized,

            (ModuleState::Uninitialized, ModuleEvent::StartSelfTest) => ModuleState::SelfTest,

            (ModuleState::SelfTest, ModuleEvent::SelfTestPassed) => ModuleState::Operational,
            (ModuleState::SelfTest, ModuleEvent::SelfTestFailed) => ModuleState::Error,

            (ModuleState::Operational, ModuleEvent::SetApprovedMode) => ModuleState::ApprovedMode,
            (ModuleState::Operational, ModuleEvent::SetNonApprovedMode) => ModuleState::NonApprovedMode,
            (ModuleState::Operational, ModuleEvent::ConditionalTestFailed) => ModuleState::Error,
            (ModuleState::Operational, ModuleEvent::IntegrityFailure) => ModuleState::Error,
            (ModuleState::Operational, ModuleEvent::BeginZeroization) => ModuleState::Zeroization,

            (ModuleState::ApprovedMode, ModuleEvent::SetNonApprovedMode) => ModuleState::NonApprovedMode,
            (ModuleState::ApprovedMode, ModuleEvent::ConditionalTestFailed) => ModuleState::Error,
            (ModuleState::ApprovedMode, ModuleEvent::IntegrityFailure) => ModuleState::Error,
            (ModuleState::ApprovedMode, ModuleEvent::BeginZeroization) => ModuleState::Zeroization,

            (ModuleState::NonApprovedMode, ModuleEvent::SetApprovedMode) => ModuleState::ApprovedMode,
            (ModuleState::NonApprovedMode, ModuleEvent::ConditionalTestFailed) => ModuleState::Error,
            (ModuleState::NonApprovedMode, ModuleEvent::IntegrityFailure) => ModuleState::Error,
            (ModuleState::NonApprovedMode, ModuleEvent::BeginZeroization) => ModuleState::Zeroization,

            (ModuleState::Error, ModuleEvent::Reload) => ModuleState::Uninitialized,

            (ModuleState::Zeroization, ModuleEvent::ZeroizationComplete) => ModuleState::Shutdown,

            (ModuleState::Shutdown, ModuleEvent::Unload) => ModuleState::PowerOff,

            (_, ModuleEvent::SelfTestFailed) => ModuleState::Error,
            (_, ModuleEvent::IntegrityFailure) => ModuleState::Error,

            (from, event) => {
                return Err(InvalidTransition {
                    from,
                    event,
                    reason: alloc::format!(
                        "No valid transition from {} on event {}",
                        from, event
                    ),
                });
            }
        };

        self.sequence_counter += 1;
        self.transition_log.push(TransitionRecord {
            from: self.state,
            to: new_state,
            event,
            sequence: self.sequence_counter,
        });
        self.state = new_state;
        Ok(new_state)
    }

    pub fn is_operational(&self) -> bool {
        matches!(
            self.state,
            ModuleState::Operational | ModuleState::ApprovedMode | ModuleState::NonApprovedMode
        )
    }

    pub fn is_approved_mode(&self) -> bool {
        self.state == ModuleState::ApprovedMode
    }

    pub fn get_mode_indicator(&self) -> ModeIndicator {
        match self.state {
            ModuleState::ApprovedMode => ModeIndicator::Approved,
            ModuleState::NonApprovedMode => ModeIndicator::NonApproved,
            ModuleState::Error => ModeIndicator::Error,
            ModuleState::Operational => ModeIndicator::NonApproved,
            _ => ModeIndicator::NotOperational,
        }
    }

    pub fn transition_log(&self) -> &[TransitionRecord] {
        &self.transition_log
    }

    pub fn check_ready(&self) -> Result<(), super::self_test::SelfTestError> {
        if self.is_operational() {
            Ok(())
        } else {
            Err(super::self_test::SelfTestError::ModuleNotReady)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_startup_sequence() {
        let mut sm = ModuleStateMachine::new();
        assert_eq!(sm.current_state(), ModuleState::PowerOff);

        sm.transition(ModuleEvent::Load).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Uninitialized);

        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        assert_eq!(sm.current_state(), ModuleState::SelfTest);

        sm.transition(ModuleEvent::SelfTestPassed).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Operational);
        assert!(sm.is_operational());
    }

    #[test]
    fn test_approved_mode_transitions() {
        let mut sm = ModuleStateMachine::new();
        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestPassed).unwrap();

        sm.transition(ModuleEvent::SetApprovedMode).unwrap();
        assert_eq!(sm.current_state(), ModuleState::ApprovedMode);
        assert!(sm.is_approved_mode());
        assert_eq!(sm.get_mode_indicator(), ModeIndicator::Approved);

        sm.transition(ModuleEvent::SetNonApprovedMode).unwrap();
        assert_eq!(sm.current_state(), ModuleState::NonApprovedMode);
        assert!(!sm.is_approved_mode());
        assert_eq!(sm.get_mode_indicator(), ModeIndicator::NonApproved);
    }

    #[test]
    fn test_self_test_failure() {
        let mut sm = ModuleStateMachine::new();
        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestFailed).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Error);
        assert!(!sm.is_operational());
        assert_eq!(sm.get_mode_indicator(), ModeIndicator::Error);
    }

    #[test]
    fn test_error_recovery_requires_reload() {
        let mut sm = ModuleStateMachine::new();
        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestFailed).unwrap();

        let result = sm.transition(ModuleEvent::SelfTestPassed);
        assert!(result.is_err());

        sm.transition(ModuleEvent::Reload).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Uninitialized);
    }

    #[test]
    fn test_conditional_test_failure_from_operational() {
        let mut sm = ModuleStateMachine::new();
        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestPassed).unwrap();

        sm.transition(ModuleEvent::ConditionalTestFailed).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Error);
    }

    #[test]
    fn test_zeroization_sequence() {
        let mut sm = ModuleStateMachine::new();
        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestPassed).unwrap();

        sm.transition(ModuleEvent::BeginZeroization).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Zeroization);

        sm.transition(ModuleEvent::ZeroizationComplete).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Shutdown);

        sm.transition(ModuleEvent::Unload).unwrap();
        assert_eq!(sm.current_state(), ModuleState::PowerOff);
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = ModuleStateMachine::new();
        let result = sm.transition(ModuleEvent::SelfTestPassed);
        assert!(result.is_err());
    }

    #[test]
    fn test_transition_log() {
        let mut sm = ModuleStateMachine::new();
        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestPassed).unwrap();

        let log = sm.transition_log();
        assert_eq!(log.len(), 3);
        assert_eq!(log[0].from, ModuleState::PowerOff);
        assert_eq!(log[0].to, ModuleState::Uninitialized);
        assert_eq!(log[2].to, ModuleState::Operational);
    }

    #[test]
    fn test_check_ready() {
        let mut sm = ModuleStateMachine::new();
        assert!(sm.check_ready().is_err());

        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestPassed).unwrap();
        assert!(sm.check_ready().is_ok());
    }

    #[test]
    fn test_integrity_failure_any_state() {
        let mut sm = ModuleStateMachine::new();
        sm.transition(ModuleEvent::Load).unwrap();
        sm.transition(ModuleEvent::StartSelfTest).unwrap();
        sm.transition(ModuleEvent::SelfTestPassed).unwrap();
        sm.transition(ModuleEvent::SetApprovedMode).unwrap();

        sm.transition(ModuleEvent::IntegrityFailure).unwrap();
        assert_eq!(sm.current_state(), ModuleState::Error);
    }

    #[test]
    fn test_state_display() {
        assert_eq!(alloc::format!("{}", ModuleState::ApprovedMode), "Approved Mode (CNSA 2.0)");
        assert_eq!(alloc::format!("{}", ModuleState::Error), "Error");
    }

    #[test]
    fn test_mode_indicator_display() {
        assert_eq!(alloc::format!("{}", ModeIndicator::Approved), "FIPS Approved Mode");
    }
}
