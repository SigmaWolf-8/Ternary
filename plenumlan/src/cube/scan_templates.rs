// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # LAN Entity Scan Templates (Technical Manifest §7)
//!
//! 6 templates for classifying LAN entities. Each template pre-fills
//! deterministic dimensions and leaves variable dimensions for probing
//! (MAC OUI, LLDP, SNMP, port scan, protocol fingerprint).

use super::constants::CLASSIFICATION_DIMS;
use super::projection::Trit;

/// A scan template pre-fills some dimensions and leaves others variable.
#[derive(Debug, Clone)]
pub struct ScanTemplate {
    pub name: &'static str,
    pub prefilled: [(usize, Trit); 6],  // max 6 pre-filled dims (1-based index, value)
    pub prefilled_count: usize,
    pub variable_count: usize,           // approximate number of questions to probe
}

/// Workstation template: D1=1, D2=1, D3=3, D5=1, D7=1, D24=3
pub const WORKSTATION: ScanTemplate = ScanTemplate {
    name: "Workstation",
    prefilled: [(1, 1), (2, 1), (3, 3), (5, 1), (7, 1), (24, 3)],
    prefilled_count: 6,
    variable_count: 12,
};

/// Server template: D2=3, D3=3, D5=2, D14=3, D24=1
pub const SERVER: ScanTemplate = ScanTemplate {
    name: "Server",
    prefilled: [(2, 3), (3, 3), (5, 2), (14, 3), (24, 1), (0, 0)],
    prefilled_count: 5,
    variable_count: 14,
};

/// Network Infrastructure template: D1=3, D3=3, D5=3, D14=3, D16=3, D24=1
pub const NETWORK_INFRA: ScanTemplate = ScanTemplate {
    name: "Network Infrastructure",
    prefilled: [(1, 3), (3, 3), (5, 3), (14, 3), (16, 3), (24, 1)],
    prefilled_count: 6,
    variable_count: 8,
};

/// Printer / Device template: D1=2, D3=3, D5=2, D7=1, D8=1, D17=1
pub const PRINTER_DEVICE: ScanTemplate = ScanTemplate {
    name: "Printer / Device",
    prefilled: [(1, 2), (3, 3), (5, 2), (7, 1), (8, 1), (17, 1)],
    prefilled_count: 6,
    variable_count: 9,
};

/// IoT / Sensor template: D1=2, D3=3, D5=2, D7=2, D8=2, D17=1
pub const IOT_SENSOR: ScanTemplate = ScanTemplate {
    name: "IoT / Sensor",
    prefilled: [(1, 2), (3, 3), (5, 2), (7, 2), (8, 2), (17, 1)],
    prefilled_count: 6,
    variable_count: 10,
};

/// Service Endpoint template: D3=3, D27=3 (rest via standard TDNS HTTP scan)
pub const SERVICE_ENDPOINT: ScanTemplate = ScanTemplate {
    name: "Service Endpoint",
    prefilled: [(3, 3), (27, 3), (0, 0), (0, 0), (0, 0), (0, 0)],
    prefilled_count: 2,
    variable_count: 25, // automatic via TDNS scanner
};

pub const ALL_TEMPLATES: [&ScanTemplate; 6] = [
    &WORKSTATION,
    &SERVER,
    &NETWORK_INFRA,
    &PRINTER_DEVICE,
    &IOT_SENSOR,
    &SERVICE_ENDPOINT,
];

/// Apply a template's pre-filled dimensions to a classification array.
/// Variable dimensions are left at 0 (caller must fill them via probing).
pub fn apply_template(template: &ScanTemplate) -> [Trit; CLASSIFICATION_DIMS] {
    let mut classification = [0u8; CLASSIFICATION_DIMS];
    for idx in 0..template.prefilled_count {
        let (dim_1based, value) = template.prefilled[idx];
        if dim_1based >= 1 && dim_1based <= CLASSIFICATION_DIMS {
            classification[dim_1based - 1] = value;
        }
    }
    classification
}

/// Count how many dimensions still need to be filled (value == 0).
pub fn remaining_questions(classification: &[Trit; CLASSIFICATION_DIMS]) -> usize {
    classification.iter().filter(|&&t| t == 0).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workstation_template_applies() {
        let class = apply_template(&WORKSTATION);
        assert_eq!(class[0], 1);  // D1
        assert_eq!(class[1], 1);  // D2
        assert_eq!(class[2], 3);  // D3
        assert_eq!(class[4], 1);  // D5
        assert_eq!(class[6], 1);  // D7
        assert_eq!(class[23], 3); // D24
    }

    #[test]
    fn server_template_applies() {
        let class = apply_template(&SERVER);
        assert_eq!(class[1], 3);  // D2
        assert_eq!(class[2], 3);  // D3
        assert_eq!(class[4], 2);  // D5
        assert_eq!(class[13], 3); // D14
        assert_eq!(class[23], 1); // D24
    }

    #[test]
    fn service_endpoint_minimal_prefill() {
        let class = apply_template(&SERVICE_ENDPOINT);
        assert_eq!(class[2], 3);   // D3
        assert_eq!(class[26], 3);  // D27
        // 25 dimensions should still be zero
        assert_eq!(remaining_questions(&class), 25);
    }

    #[test]
    fn all_templates_have_valid_dims() {
        for template in ALL_TEMPLATES {
            for idx in 0..template.prefilled_count {
                let (dim, val) = template.prefilled[idx];
                assert!(dim >= 1 && dim <= CLASSIFICATION_DIMS,
                    "{}: dim {} out of range", template.name, dim);
                assert!(val >= 1 && val <= 3,
                    "{}: value {} out of Rep C range", template.name, val);
            }
        }
    }
}
