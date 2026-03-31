// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # 27→3 Slot Projection
//!
//! Source-agnostic projection per spec §9.3: takes 27 classification trits
//! from ANY source (internet TDNS, LAN entity scan, manual entry, future
//! methods) and returns 3 cube coordinate trits in Rep C {1, 2, 3}.
//!
//! 27 classification dimensions partition into 3 groups of 9
//! (CLASSIFICATION_DIMS / CUBE_DIMS = DIMS_PER_GROUP). Each group
//! determines one cube dimension via `project_to_gf3(k, DIMS_PER_GROUP)`
//! with polarity-adjusted dimensions.

use ternary_math::gf3_algebra::project_to_gf3;

use super::constants::{CLASSIFICATION_DIMS, CUBE_DIMS, DIMS_PER_GROUP};

/// A Rep C trit value: 1, 2, or 3.
pub type Trit = u8;

/// 3-trit slot address in Rep C {1, 2, 3}. Index 0 = plane, 1 = role, 2 = instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotAddress {
    pub plane: Trit,    // Rep C: 1=Data, 2=Control, 3=Management
    pub role: Trit,     // Rep C: 1=Primary, 2=Secondary, 3=Tertiary
    pub instance: Trit, // Rep C: 1, 2, or 3
}

impl SlotAddress {
    pub fn new(plane: Trit, role: Trit, instance: Trit) -> Self {
        assert!(plane >= 1 && plane <= 3, "plane must be Rep C {{1,2,3}}, got {}", plane);
        assert!(role >= 1 && role <= 3, "role must be Rep C {{1,2,3}}, got {}", role);
        assert!(instance >= 1 && instance <= 3, "instance must be Rep C {{1,2,3}}, got {}", instance);
        SlotAddress { plane, role, instance }
    }

    pub fn to_array(&self) -> [Trit; CUBE_DIMS] {
        [self.plane, self.role, self.instance]
    }

    /// GF(3) offset within a node's 27-slot range.
    pub fn to_offset(&self) -> usize {
        let p = (self.plane - 1) as usize;
        let r = (self.role - 1) as usize;
        let i = (self.instance - 1) as usize;
        p * 9 + r * 3 + i
    }
}

/// Polarity: + means use value as-is, − means invert (1↔3, 2→2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Positive,
    Negative,
}

/// Polarity-invert a Rep C trit: 1→3, 3→1, 2→2.
fn invert_trit(t: Trit) -> Trit {
    match t {
        1 => 3,
        3 => 1,
        2 => 2,
        _ => panic!("invalid Rep C trit: {}", t),
    }
}

/// Dimension assignment for each of the 3 projection groups.
/// Each entry is (1-based dimension number, polarity).
///
/// Plane group (security domain): D1, D2(−), D3, D9(−), D10, D17, D19, D25, D26
/// Role group (functional class): D5, D6, D7, D8, D12, D18, D22(−), D23, D24(−)
/// Instance group (temporal/scale): D4, D11, D13, D14, D15, D16, D20, D21, D27
const PLANE_DIMS: [(usize, Polarity); DIMS_PER_GROUP] = [
    ( 1, Polarity::Positive),  // D1  EntityKind
    ( 2, Polarity::Negative),  // D2  OperatorScale (inverted)
    ( 3, Polarity::Positive),  // D3  OperatorTransparency
    ( 9, Polarity::Negative),  // D9  Encryption (inverted)
    (10, Polarity::Positive),  // D10 AuthNMethod
    (17, Polarity::Positive),  // D17 Jurisdiction
    (19, Polarity::Positive),  // D19 PolicyPresence — compliance posture is a security domain property
    (25, Polarity::Positive),  // D25 AuditPosture
    (26, Polarity::Positive),  // D26 TrackerCount
];

const ROLE_DIMS: [(usize, Polarity); DIMS_PER_GROUP] = [
    ( 5, Polarity::Positive),  // D5  Interactivity
    ( 6, Polarity::Positive),  // D6  MediaRichness
    ( 7, Polarity::Positive),  // D7  DataPersistence
    ( 8, Polarity::Positive),  // D8  Intelligence — what kind of processing
    (12, Polarity::Positive),  // D12 APIPresence
    (18, Polarity::Positive),  // D18 DataAppetite — heavier data → secondary/tertiary role
    (22, Polarity::Negative),  // D22 Monetization (inverted)
    (23, Polarity::Positive),  // D23 UpdateCadence
    (24, Polarity::Negative),  // D24 Availability (inverted)
];

const INSTANCE_DIMS: [(usize, Polarity); DIMS_PER_GROUP] = [
    ( 4, Polarity::Positive),  // D4  LifespanIntent
    (11, Polarity::Positive),  // D11 ProtocolComplexity
    (13, Polarity::Positive),  // D13 ContentVolatility
    (14, Polarity::Positive),  // D14 UserBase
    (15, Polarity::Positive),  // D15 ProtocolLayering
    (16, Polarity::Positive),  // D16 Freshness
    (20, Polarity::Positive),  // D20 CostModel — subscription implies higher availability
    (21, Polarity::Positive),  // D21 GeographicReach
    (27, Polarity::Positive),  // D27 Confidence — balances the 9/9/9 partition
];

/// Count how many of the polarity-adjusted dimensions in a group have value 3 (high).
/// This count `k` is fed to `project_to_gf3(k, DIMS_PER_GROUP)`.
fn count_high(
    classification: &[Trit; CLASSIFICATION_DIMS],
    group: &[(usize, Polarity); DIMS_PER_GROUP],
) -> u64 {
    let mut k = 0u64;
    for &(dim_1based, polarity) in group {
        let raw = classification[dim_1based - 1]; // 0-based array index
        let adjusted = match polarity {
            Polarity::Positive => raw,
            Polarity::Negative => invert_trit(raw),
        };
        if adjusted == 3 {
            k += 1;
        }
    }
    k
}

/// Project 27 classification trits (Rep C) to a 3-trit slot address (Rep C).
///
/// Source-agnostic: works with internet TDNS, LAN entity scan, manual entry,
/// or any future classification method.
///
/// Returns `None` if any input trit is outside {1, 2, 3} (zero-sentinel violation).
pub fn project_to_slot(classification: &[Trit; CLASSIFICATION_DIMS]) -> Option<SlotAddress> {
    for &t in classification.iter() {
        if t < 1 || t > 3 {
            return None; // zero-sentinel: reject any non-Rep-C value
        }
    }

    let plane_k = count_high(classification, &PLANE_DIMS);
    let role_k  = count_high(classification, &ROLE_DIMS);
    let inst_k  = count_high(classification, &INSTANCE_DIMS);

    let n = DIMS_PER_GROUP as u64;

    // project_to_gf3 returns GF(3) {0,1,2}, lift to Rep C by +1
    let plane    = project_to_gf3(plane_k, n) + 1;
    let role     = project_to_gf3(role_k, n) + 1;
    let instance = project_to_gf3(inst_k, n) + 1;

    Some(SlotAddress::new(plane, role, instance))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ones_projects_to_data_primary_1() {
        let class = [1u8; CLASSIFICATION_DIMS];
        let slot = project_to_slot(&class).unwrap();
        // All trits are 1; after polarity inversion on negative dims,
        // inverted 1 = 3 (high). Positive dims stay 1 (not high).
        // Plane: 4 negative dims (D2−, D9−) → those become 3 = high.
        //   D2(−): 1→3=high, D9(−): 1→3=high = 2 high out of 9
        //   project_to_gf3(2, 9) = min(3*2/9, 2) = min(0, 2) = 0 → Rep C 1
        assert_eq!(slot.plane, 1); // Data
    }

    #[test]
    fn all_threes_projects_correctly() {
        let class = [3u8; CLASSIFICATION_DIMS];
        let slot = project_to_slot(&class).unwrap();
        // All trits are 3; after polarity inversion on negative dims,
        // inverted 3 = 1 (not high). Positive dims stay 3 (high).
        // Plane: 7 positive dims all high, 2 negative dims → 1 (not high)
        //   k = 7, project_to_gf3(7, 9) = min(3*7/9, 2) = min(2, 2) = 2 → Rep C 3
        assert_eq!(slot.plane, 3); // Management
    }

    #[test]
    fn center_classification_projects_to_center() {
        let class = [2u8; CLASSIFICATION_DIMS];
        let slot = project_to_slot(&class).unwrap();
        // All trits are 2 (center). Polarity inversion: 2→2 (center stays center).
        // No dim has value 3, so k=0 for all groups.
        // project_to_gf3(0, 9) = 0 → Rep C 1
        assert_eq!(slot.plane, 1);
        assert_eq!(slot.role, 1);
        assert_eq!(slot.instance, 1);
    }

    #[test]
    fn rejects_zero_trit() {
        let mut class = [1u8; CLASSIFICATION_DIMS];
        class[0] = 0; // zero-sentinel violation
        assert!(project_to_slot(&class).is_none());
    }

    #[test]
    fn rejects_four_trit() {
        let mut class = [1u8; CLASSIFICATION_DIMS];
        class[5] = 4; // out of Rep C range
        assert!(project_to_slot(&class).is_none());
    }

    #[test]
    fn slot_offset_center() {
        let center = SlotAddress::new(2, 2, 2);
        assert_eq!(center.to_offset(), 13); // GATEWAY_OFFSET
    }

    #[test]
    fn slot_offset_range() {
        for p in 1..=3u8 {
            for r in 1..=3u8 {
                for i in 1..=3u8 {
                    let slot = SlotAddress::new(p, r, i);
                    let off = slot.to_offset();
                    assert!(off < 27, "offset {} out of range for ({},{},{})", off, p, r, i);
                }
            }
        }
    }

    #[test]
    fn polarity_tables_cover_all_27_dims() {
        let mut seen = [false; 27];
        for &(d, _) in PLANE_DIMS.iter().chain(ROLE_DIMS.iter()).chain(INSTANCE_DIMS.iter()) {
            assert!(d >= 1 && d <= 27, "dimension {} out of range", d);
            assert!(!seen[d - 1], "dimension {} appears twice", d);
            seen[d - 1] = true;
        }
        for (i, &s) in seen.iter().enumerate() {
            assert!(s, "dimension {} missing from polarity tables", i + 1);
        }
    }

    #[test]
    fn each_group_has_9_dims() {
        assert_eq!(PLANE_DIMS.len(), DIMS_PER_GROUP);
        assert_eq!(ROLE_DIMS.len(), DIMS_PER_GROUP);
        assert_eq!(INSTANCE_DIMS.len(), DIMS_PER_GROUP);
    }
}
