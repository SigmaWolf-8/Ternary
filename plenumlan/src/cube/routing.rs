// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # Inter-Service Routing by Hamming Distance
//!
//! HD determines the auth level required for a call between two slots:
//!
//! | HD | Count | Auth Level |
//! |----|-------|------------|
//! | 0  | 1     | Loopback — no auth |
//! | 1  | 6     | Same domain — direct call |
//! | 2  | 12    | Cross-boundary — capability token |
//! | 3  | 8     | Corners — full mutual TL-DSA auth |
//!
//! The gateway at (2,2,2) Rep C / (1,1,1) GF(3) has HD ≤ 2 from 19/27
//! slots (self + 6 face + 12 edge). The 8 corners are HD 3 (full TL-DSA).
//!
//! Greedy routing is loop-free and optimal: Hamming distance is a metric
//! (non-negative, symmetric, triangle inequality). Greedy routing decreases
//! HD by exactly 1 at each hop, terminating in exactly HD(src, dst) hops.

use super::constants::{CUBE_DIMS, GF3_ORDER, SLOTS_PER_NODE};
use super::projection::{SlotAddress, Trit};

/// Auth level required for inter-service calls, determined by Hamming distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthLevel {
    Loopback,       // HD 0 — same slot
    Direct,         // HD 1 — same security domain
    CapabilityToken, // HD 2 — crosses boundaries
    FullMutualAuth, // HD 3 — maximum separation, full TL-DSA
}

/// Compute the 3-trit Hamming distance between two slot addresses.
/// Returns the number of positions where the trits differ (0, 1, 2, or 3).
pub fn slot_hamming_distance(a: &SlotAddress, b: &SlotAddress) -> usize {
    let aa = a.to_array();
    let bb = b.to_array();
    let mut hd = 0;
    for i in 0..CUBE_DIMS {
        if aa[i] != bb[i] {
            hd += 1;
        }
    }
    hd
}

/// Determine the auth level required for a call from `src` to `dst`.
pub fn required_auth_level(src: &SlotAddress, dst: &SlotAddress) -> AuthLevel {
    match slot_hamming_distance(src, dst) {
        0 => AuthLevel::Loopback,
        1 => AuthLevel::Direct,
        2 => AuthLevel::CapabilityToken,
        3 => AuthLevel::FullMutualAuth,
        _ => unreachable!("3-trit HD cannot exceed 3"),
    }
}

/// Compute the next hop in greedy routing from `current` toward `target`.
/// Returns the neighbor that decreases HD by exactly 1, choosing the
/// first differing dimension (plane > role > instance priority).
///
/// Returns `None` if current == target (already at destination).
pub fn greedy_next_hop(current: &SlotAddress, target: &SlotAddress) -> Option<SlotAddress> {
    let ca = current.to_array();
    let ta = target.to_array();

    for i in 0..CUBE_DIMS {
        if ca[i] != ta[i] {
            let mut next = ca;
            next[i] = ta[i]; // move to target's value in this dimension
            return Some(SlotAddress::new(next[0], next[1], next[2]));
        }
    }
    None // already at destination
}

/// Verify the full routing path from `src` to `dst` using greedy routing.
/// Returns the path as a list of slot addresses (including src and dst).
pub fn greedy_route(src: &SlotAddress, dst: &SlotAddress) -> Vec<SlotAddress> {
    let mut path = vec![*src];
    let mut current = *src;
    while current != *dst {
        let next = greedy_next_hop(&current, dst)
            .expect("greedy routing should always converge");
        path.push(next);
        current = next;
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hd0() {
        let s = SlotAddress::new(2, 2, 2);
        assert_eq!(slot_hamming_distance(&s, &s), 0);
        assert_eq!(required_auth_level(&s, &s), AuthLevel::Loopback);
    }

    #[test]
    fn adjacent_hd1() {
        let a = SlotAddress::new(2, 2, 2); // center
        let b = SlotAddress::new(2, 2, 3); // one trit differs
        assert_eq!(slot_hamming_distance(&a, &b), 1);
        assert_eq!(required_auth_level(&a, &b), AuthLevel::Direct);
    }

    #[test]
    fn cross_boundary_hd2() {
        let a = SlotAddress::new(1, 1, 2);
        let b = SlotAddress::new(2, 2, 2);
        assert_eq!(slot_hamming_distance(&a, &b), 2);
        assert_eq!(required_auth_level(&a, &b), AuthLevel::CapabilityToken);
    }

    #[test]
    fn corner_to_corner_hd3() {
        let a = SlotAddress::new(1, 1, 1);
        let b = SlotAddress::new(3, 3, 3);
        assert_eq!(slot_hamming_distance(&a, &b), 3);
        assert_eq!(required_auth_level(&a, &b), AuthLevel::FullMutualAuth);
    }

    #[test]
    fn gateway_hd_distribution() {
        let gw = SlotAddress::new(2, 2, 2);
        let mut counts = [0usize; 4];
        for p in 1..=3u8 {
            for r in 1..=3u8 {
                for i in 1..=3u8 {
                    let slot = SlotAddress::new(p, r, i);
                    let hd = slot_hamming_distance(&gw, &slot);
                    assert!(hd <= 3, "HD cannot exceed 3 in 3D cube");
                    counts[hd] += 1;
                }
            }
        }
        assert_eq!(counts, [1, 6, 12, 8]);
    }

    #[test]
    fn hd_distribution() {
        // In 3³ cube: HD 0 = 1, HD 1 = 6, HD 2 = 12, HD 3 = 8
        let mut counts = [0usize; 4];
        let origin = SlotAddress::new(1, 1, 1);
        for p in 1..=3u8 {
            for r in 1..=3u8 {
                for i in 1..=3u8 {
                    let slot = SlotAddress::new(p, r, i);
                    let hd = slot_hamming_distance(&origin, &slot);
                    counts[hd] += 1;
                }
            }
        }
        assert_eq!(counts, [1, 6, 12, 8]);
    }

    #[test]
    fn greedy_routing_loop_free_all_702_pairs() {
        let mut failures = 0;
        for sp in 1..=3u8 {
            for sr in 1..=3u8 {
                for si in 1..=3u8 {
                    let src = SlotAddress::new(sp, sr, si);
                    for dp in 1..=3u8 {
                        for dr in 1..=3u8 {
                            for di in 1..=3u8 {
                                let dst = SlotAddress::new(dp, dr, di);
                                if src == dst { continue; }
                                let path = greedy_route(&src, &dst);
                                let expected_hops = slot_hamming_distance(&src, &dst);
                                if path.len() != expected_hops + 1 {
                                    failures += 1;
                                }
                                // Verify HD strictly decreases
                                for w in path.windows(2) {
                                    let hd_before = slot_hamming_distance(&w[0], &dst);
                                    let hd_after = slot_hamming_distance(&w[1], &dst);
                                    assert_eq!(hd_before, hd_after + 1,
                                        "HD did not decrease by 1: {} → {}",
                                        hd_before, hd_after);
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(failures, 0, "greedy routing failed for {} pairs", failures);
    }

    #[test]
    fn greedy_next_hop_at_destination() {
        let s = SlotAddress::new(1, 2, 3);
        assert!(greedy_next_hop(&s, &s).is_none());
    }
}
