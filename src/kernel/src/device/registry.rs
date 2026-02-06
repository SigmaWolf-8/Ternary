use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{DeviceId, DeviceDescriptor, DeviceInfo, DeviceCapabilities, DeviceState, DeviceType, DeviceClass, DeviceError, DeviceResult, OperationMode};

pub struct DeviceRegistry {
    devices: BTreeMap<u32, DeviceDescriptor>,
    next_id: u32,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn register(&mut self, name: String, device_type: DeviceType, device_class: DeviceClass, capabilities: DeviceCapabilities) -> DeviceResult<DeviceId> {
        let id = DeviceId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(DeviceError::CapacityExceeded)?;

        let info = DeviceInfo {
            id,
            name,
            device_type,
            device_class,
            vendor_id: 0,
            product_id: 0,
            revision: 0,
        };

        let desc = DeviceDescriptor::new(info, capabilities);
        self.devices.insert(id.0, desc);
        Ok(id)
    }

    pub fn unregister(&mut self, id: DeviceId) -> DeviceResult<()> {
        self.devices.remove(&id.0).ok_or(DeviceError::NotFound)?;
        Ok(())
    }

    pub fn get(&self, id: DeviceId) -> DeviceResult<&DeviceDescriptor> {
        self.devices.get(&id.0).ok_or(DeviceError::NotFound)
    }

    pub fn get_mut(&mut self, id: DeviceId) -> DeviceResult<&mut DeviceDescriptor> {
        self.devices.get_mut(&id.0).ok_or(DeviceError::NotFound)
    }

    pub fn initialize(&mut self, id: DeviceId) -> DeviceResult<()> {
        let dev = self.get_mut(id)?;
        dev.transition(DeviceState::Initializing)?;
        dev.transition(DeviceState::Ready)?;
        Ok(())
    }

    pub fn suspend(&mut self, id: DeviceId) -> DeviceResult<()> {
        let dev = self.get_mut(id)?;
        dev.transition(DeviceState::Suspended)?;
        Ok(())
    }

    pub fn resume(&mut self, id: DeviceId) -> DeviceResult<()> {
        let dev = self.get_mut(id)?;
        dev.transition(DeviceState::Ready)?;
        Ok(())
    }

    pub fn remove(&mut self, id: DeviceId) -> DeviceResult<()> {
        let dev = self.get_mut(id)?;
        dev.transition(DeviceState::Removed)?;
        self.devices.remove(&id.0);
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.devices.len()
    }

    pub fn find_by_type(&self, device_type: DeviceType) -> Vec<DeviceId> {
        self.devices
            .values()
            .filter(|d| d.info.device_type == device_type)
            .map(|d| d.info.id)
            .collect()
    }

    pub fn find_by_class(&self, device_class: DeviceClass) -> Vec<DeviceId> {
        self.devices
            .values()
            .filter(|d| d.info.device_class == device_class)
            .map(|d| d.info.id)
            .collect()
    }

    pub fn find_by_state(&self, state: DeviceState) -> Vec<DeviceId> {
        self.devices
            .values()
            .filter(|d| d.state == state)
            .map(|d| d.info.id)
            .collect()
    }

    pub fn find_ready(&self) -> Vec<DeviceId> {
        self.find_by_state(DeviceState::Ready)
    }

    pub fn all_ids(&self) -> Vec<DeviceId> {
        self.devices.values().map(|d| d.info.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_caps() -> DeviceCapabilities {
        DeviceCapabilities {
            dma_capable: false,
            interrupt_capable: true,
            ternary_native: false,
            max_transfer_size: 4096,
            supported_modes: alloc::vec![OperationMode::Polled],
        }
    }

    #[test]
    fn test_register_device() {
        let mut reg = DeviceRegistry::new();
        let id = reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        assert_eq!(reg.count(), 1);
        let dev = reg.get(id).unwrap();
        assert_eq!(dev.info.name, "blk0");
        assert_eq!(dev.state, DeviceState::Uninitialized);
    }

    #[test]
    fn test_unregister_device() {
        let mut reg = DeviceRegistry::new();
        let id = reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        reg.unregister(id).unwrap();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_initialize_device() {
        let mut reg = DeviceRegistry::new();
        let id = reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        reg.initialize(id).unwrap();
        assert_eq!(reg.get(id).unwrap().state, DeviceState::Ready);
    }

    #[test]
    fn test_suspend_resume() {
        let mut reg = DeviceRegistry::new();
        let id = reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        reg.initialize(id).unwrap();
        reg.suspend(id).unwrap();
        assert_eq!(reg.get(id).unwrap().state, DeviceState::Suspended);
        reg.resume(id).unwrap();
        assert_eq!(reg.get(id).unwrap().state, DeviceState::Ready);
    }

    #[test]
    fn test_remove_device() {
        let mut reg = DeviceRegistry::new();
        let id = reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        reg.initialize(id).unwrap();
        reg.remove(id).unwrap();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_find_by_type() {
        let mut reg = DeviceRegistry::new();
        reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        reg.register(String::from("chr0"), DeviceType::Character, DeviceClass::Communication, default_caps()).unwrap();
        reg.register(String::from("blk1"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        assert_eq!(reg.find_by_type(DeviceType::Block).len(), 2);
        assert_eq!(reg.find_by_type(DeviceType::Character).len(), 1);
    }

    #[test]
    fn test_find_by_class() {
        let mut reg = DeviceRegistry::new();
        reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        reg.register(String::from("net0"), DeviceType::Network, DeviceClass::Communication, default_caps()).unwrap();
        assert_eq!(reg.find_by_class(DeviceClass::Storage).len(), 1);
        assert_eq!(reg.find_by_class(DeviceClass::Communication).len(), 1);
    }

    #[test]
    fn test_find_ready() {
        let mut reg = DeviceRegistry::new();
        let id1 = reg.register(String::from("blk0"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        let _id2 = reg.register(String::from("blk1"), DeviceType::Block, DeviceClass::Storage, default_caps()).unwrap();
        reg.initialize(id1).unwrap();
        assert_eq!(reg.find_ready().len(), 1);
    }

    #[test]
    fn test_not_found() {
        let reg = DeviceRegistry::new();
        assert!(reg.get(DeviceId(999)).is_err());
    }
}
