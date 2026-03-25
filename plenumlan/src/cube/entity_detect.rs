// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Entity Class Detection
//!
//! Two registration flows, same output:
//! - **Service endpoints** (HTTP-accessible): standard TDNS scanner runs
//!   against .plm.local URL, auto-derives all 27 classification trits.
//! - **Hardware entities**: LAN scan template pre-fills deterministic
//!   dimensions, probes variable dimensions via MAC OUI, LLDP, SNMP,
//!   port scan, protocol fingerprint.

use super::scan_templates::{
    ScanTemplate, WORKSTATION, SERVER, NETWORK_INFRA,
    PRINTER_DEVICE, IOT_SENSOR, SERVICE_ENDPOINT,
};

/// Detected entity class on the LAN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityClass {
    Workstation,
    Server,
    NetworkInfra,
    PrinterDevice,
    IotSensor,
    ServiceEndpoint,
}

impl EntityClass {
    pub fn template(&self) -> &'static ScanTemplate {
        match self {
            EntityClass::Workstation => &WORKSTATION,
            EntityClass::Server => &SERVER,
            EntityClass::NetworkInfra => &NETWORK_INFRA,
            EntityClass::PrinterDevice => &PRINTER_DEVICE,
            EntityClass::IotSensor => &IOT_SENSOR,
            EntityClass::ServiceEndpoint => &SERVICE_ENDPOINT,
        }
    }

    pub fn label(&self) -> &'static str {
        self.template().name
    }
}

/// Signals gathered from LAN probing that inform entity class detection.
#[derive(Debug, Clone, Default)]
pub struct ProbeSignals {
    pub has_http: bool,              // responds to HTTP on any port
    pub has_plm_local_tld: bool,     // hostname ends in .plm.local
    pub open_ports: Vec<u16>,        // detected open TCP ports
    pub mac_oui_vendor: Option<String>, // MAC OUI vendor lookup
    pub has_lldp: bool,              // responds to LLDP
    pub has_snmp: bool,              // responds to SNMP
    pub os_fingerprint: Option<String>, // OS detection result
}

/// Detect entity class from probe signals.
///
/// Decision tree:
/// 1. HTTP with .plm.local TLD → ServiceEndpoint (use standard TDNS scanner)
/// 2. LLDP or SNMP with network ports (22, 23, 161, 179, 520) → NetworkInfra
/// 3. Printer ports (515, 631, 9100) → PrinterDevice
/// 4. IoT-like OUI or minimal ports → IotSensor
/// 5. Server-like (many open ports, no desktop indicators) → Server
/// 6. Default → Workstation
pub fn detect_entity_class(signals: &ProbeSignals) -> EntityClass {
    // 1. Service endpoint detection
    if signals.has_http && signals.has_plm_local_tld {
        return EntityClass::ServiceEndpoint;
    }

    // 2. Network infrastructure
    let network_ports = [22, 23, 161, 179, 520];
    let has_network_ports = signals.open_ports.iter()
        .any(|p| network_ports.contains(p));
    if (signals.has_lldp || signals.has_snmp) && has_network_ports {
        return EntityClass::NetworkInfra;
    }

    // 3. Printer / device
    let printer_ports = [515, 631, 9100];
    if signals.open_ports.iter().any(|p| printer_ports.contains(p)) {
        return EntityClass::PrinterDevice;
    }

    // 4. IoT / sensor (minimal footprint)
    if signals.open_ports.len() <= 2 && !signals.has_http {
        if let Some(ref vendor) = signals.mac_oui_vendor {
            let v = vendor.to_lowercase();
            if v.contains("espressif") || v.contains("raspberry")
                || v.contains("arduino") || v.contains("texas instruments")
            {
                return EntityClass::IotSensor;
            }
        }
    }

    // 5. Server (many open ports)
    let server_ports = [80, 443, 3306, 5432, 6379, 8080, 8443];
    let server_port_count = signals.open_ports.iter()
        .filter(|p| server_ports.contains(p))
        .count();
    if server_port_count >= 2 || signals.open_ports.len() >= 5 {
        return EntityClass::Server;
    }

    // 6. Default
    EntityClass::Workstation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_plm_local_is_service() {
        let signals = ProbeSignals {
            has_http: true,
            has_plm_local_tld: true,
            ..Default::default()
        };
        assert_eq!(detect_entity_class(&signals), EntityClass::ServiceEndpoint);
    }

    #[test]
    fn lldp_with_ssh_is_network() {
        let signals = ProbeSignals {
            has_lldp: true,
            open_ports: vec![22, 23],
            ..Default::default()
        };
        assert_eq!(detect_entity_class(&signals), EntityClass::NetworkInfra);
    }

    #[test]
    fn printer_port_detected() {
        let signals = ProbeSignals {
            open_ports: vec![631, 9100],
            ..Default::default()
        };
        assert_eq!(detect_entity_class(&signals), EntityClass::PrinterDevice);
    }

    #[test]
    fn espressif_iot() {
        let signals = ProbeSignals {
            open_ports: vec![80],
            mac_oui_vendor: Some("Espressif Inc.".to_string()),
            ..Default::default()
        };
        // Has HTTP but no plm.local, 1 port, IoT vendor
        assert_eq!(detect_entity_class(&signals), EntityClass::IotSensor);
    }

    #[test]
    fn many_ports_is_server() {
        let signals = ProbeSignals {
            has_http: true,
            open_ports: vec![22, 80, 443, 3306, 5432, 8080],
            ..Default::default()
        };
        assert_eq!(detect_entity_class(&signals), EntityClass::Server);
    }

    #[test]
    fn default_is_workstation() {
        let signals = ProbeSignals {
            open_ports: vec![445],
            ..Default::default()
        };
        assert_eq!(detect_entity_class(&signals), EntityClass::Workstation);
    }
}
