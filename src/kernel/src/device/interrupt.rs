use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::{DeviceId, DeviceError, DeviceResult};

pub type IrqLine = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqMode {
    Edge,
    Level,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqStatus {
    Handled,
    NotHandled,
    Deferred,
}

#[derive(Debug, Clone)]
pub struct IrqDescriptor {
    pub line: IrqLine,
    pub mode: IrqMode,
    pub priority: IrqPriority,
    pub device_id: DeviceId,
    pub enabled: bool,
    pub trigger_count: u64,
}

impl IrqDescriptor {
    pub fn new(line: IrqLine, mode: IrqMode, priority: IrqPriority, device_id: DeviceId) -> Self {
        Self {
            line,
            mode,
            priority,
            device_id,
            enabled: false,
            trigger_count: 0,
        }
    }
}

pub struct InterruptController {
    handlers: BTreeMap<IrqLine, Vec<IrqDescriptor>>,
    masked: Vec<IrqLine>,
    global_enabled: bool,
    max_irq: IrqLine,
    pending: Vec<IrqLine>,
}

impl InterruptController {
    pub fn new(max_irq: IrqLine) -> Self {
        Self {
            handlers: BTreeMap::new(),
            masked: Vec::new(),
            global_enabled: true,
            max_irq,
            pending: Vec::new(),
        }
    }

    pub fn register_handler(&mut self, line: IrqLine, mode: IrqMode, priority: IrqPriority, device_id: DeviceId) -> DeviceResult<()> {
        if line > self.max_irq {
            return Err(DeviceError::InvalidParameter);
        }

        let desc = IrqDescriptor::new(line, mode, priority, device_id);
        self.handlers.entry(line).or_insert_with(Vec::new).push(desc);
        Ok(())
    }

    pub fn unregister_handler(&mut self, line: IrqLine, device_id: DeviceId) -> DeviceResult<()> {
        if let Some(handlers) = self.handlers.get_mut(&line) {
            let initial_len = handlers.len();
            handlers.retain(|h| h.device_id != device_id);
            if handlers.len() == initial_len {
                return Err(DeviceError::NotFound);
            }
            if handlers.is_empty() {
                self.handlers.remove(&line);
            }
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn enable_irq(&mut self, line: IrqLine) -> DeviceResult<()> {
        if let Some(handlers) = self.handlers.get_mut(&line) {
            for h in handlers.iter_mut() {
                h.enabled = true;
            }
            self.masked.retain(|&l| l != line);
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn disable_irq(&mut self, line: IrqLine) -> DeviceResult<()> {
        if let Some(handlers) = self.handlers.get_mut(&line) {
            for h in handlers.iter_mut() {
                h.enabled = false;
            }
            if !self.masked.contains(&line) {
                self.masked.push(line);
            }
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn trigger_irq(&mut self, line: IrqLine) -> DeviceResult<IrqStatus> {
        if !self.global_enabled {
            return Ok(IrqStatus::NotHandled);
        }
        if self.masked.contains(&line) {
            if !self.pending.contains(&line) {
                self.pending.push(line);
            }
            return Ok(IrqStatus::Deferred);
        }

        if let Some(handlers) = self.handlers.get_mut(&line) {
            let mut any_handled = false;
            for h in handlers.iter_mut() {
                if h.enabled {
                    h.trigger_count += 1;
                    any_handled = true;
                }
            }
            if any_handled {
                Ok(IrqStatus::Handled)
            } else {
                Ok(IrqStatus::NotHandled)
            }
        } else {
            Ok(IrqStatus::NotHandled)
        }
    }

    pub fn set_global_enabled(&mut self, enabled: bool) {
        self.global_enabled = enabled;
    }

    pub fn is_global_enabled(&self) -> bool {
        self.global_enabled
    }

    pub fn pending_irqs(&self) -> &[IrqLine] {
        &self.pending
    }

    pub fn process_pending(&mut self) -> Vec<(IrqLine, IrqStatus)> {
        let pending: Vec<IrqLine> = self.pending.drain(..).collect();
        let mut results = Vec::new();
        for line in pending {
            if let Ok(status) = self.trigger_irq(line) {
                results.push((line, status));
            }
        }
        results
    }

    pub fn handler_count(&self, line: IrqLine) -> usize {
        self.handlers.get(&line).map_or(0, |h| h.len())
    }

    pub fn total_triggers(&self, line: IrqLine) -> u64 {
        self.handlers
            .get(&line)
            .map_or(0, |handlers| handlers.iter().map(|h| h.trigger_count).sum())
    }

    pub fn registered_lines(&self) -> Vec<IrqLine> {
        self.handlers.keys().copied().collect()
    }

    pub fn handlers_for_device(&self, device_id: DeviceId) -> Vec<IrqLine> {
        let mut lines = Vec::new();
        for (line, handlers) in &self.handlers {
            if handlers.iter().any(|h| h.device_id == device_id) {
                lines.push(*line);
            }
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_handler() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(1, IrqMode::Edge, IrqPriority::Normal, DeviceId(1)).unwrap();
        assert_eq!(ic.handler_count(1), 1);
    }

    #[test]
    fn test_register_handler_invalid_line() {
        let mut ic = InterruptController::new(15);
        assert_eq!(
            ic.register_handler(16, IrqMode::Edge, IrqPriority::Normal, DeviceId(1)),
            Err(DeviceError::InvalidParameter)
        );
    }

    #[test]
    fn test_shared_irq() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(5, IrqMode::Level, IrqPriority::Normal, DeviceId(1)).unwrap();
        ic.register_handler(5, IrqMode::Level, IrqPriority::Normal, DeviceId(2)).unwrap();
        assert_eq!(ic.handler_count(5), 2);
    }

    #[test]
    fn test_unregister_handler() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(1, IrqMode::Edge, IrqPriority::Normal, DeviceId(1)).unwrap();
        ic.unregister_handler(1, DeviceId(1)).unwrap();
        assert_eq!(ic.handler_count(1), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(1, IrqMode::Edge, IrqPriority::Normal, DeviceId(1)).unwrap();
        ic.enable_irq(1).unwrap();
        ic.disable_irq(1).unwrap();
        let status = ic.trigger_irq(1).unwrap();
        assert_eq!(status, IrqStatus::Deferred);
    }

    #[test]
    fn test_trigger_irq() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(3, IrqMode::Edge, IrqPriority::High, DeviceId(1)).unwrap();
        ic.enable_irq(3).unwrap();
        let status = ic.trigger_irq(3).unwrap();
        assert_eq!(status, IrqStatus::Handled);
        assert_eq!(ic.total_triggers(3), 1);
    }

    #[test]
    fn test_global_disable() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(1, IrqMode::Edge, IrqPriority::Normal, DeviceId(1)).unwrap();
        ic.enable_irq(1).unwrap();
        ic.set_global_enabled(false);
        let status = ic.trigger_irq(1).unwrap();
        assert_eq!(status, IrqStatus::NotHandled);
    }

    #[test]
    fn test_pending_irqs() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(2, IrqMode::Edge, IrqPriority::Normal, DeviceId(1)).unwrap();
        ic.enable_irq(2).unwrap();
        ic.disable_irq(2).unwrap();
        ic.trigger_irq(2).unwrap();
        assert_eq!(ic.pending_irqs().len(), 1);
    }

    #[test]
    fn test_process_pending() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(2, IrqMode::Edge, IrqPriority::Normal, DeviceId(1)).unwrap();
        ic.enable_irq(2).unwrap();
        ic.disable_irq(2).unwrap();
        ic.trigger_irq(2).unwrap();
        ic.enable_irq(2).unwrap();
        let results = ic.process_pending();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, IrqStatus::Handled);
    }

    #[test]
    fn test_handlers_for_device() {
        let mut ic = InterruptController::new(255);
        ic.register_handler(1, IrqMode::Edge, IrqPriority::Normal, DeviceId(5)).unwrap();
        ic.register_handler(3, IrqMode::Level, IrqPriority::High, DeviceId(5)).unwrap();
        ic.register_handler(2, IrqMode::Edge, IrqPriority::Normal, DeviceId(6)).unwrap();
        let lines = ic.handlers_for_device(DeviceId(5));
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_unregistered_trigger() {
        let mut ic = InterruptController::new(255);
        let status = ic.trigger_irq(99).unwrap();
        assert_eq!(status, IrqStatus::NotHandled);
    }
}
