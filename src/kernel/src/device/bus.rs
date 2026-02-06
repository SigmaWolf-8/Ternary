use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{DeviceId, DeviceError, DeviceResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BusId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    System,
    Ternary,
    Memory,
    Peripheral,
    Virtual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusSpeed {
    Low,
    Full,
    High,
    Super,
}

impl BusSpeed {
    pub fn bandwidth_trits_per_sec(&self) -> u64 {
        match self {
            BusSpeed::Low => 1_000_000,
            BusSpeed::Full => 12_000_000,
            BusSpeed::High => 480_000_000,
            BusSpeed::Super => 5_000_000_000,
        }
    }
}

pub struct BusDescriptor {
    pub id: BusId,
    pub name: String,
    pub bus_type: BusType,
    pub speed: BusSpeed,
    pub parent: Option<BusId>,
    pub devices: Vec<DeviceId>,
    pub max_devices: usize,
}

impl BusDescriptor {
    pub fn new(id: BusId, name: String, bus_type: BusType, speed: BusSpeed, max_devices: usize) -> Self {
        Self {
            id,
            name,
            bus_type,
            speed,
            parent: None,
            devices: Vec::new(),
            max_devices,
        }
    }

    pub fn attach_device(&mut self, device_id: DeviceId) -> DeviceResult<()> {
        if self.devices.len() >= self.max_devices {
            return Err(DeviceError::CapacityExceeded);
        }
        if self.devices.contains(&device_id) {
            return Err(DeviceError::AlreadyExists);
        }
        self.devices.push(device_id);
        Ok(())
    }

    pub fn detach_device(&mut self, device_id: DeviceId) -> DeviceResult<()> {
        if let Some(pos) = self.devices.iter().position(|&d| d == device_id) {
            self.devices.remove(pos);
            Ok(())
        } else {
            Err(DeviceError::NotFound)
        }
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn has_device(&self, device_id: DeviceId) -> bool {
        self.devices.contains(&device_id)
    }
}

pub struct BusManager {
    buses: BTreeMap<BusId, BusDescriptor>,
    next_id: u16,
}

impl BusManager {
    pub fn new() -> Self {
        Self {
            buses: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub fn create_bus(&mut self, name: String, bus_type: BusType, speed: BusSpeed, max_devices: usize) -> DeviceResult<BusId> {
        let id = BusId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(DeviceError::CapacityExceeded)?;
        let bus = BusDescriptor::new(id, name, bus_type, speed, max_devices);
        self.buses.insert(id, bus);
        Ok(id)
    }

    pub fn remove_bus(&mut self, id: BusId) -> DeviceResult<()> {
        if let Some(bus) = self.buses.get(&id) {
            if !bus.devices.is_empty() {
                return Err(DeviceError::InvalidState);
            }
        } else {
            return Err(DeviceError::NotFound);
        }
        self.buses.remove(&id);
        Ok(())
    }

    pub fn get_bus(&self, id: BusId) -> DeviceResult<&BusDescriptor> {
        self.buses.get(&id).ok_or(DeviceError::NotFound)
    }

    pub fn get_bus_mut(&mut self, id: BusId) -> DeviceResult<&mut BusDescriptor> {
        self.buses.get_mut(&id).ok_or(DeviceError::NotFound)
    }

    pub fn attach_device(&mut self, bus_id: BusId, device_id: DeviceId) -> DeviceResult<()> {
        let bus = self.buses.get_mut(&bus_id).ok_or(DeviceError::NotFound)?;
        bus.attach_device(device_id)
    }

    pub fn detach_device(&mut self, bus_id: BusId, device_id: DeviceId) -> DeviceResult<()> {
        let bus = self.buses.get_mut(&bus_id).ok_or(DeviceError::NotFound)?;
        bus.detach_device(device_id)
    }

    pub fn find_device_bus(&self, device_id: DeviceId) -> Option<BusId> {
        for (id, bus) in &self.buses {
            if bus.has_device(device_id) {
                return Some(*id);
            }
        }
        None
    }

    pub fn set_parent(&mut self, child: BusId, parent: BusId) -> DeviceResult<()> {
        if !self.buses.contains_key(&parent) {
            return Err(DeviceError::NotFound);
        }
        let bus = self.buses.get_mut(&child).ok_or(DeviceError::NotFound)?;
        bus.parent = Some(parent);
        Ok(())
    }

    pub fn bus_count(&self) -> usize {
        self.buses.len()
    }

    pub fn buses_by_type(&self, bus_type: BusType) -> Vec<BusId> {
        self.buses
            .iter()
            .filter(|(_, b)| b.bus_type == bus_type)
            .map(|(id, _)| *id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_creation() {
        let mut mgr = BusManager::new();
        let id = mgr.create_bus(String::from("sys"), BusType::System, BusSpeed::High, 8).unwrap();
        assert_eq!(mgr.bus_count(), 1);
        let bus = mgr.get_bus(id).unwrap();
        assert_eq!(bus.bus_type, BusType::System);
        assert_eq!(bus.speed, BusSpeed::High);
    }

    #[test]
    fn test_bus_attach_detach() {
        let mut mgr = BusManager::new();
        let bus_id = mgr.create_bus(String::from("sys"), BusType::System, BusSpeed::High, 4).unwrap();
        let dev = DeviceId(1);

        mgr.attach_device(bus_id, dev).unwrap();
        assert_eq!(mgr.get_bus(bus_id).unwrap().device_count(), 1);
        assert_eq!(mgr.find_device_bus(dev), Some(bus_id));

        mgr.detach_device(bus_id, dev).unwrap();
        assert_eq!(mgr.get_bus(bus_id).unwrap().device_count(), 0);
        assert_eq!(mgr.find_device_bus(dev), None);
    }

    #[test]
    fn test_bus_capacity() {
        let mut mgr = BusManager::new();
        let bus_id = mgr.create_bus(String::from("tiny"), BusType::Peripheral, BusSpeed::Low, 2).unwrap();
        mgr.attach_device(bus_id, DeviceId(1)).unwrap();
        mgr.attach_device(bus_id, DeviceId(2)).unwrap();
        assert_eq!(mgr.attach_device(bus_id, DeviceId(3)), Err(DeviceError::CapacityExceeded));
    }

    #[test]
    fn test_bus_duplicate_device() {
        let mut mgr = BusManager::new();
        let bus_id = mgr.create_bus(String::from("sys"), BusType::System, BusSpeed::High, 8).unwrap();
        mgr.attach_device(bus_id, DeviceId(1)).unwrap();
        assert_eq!(mgr.attach_device(bus_id, DeviceId(1)), Err(DeviceError::AlreadyExists));
    }

    #[test]
    fn test_bus_remove_nonempty() {
        let mut mgr = BusManager::new();
        let bus_id = mgr.create_bus(String::from("sys"), BusType::System, BusSpeed::High, 8).unwrap();
        mgr.attach_device(bus_id, DeviceId(1)).unwrap();
        assert_eq!(mgr.remove_bus(bus_id), Err(DeviceError::InvalidState));
    }

    #[test]
    fn test_bus_remove_empty() {
        let mut mgr = BusManager::new();
        let bus_id = mgr.create_bus(String::from("sys"), BusType::System, BusSpeed::High, 8).unwrap();
        mgr.remove_bus(bus_id).unwrap();
        assert_eq!(mgr.bus_count(), 0);
    }

    #[test]
    fn test_bus_parent() {
        let mut mgr = BusManager::new();
        let parent = mgr.create_bus(String::from("root"), BusType::System, BusSpeed::Super, 16).unwrap();
        let child = mgr.create_bus(String::from("peri"), BusType::Peripheral, BusSpeed::Full, 8).unwrap();
        mgr.set_parent(child, parent).unwrap();
        assert_eq!(mgr.get_bus(child).unwrap().parent, Some(parent));
    }

    #[test]
    fn test_buses_by_type() {
        let mut mgr = BusManager::new();
        mgr.create_bus(String::from("sys1"), BusType::System, BusSpeed::High, 8).unwrap();
        mgr.create_bus(String::from("peri"), BusType::Peripheral, BusSpeed::Low, 4).unwrap();
        mgr.create_bus(String::from("sys2"), BusType::System, BusSpeed::Super, 16).unwrap();
        let sys_buses = mgr.buses_by_type(BusType::System);
        assert_eq!(sys_buses.len(), 2);
    }

    #[test]
    fn test_bus_speed_bandwidth() {
        assert!(BusSpeed::Low.bandwidth_trits_per_sec() < BusSpeed::Full.bandwidth_trits_per_sec());
        assert!(BusSpeed::Full.bandwidth_trits_per_sec() < BusSpeed::High.bandwidth_trits_per_sec());
        assert!(BusSpeed::High.bandwidth_trits_per_sec() < BusSpeed::Super.bandwidth_trits_per_sec());
    }
}
