# Module Guide: Device Framework

**Module:** `salvi_drivers::device`  
**Status:** Complete (P1.5-001 to P1.5-005)  
**Tests:** ~48 tests

---

## Overview

The Device Framework provides a unified interface for hardware device management in the Salvi kernel. It handles device lifecycle, bus hierarchies, driver registration, and interrupt management with ternary-aware DMA support.

### Key Features

- **Device Lifecycle Management** — Probe, bind, unbind, remove
- **Bus Hierarchy** — Parent-child device relationships
- **Device Registry** — Dynamic device enumeration
- **Interrupt Controller** — Shared IRQ support
- **DMA Engine** — Ternary-aligned memory transfers

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Device Manager                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   Device Registry                    │   │
│  │   ┌─────────┐  ┌─────────┐  ┌─────────┐           │   │
│  │   │ PCI Bus │  │ USB Bus │  │Platform │           │   │
│  │   └────┬────┘  └────┬────┘  └────┬────┘           │   │
│  │        │            │            │                 │   │
│  │   ┌────┴────┐  ┌────┴────┐  ┌────┴────┐           │   │
│  │   │Devices  │  │Devices  │  │Devices  │           │   │
│  │   └─────────┘  └─────────┘  └─────────┘           │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ Driver Registry  │  │ Interrupt Ctrl   │                │
│  └──────────────────┘  └──────────────────┘                │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    DMA Engine                        │   │
│  │   Ternary-aligned transfers, scatter-gather         │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Device Model

### Device Structure

```rust
use salvi_drivers::device::{Device, DeviceId, DeviceState};

pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub bus_type: BusType,
    pub parent: Option<DeviceId>,
    pub children: Vec<DeviceId>,
    pub state: DeviceState,
    pub driver: Option<DriverId>,
    pub resources: ResourceSet,
    pub irqs: Vec<IrqNumber>,
    pub dma_channels: Vec<DmaChannel>,
    pub power_state: PowerState,
}

pub enum DeviceState {
    Uninitialized,
    Probing,
    Bound,
    Running,
    Suspended,
    Removed,
    Error(DeviceError),
}
```

### Device Lifecycle

```
┌─────────────┐
│ Discovered  │
└──────┬──────┘
       │ probe()
       ▼
┌─────────────┐
│  Probing    │
└──────┬──────┘
       │ driver match
       ▼
┌─────────────┐     ┌─────────────┐
│   Bound     │◄───▶│  Suspended  │
└──────┬──────┘     └─────────────┘
       │ start()         suspend()/resume()
       ▼
┌─────────────┐
│  Running    │
└──────┬──────┘
       │ remove()
       ▼
┌─────────────┐
│  Removed    │
└─────────────┘
```

---

## Basic Usage

### Registering a Driver

```rust
use salvi_drivers::{Driver, DriverOps, DeviceId};

struct MyDriver;

impl DriverOps for MyDriver {
    fn name(&self) -> &str { "my_driver" }
    
    fn probe(&self, device: &mut Device) -> Result<(), DriverError> {
        if !self.can_handle(device) {
            return Err(DriverError::NotSupported);
        }
        self.init_hardware(device)?;
        Ok(())
    }
    
    fn remove(&self, device: &mut Device) -> Result<(), DriverError> {
        self.shutdown_hardware(device)?;
        Ok(())
    }
    
    fn suspend(&self, device: &mut Device) -> Result<(), DriverError> {
        self.save_state(device)?;
        self.enter_low_power(device)?;
        Ok(())
    }
    
    fn resume(&self, device: &mut Device) -> Result<(), DriverError> {
        self.exit_low_power(device)?;
        self.restore_state(device)?;
        Ok(())
    }
}

let driver = Driver::new(MyDriver);
driver_registry::register(driver)?;
```

### Device Matching

```rust
use salvi_drivers::device::{MatchTable, MatchEntry};

let match_table = MatchTable::new()
    .add(MatchEntry::pci(0x1234, 0x5678))
    .add(MatchEntry::pci(0x1234, 0x5679))
    .add(MatchEntry::compatible("acme,widget-v2"))
    .add(MatchEntry::name("platform-timer"));

driver.set_match_table(match_table);
```

---

## Bus Hierarchy

### Bus Types

```rust
pub enum BusType {
    Platform,   // System-on-chip devices
    PCI,        // PCI/PCIe devices
    USB,        // USB devices
    I2C,        // I2C bus devices
    SPI,        // SPI bus devices
    Ternary,    // Native ternary bus
}
```

### Creating Bus Hierarchy

```rust
use salvi_drivers::bus::{Bus, BusOps};

let pci_bus = Bus::new("pci0", BusType::PCI);
device_manager::register_bus(pci_bus)?;

let device = Device::new("gpu0", BusType::PCI);
device.set_parent(pci_bus.id());
device_manager::register_device(device)?;

pci_bus.enumerate()?;
```

### Walking Device Tree

```rust
use salvi_drivers::device::DeviceTree;

for device in DeviceTree::iter() {
    println!("{}: {:?}", device.name, device.state);
}

let device = DeviceTree::find("/pci0/gpu0")?;

let usb_devices: Vec<&Device> = DeviceTree::iter()
    .filter(|d| d.bus_type == BusType::USB)
    .collect();
```

---

## Interrupt Controller

### Registering Interrupt Handlers

```rust
use salvi_drivers::interrupt::{IrqHandler, IrqFlags};

let irq_num = device.irqs[0];

irq::request(
    irq_num,
    IrqHandler::new(|irq, data| {
        let device = data.downcast_ref::<MyDevice>().unwrap();
        let status = device.read_status();
        if status.has_data() {
            device.process_data();
        }
        device.ack_interrupt();
        IrqReturn::Handled
    }),
    IrqFlags::SHARED,
    "my_device",
    device_data,
)?;

irq::free(irq_num, device_data)?;
```

### Shared IRQs

```rust
irq::request(irq_num, handler1, IrqFlags::SHARED, "device1", data1)?;
irq::request(irq_num, handler2, IrqFlags::SHARED, "device2", data2)?;
```

---

## DMA Engine

### DMA Transfers

```rust
use salvi_drivers::dma::{DmaEngine, DmaDirection, DmaBuffer};

let dma = DmaEngine::instance();
let buffer = DmaBuffer::allocate(4096, DmaAlignment::Ternary)?;

buffer.write(&data);

dma.transfer(
    &buffer,
    device_address,
    DmaDirection::ToDevice,
)?;

let result_buffer = DmaBuffer::allocate(4096, DmaAlignment::Ternary)?;
dma.transfer(
    &result_buffer,
    device_address,
    DmaDirection::FromDevice,
)?;
let result = result_buffer.read();
```

### Scatter-Gather DMA

```rust
use salvi_drivers::dma::ScatterGatherList;

let sgl = ScatterGatherList::new()
    .add_entry(buffer1.physical_addr(), buffer1.len())
    .add_entry(buffer2.physical_addr(), buffer2.len())
    .add_entry(buffer3.physical_addr(), buffer3.len());

dma.scatter_gather_transfer(&sgl, device_address, DmaDirection::ToDevice)?;
```

---

## Best Practices

### 1. Always Clean Up in remove()
Release all resources, free IRQs, and unmap DMA buffers.

### 2. Handle Shared IRQs Correctly
Always check if your device generated the interrupt before handling.

### 3. Use Ternary-Aligned DMA
For best performance, align DMA buffers to ternary boundaries.

### 4. Implement Power Management
Support suspend/resume for battery-powered devices.

---

*Così sia.*
