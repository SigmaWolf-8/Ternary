//! Process Context (CPU State) Management
//!
//! Defines the CPU register state saved/restored during context switches.
//! Supports both standard binary registers and ternary coprocessor state.
//! Context structures are architecture-portable for x86_64, aarch64, and
//! custom ternary hardware targets.

use alloc::vec::Vec;
use super::{ProcessId, ProcessResult, ProcessError};

#[derive(Debug, Clone, Default)]
pub struct GeneralRegisters {
    pub r0: u64,
    pub r1: u64,
    pub r2: u64,
    pub r3: u64,
    pub r4: u64,
    pub r5: u64,
    pub r6: u64,
    pub r7: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ControlRegisters {
    pub instruction_pointer: u64,
    pub stack_pointer: u64,
    pub base_pointer: u64,
    pub flags: u64,
    pub page_table_base: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TernaryRegisters {
    pub t0: [i8; 27],
    pub t1: [i8; 27],
    pub t2: [i8; 27],
    pub t3: [i8; 27],
    pub t4: [i8; 27],
    pub t5: [i8; 27],
    pub t6: [i8; 27],
    pub t7: [i8; 27],
    pub ternary_flags: u32,
    pub ternary_mode: u8,
}

impl TernaryRegisters {
    pub fn clear(&mut self) {
        self.t0 = [0; 27];
        self.t1 = [0; 27];
        self.t2 = [0; 27];
        self.t3 = [0; 27];
        self.t4 = [0; 27];
        self.t5 = [0; 27];
        self.t6 = [0; 27];
        self.t7 = [0; 27];
        self.ternary_flags = 0;
        self.ternary_mode = 0;
    }

    pub fn register(&self, index: usize) -> Option<&[i8; 27]> {
        match index {
            0 => Some(&self.t0),
            1 => Some(&self.t1),
            2 => Some(&self.t2),
            3 => Some(&self.t3),
            4 => Some(&self.t4),
            5 => Some(&self.t5),
            6 => Some(&self.t6),
            7 => Some(&self.t7),
            _ => None,
        }
    }

    pub fn register_mut(&mut self, index: usize) -> Option<&mut [i8; 27]> {
        match index {
            0 => Some(&mut self.t0),
            1 => Some(&mut self.t1),
            2 => Some(&mut self.t2),
            3 => Some(&mut self.t3),
            4 => Some(&mut self.t4),
            5 => Some(&mut self.t5),
            6 => Some(&mut self.t6),
            7 => Some(&mut self.t7),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessContext {
    pub general: GeneralRegisters,
    pub control: ControlRegisters,
    pub ternary: TernaryRegisters,
    pub has_ternary_state: bool,
}

impl ProcessContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_kernel(entry_point: u64, stack_top: u64) -> Self {
        Self {
            general: GeneralRegisters::default(),
            control: ControlRegisters {
                instruction_pointer: entry_point,
                stack_pointer: stack_top,
                base_pointer: stack_top,
                flags: 0x0002, // Interrupt flag only
                page_table_base: 0,
            },
            ternary: TernaryRegisters::default(),
            has_ternary_state: false,
        }
    }

    pub fn new_user(entry_point: u64, stack_top: u64, page_table: u64) -> Self {
        Self {
            general: GeneralRegisters::default(),
            control: ControlRegisters {
                instruction_pointer: entry_point,
                stack_pointer: stack_top,
                base_pointer: stack_top,
                flags: 0x0202, // Interrupt flag + User mode
                page_table_base: page_table,
            },
            ternary: TernaryRegisters::default(),
            has_ternary_state: false,
        }
    }

    pub fn new_ternary_compute(entry_point: u64, stack_top: u64, page_table: u64) -> Self {
        Self {
            general: GeneralRegisters::default(),
            control: ControlRegisters {
                instruction_pointer: entry_point,
                stack_pointer: stack_top,
                base_pointer: stack_top,
                flags: 0x0202,
                page_table_base: page_table,
            },
            ternary: TernaryRegisters::default(),
            has_ternary_state: true,
        }
    }

    pub fn save_size(&self) -> usize {
        let base = core::mem::size_of::<GeneralRegisters>()
            + core::mem::size_of::<ControlRegisters>();
        if self.has_ternary_state {
            base + core::mem::size_of::<TernaryRegisters>()
        } else {
            base
        }
    }
}

pub struct ContextSwitch {
    pub from_pid: ProcessId,
    pub to_pid: ProcessId,
    pub from_context: ProcessContext,
    pub to_context: ProcessContext,
}

impl ContextSwitch {
    pub fn new(
        from_pid: ProcessId,
        to_pid: ProcessId,
        from_context: ProcessContext,
        to_context: ProcessContext,
    ) -> Self {
        Self {
            from_pid,
            to_pid,
            from_context,
            to_context,
        }
    }

    pub fn needs_ternary_save(&self) -> bool {
        self.from_context.has_ternary_state
    }

    pub fn needs_ternary_restore(&self) -> bool {
        self.to_context.has_ternary_state
    }

    pub fn needs_page_table_switch(&self) -> bool {
        self.from_context.control.page_table_base
            != self.to_context.control.page_table_base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_registers_default() {
        let regs = GeneralRegisters::default();
        assert_eq!(regs.r0, 0);
        assert_eq!(regs.r15, 0);
    }

    #[test]
    fn test_ternary_registers_clear() {
        let mut tregs = TernaryRegisters::default();
        tregs.t0[0] = 1;
        tregs.t3[5] = -1;
        tregs.ternary_flags = 0xFF;
        tregs.clear();
        assert_eq!(tregs.t0[0], 0);
        assert_eq!(tregs.t3[5], 0);
        assert_eq!(tregs.ternary_flags, 0);
    }

    #[test]
    fn test_ternary_register_access() {
        let mut tregs = TernaryRegisters::default();
        tregs.t0[0] = 1;
        tregs.t0[1] = -1;
        tregs.t0[2] = 0;

        let r = tregs.register(0).unwrap();
        assert_eq!(r[0], 1);
        assert_eq!(r[1], -1);
        assert_eq!(r[2], 0);

        assert!(tregs.register(8).is_none());
    }

    #[test]
    fn test_ternary_register_mut_access() {
        let mut tregs = TernaryRegisters::default();
        if let Some(r) = tregs.register_mut(3) {
            r[0] = 1;
            r[1] = 0;
            r[2] = -1;
        }
        assert_eq!(tregs.t3[0], 1);
        assert_eq!(tregs.t3[1], 0);
        assert_eq!(tregs.t3[2], -1);
    }

    #[test]
    fn test_context_new_kernel() {
        let ctx = ProcessContext::new_kernel(0x1000, 0x8000);
        assert_eq!(ctx.control.instruction_pointer, 0x1000);
        assert_eq!(ctx.control.stack_pointer, 0x8000);
        assert_eq!(ctx.control.base_pointer, 0x8000);
        assert!(!ctx.has_ternary_state);
    }

    #[test]
    fn test_context_new_user() {
        let ctx = ProcessContext::new_user(0x4000, 0xF000, 0x20000);
        assert_eq!(ctx.control.instruction_pointer, 0x4000);
        assert_eq!(ctx.control.stack_pointer, 0xF000);
        assert_eq!(ctx.control.page_table_base, 0x20000);
        assert!(!ctx.has_ternary_state);
    }

    #[test]
    fn test_context_new_ternary_compute() {
        let ctx = ProcessContext::new_ternary_compute(0x4000, 0xF000, 0x20000);
        assert!(ctx.has_ternary_state);
    }

    #[test]
    fn test_context_save_size() {
        let ctx_no_ternary = ProcessContext::new_kernel(0, 0);
        let ctx_with_ternary = ProcessContext::new_ternary_compute(0, 0, 0);
        assert!(ctx_with_ternary.save_size() > ctx_no_ternary.save_size());
    }

    #[test]
    fn test_context_switch_creation() {
        let from_ctx = ProcessContext::new_user(0x1000, 0x8000, 0x10000);
        let to_ctx = ProcessContext::new_ternary_compute(0x2000, 0x9000, 0x20000);

        let cs = ContextSwitch::new(1, 2, from_ctx, to_ctx);
        assert_eq!(cs.from_pid, 1);
        assert_eq!(cs.to_pid, 2);
    }

    #[test]
    fn test_context_switch_ternary_flags() {
        let from_ctx = ProcessContext::new_user(0x1000, 0x8000, 0x10000);
        let to_ctx = ProcessContext::new_ternary_compute(0x2000, 0x9000, 0x20000);

        let cs = ContextSwitch::new(1, 2, from_ctx, to_ctx);
        assert!(!cs.needs_ternary_save());
        assert!(cs.needs_ternary_restore());
    }

    #[test]
    fn test_context_switch_page_table() {
        let from_ctx = ProcessContext::new_user(0x1000, 0x8000, 0x10000);
        let to_ctx = ProcessContext::new_user(0x2000, 0x9000, 0x20000);
        let cs = ContextSwitch::new(1, 2, from_ctx, to_ctx);
        assert!(cs.needs_page_table_switch());

        let from_ctx2 = ProcessContext::new_user(0x1000, 0x8000, 0x10000);
        let to_ctx2 = ProcessContext::new_user(0x2000, 0x9000, 0x10000);
        let cs2 = ContextSwitch::new(1, 2, from_ctx2, to_ctx2);
        assert!(!cs2.needs_page_table_switch());
    }
}
