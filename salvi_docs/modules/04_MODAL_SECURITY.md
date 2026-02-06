# Module Guide: Modal Security System

**Module:** `salvi_kernel::security`  
**Status:** Complete (P1-016 to P1-021)  
**Tests:** ~70 tests

---

## Overview

The Modal Security System provides hardware-enforced protection through a novel four-mode security model. Unlike traditional ring-based protection (ring 0-3), the Salvi security model uses mathematically-inspired mode names and ternary-native access control.

### Key Features

- **Four Security Modes** — Mode 0, Mode 1, Mode phi, Mode phi+
- **Domain Isolation** — Process isolation with controlled sharing
- **Capability System** — Fine-grained permissions with delegation
- **Audit Trail** — Comprehensive security event logging
- **Policy Engine** — Configurable security policies

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Manager                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │                  Policy Engine                       │   │
│  │   ┌─────────┐  ┌─────────┐  ┌─────────┐           │   │
│  │   │  Rules  │  │Decisions│  │ Cache   │           │   │
│  │   └─────────┘  └─────────┘  └─────────┘           │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │  Mode Enforcer   │  │  Capability Mgr  │                │
│  │ ┌──────────────┐ │  │ ┌──────────────┐ │                │
│  │ │ Mode 0 (Hyp) │ │  │ │ Grant/Revoke │ │                │
│  │ │ Mode 1 (Ker) │ │  │ │ Delegation   │ │                │
│  │ │ Mode phi(Sup)│ │  │ │ Expiration   │ │                │
│  │ │ Mode phi+(Us)│ │  │ └──────────────┘ │                │
│  │ └──────────────┘ │  └──────────────────┘                │
│  └──────────────────┘                                      │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │  Domain Manager  │  │   Audit System   │                │
│  │ ┌──────────────┐ │  │ ┌──────────────┐ │                │
│  │ │ Isolation    │ │  │ │ Event Log    │ │                │
│  │ │ Boundaries   │ │  │ │ Alerting     │ │                │
│  │ │ Sharing      │ │  │ │ Compliance   │ │                │
│  │ └──────────────┘ │  │ └──────────────┘ │                │
│  └──────────────────┘  └──────────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

---

## Security Modes

### Mode Hierarchy

| Mode | Symbol | Privilege Level | Typical Use |
|------|--------|-----------------|-------------|
| 0 | - | Highest | Hypervisor, VMM |
| 1 | - | High | Kernel, drivers |
| phi | phi | Elevated | System services |
| phi+ | phi+ | Lowest | User applications |

```
┌─────────────────────────────────────────────────────────────┐
│  Mode 0 (Hypervisor)                                        │
│  - Full system access, hardware configuration               │
├─────────────────────────────────────────────────────────────┤
│  Mode 1 (Kernel)                                            │
│  - OS kernel operations, device drivers, system calls       │
├─────────────────────────────────────────────────────────────┤
│  Mode phi (Supervisor)                                      │
│  - Elevated user processes, system services                 │
├─────────────────────────────────────────────────────────────┤
│  Mode phi+ (User)                                           │
│  - Regular user applications, sandboxed execution           │
└─────────────────────────────────────────────────────────────┘
```

### Checking Current Mode

```rust
use salvi_kernel::security::{Mode, current_mode};

let mode = current_mode();

match mode {
    Mode::Hypervisor => println!("Running in hypervisor mode"),
    Mode::Kernel => println!("Running in kernel mode"),
    Mode::Supervisor => println!("Running in supervisor mode"),
    Mode::User => println!("Running in user mode"),
}

if mode.is_privileged() {
    // Kernel or Hypervisor
}
```

### Mode Transitions

```rust
use salvi_kernel::security::{escalate, deescalate, ModeTransition};

match escalate(Mode::Kernel) {
    Ok(_) => println!("Now in kernel mode"),
    Err(SecurityError::InsufficientPrivilege) => {
        println!("Escalation denied");
    }
}

deescalate(Mode::User);

{
    let _guard = ModeTransition::elevate(Mode::Supervisor)?;
    // Operations at supervisor level
}  // Automatically returns to previous mode
```

---

## Domains

Domains provide isolation boundaries between processes:

### Creating Domains

```rust
use salvi_kernel::security::{Domain, DomainBuilder, DomainFlags};

let domain = DomainBuilder::new("secure_domain")
    .flags(DomainFlags::ISOLATED | DomainFlags::NO_NETWORK)
    .memory_limit(256 * 1024 * 1024)
    .cpu_quota(50)
    .build()?;

domain.add_process(pid)?;
domain.remove_process(pid)?;
```

### Domain Isolation

```rust
let domain = DomainBuilder::new("sandbox")
    .isolation_level(IsolationLevel::Strict)
    .build()?;

pub enum IsolationLevel {
    Minimal,   // Shared namespace, minimal isolation
    Moderate,  // Separate memory, shared filesystem
    Strict,    // Separate memory and filesystem
    Maximum,   // Complete isolation (like VM)
}
```

### Cross-Domain Communication

```rust
use salvi_kernel::security::{DomainGate, GatePermission};

let gate = DomainGate::create(
    source_domain,
    target_domain,
    GatePermission::MESSAGE_PASS | GatePermission::SHARED_MEMORY,
)?;

gate.send_message(&message)?;
gate.close()?;
```

---

## Capability System

Fine-grained permissions using unforgeable capability tokens:

### Capability Types

```rust
pub enum CapabilityType {
    MemoryRead(AddressRange),
    MemoryWrite(AddressRange),
    MemoryExecute(AddressRange),
    FileRead(PathPattern),
    FileWrite(PathPattern),
    FileCreate(PathPattern),
    FileDelete(PathPattern),
    ProcessCreate,
    ProcessSignal(ProcessIdPattern),
    ProcessDebug(ProcessIdPattern),
    NetworkListen(PortRange),
    NetworkConnect(AddressPattern),
    DeviceAccess(DeviceId),
    ModeEscalate(Mode),
    SystemShutdown,
    TimeCritical,
}
```

### Granting Capabilities

```rust
use salvi_kernel::security::{CapabilityManager, CapabilityFlags};

let cap_mgr = CapabilityManager::instance();

let cap = cap_mgr.grant(
    target_pid,
    CapabilityType::FileRead(PathPattern::new("/data/*")),
    CapabilityFlags::DELEGATABLE,
    Duration::from_hours(24),
)?;
```

### Checking Capabilities

```rust
if cap_mgr.check(pid, &CapabilityType::FileRead(path.into()))? {
    // Access allowed
} else {
    return Err(SecurityError::PermissionDenied);
}
```

### Delegation

```rust
let delegated = cap_mgr.delegate(
    original_cap,
    new_target_pid,
    CapabilityFlags::NON_DELEGATABLE,
)?;
```

---

## Audit Trail

### Event Logging

```rust
use salvi_kernel::security::audit::{AuditLog, AuditEvent, Severity};

let audit = AuditLog::instance();

audit.log(AuditEvent {
    timestamp: FemtosecondTimestamp::now(),
    source_pid: current_pid(),
    source_mode: current_mode(),
    event_type: AuditEventType::ModeEscalation,
    target: Some(AuditTarget::Mode(Mode::Kernel)),
    result: AuditResult::Success,
    severity: Severity::Warning,
    details: "Kernel mode escalation for device access".into(),
});
```

### Querying Audit Logs

```rust
let events = audit.query()
    .since(one_hour_ago)
    .severity(Severity::Warning)
    .event_type(AuditEventType::ModeEscalation)
    .execute()?;

for event in events {
    println!("[{}] PID {} {:?}: {:?}",
        event.timestamp, event.source_pid, event.event_type, event.result);
}
```

---

## Policy Engine

### Defining Policies

```rust
use salvi_kernel::security::policy::{Policy, PolicyRule, PolicyAction};

let policy = Policy::builder()
    .name("network_isolation")
    .priority(100)
    .add_rule(PolicyRule {
        subject: Subject::Domain("sandbox"),
        action: PolicyAction::Deny,
        resource: Resource::Network(NetworkResource::All),
        conditions: vec![Condition::Always],
    })
    .add_rule(PolicyRule {
        subject: Subject::Mode(Mode::User),
        action: PolicyAction::Deny,
        resource: Resource::Device(DeviceClass::Storage),
        conditions: vec![Condition::Unless(CapabilityType::DeviceAccess(any()))],
    })
    .build()?;

policy_engine::install(policy)?;
```

### Policy Evaluation

```rust
let decision = policy_engine::evaluate(
    &Subject::Process(pid),
    &Resource::File("/etc/secrets".into()),
    &Operation::Read,
)?;

match decision {
    PolicyDecision::Allow => { /* proceed */ },
    PolicyDecision::Deny(reason) => {
        audit.log_denial(pid, reason);
        return Err(SecurityError::PolicyDenied(reason));
    },
    PolicyDecision::Audit => {
        audit.log_access(pid, resource);
        // proceed but log
    },
}
```

---

## Best Practices

### 1. Principle of Least Privilege
Always run processes in the lowest mode that satisfies their requirements.

### 2. Use Capabilities Over Mode Escalation
Instead of escalating to kernel mode, grant specific capabilities.

### 3. Enable Auditing for Sensitive Operations
All mode transitions and capability grants should be audited.

### 4. Time-Limit Capabilities
Always set expiration on granted capabilities to prevent privilege accumulation.

---

*Così sia.*
