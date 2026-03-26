// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Coprime Walk: (7, 11, 13) over 540 nodes.
// gcd(1001, 540) = 1 — all nodes visited before repeating.

use alloc::vec::Vec;

pub const STRIDE_7: u32 = 7;
pub const STRIDE_11: u32 = 11;
pub const STRIDE_13: u32 = 13;
pub const COMBINED_STRIDE: u32 = STRIDE_7 * STRIDE_11 * STRIDE_13;
pub const RING_SIZE: u32 = 540;

#[derive(Debug, Clone)]
pub struct CoprimeWalker {
    position: u32,
    stride: u32,
    ring_size: u32,
    steps_taken: u32,
}

impl CoprimeWalker {
    pub fn new(stride: u32, ring_size: u32) -> Self {
        Self {
            position: 0,
            stride,
            ring_size,
            steps_taken: 0,
        }
    }

    pub fn with_combined_stride() -> Self {
        Self::new(COMBINED_STRIDE, RING_SIZE)
    }

    pub fn step(&mut self) -> u32 {
        self.position = (self.position + self.stride) % self.ring_size;
        self.steps_taken += 1;
        self.position
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn steps_taken(&self) -> u32 {
        self.steps_taken
    }

    pub fn is_complete_cycle(&self) -> bool {
        self.steps_taken >= self.ring_size && self.position == 0
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.steps_taken = 0;
    }

    pub fn assign_request(&mut self, _request_id: u64) -> u32 {
        self.step()
    }
}

pub struct ParallelWalker {
    walkers: [CoprimeWalker; 3],
}

impl ParallelWalker {
    pub fn new() -> Self {
        Self {
            walkers: [
                CoprimeWalker::new(STRIDE_7, RING_SIZE),
                CoprimeWalker::new(STRIDE_11, RING_SIZE),
                CoprimeWalker::new(STRIDE_13, RING_SIZE),
            ],
        }
    }

    pub fn step_all(&mut self) -> [u32; 3] {
        [
            self.walkers[0].step(),
            self.walkers[1].step(),
            self.walkers[2].step(),
        ]
    }

    pub fn walker(&self, index: usize) -> Option<&CoprimeWalker> {
        self.walkers.get(index)
    }
}

pub fn verify_full_coverage(stride: u32, ring_size: u32) -> bool {
    gcd(stride, ring_size) == 1
}

pub fn compute_walk_sequence(stride: u32, ring_size: u32) -> Vec<u32> {
    let mut visited = Vec::with_capacity(ring_size as usize);
    let mut pos: u32 = 0;
    for _ in 0..ring_size {
        pos = (pos + stride) % ring_size;
        visited.push(pos);
    }
    visited
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet;

    #[test]
    fn test_gcd_coprime() {
        assert_eq!(gcd(COMBINED_STRIDE, RING_SIZE), 1);
        assert_eq!(COMBINED_STRIDE, 1001);
        assert_eq!(RING_SIZE, 540);
    }

    #[test]
    fn test_individual_strides_coprime() {
        assert_eq!(gcd(STRIDE_7, RING_SIZE), 1);
        assert_eq!(gcd(STRIDE_11, RING_SIZE), 1);
        assert_eq!(gcd(STRIDE_13, RING_SIZE), 1);
    }

    #[test]
    fn test_combined_stride_full_coverage() {
        let seq = compute_walk_sequence(COMBINED_STRIDE, RING_SIZE);
        let unique: BTreeSet<u32> = seq.iter().copied().collect();
        assert_eq!(unique.len(), RING_SIZE as usize);
    }

    #[test]
    fn test_stride_7_full_coverage() {
        let seq = compute_walk_sequence(STRIDE_7, RING_SIZE);
        let unique: BTreeSet<u32> = seq.iter().copied().collect();
        assert_eq!(unique.len(), RING_SIZE as usize);
    }

    #[test]
    fn test_stride_11_full_coverage() {
        let seq = compute_walk_sequence(STRIDE_11, RING_SIZE);
        let unique: BTreeSet<u32> = seq.iter().copied().collect();
        assert_eq!(unique.len(), RING_SIZE as usize);
    }

    #[test]
    fn test_stride_13_full_coverage() {
        let seq = compute_walk_sequence(STRIDE_13, RING_SIZE);
        let unique: BTreeSet<u32> = seq.iter().copied().collect();
        assert_eq!(unique.len(), RING_SIZE as usize);
    }

    #[test]
    fn test_walker_cycle() {
        let mut walker = CoprimeWalker::with_combined_stride();
        for _ in 0..RING_SIZE {
            walker.step();
        }
        assert!(walker.is_complete_cycle());
    }

    #[test]
    fn test_parallel_walker() {
        let mut pw = ParallelWalker::new();
        let positions = pw.step_all();
        assert_eq!(positions[0], 7);
        assert_eq!(positions[1], 11);
        assert_eq!(positions[2], 13);
    }

    #[test]
    fn test_verify_full_coverage() {
        assert!(verify_full_coverage(1001, 540));
        assert!(!verify_full_coverage(10, 540));
    }
}
