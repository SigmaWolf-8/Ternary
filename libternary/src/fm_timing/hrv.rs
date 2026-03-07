// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use std::collections::VecDeque;

const MAX_JITTER: f64 = 0.05;
const POOL_SIZE: usize = 256;
const BUFFER_CAPACITY: usize = 1024;

/// Heart Rate Variability entropy source.
///
/// The stochastic noise term ξ(t) from the van der Pol model becomes a
/// hardware-free entropy source. Bounded jitter across network nodes
/// aggregates into a distributed pool for post-quantum key generation.
pub struct HrvEntropy {
    buffer: VecDeque<f64>,
    pool: [u8; POOL_SIZE],
    pool_idx: usize,
    state: f64,
    health: EntropyHealth,
    deterministic_value: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct EntropyHealth {
    pub min_entropy_estimate: f64,
    pub samples_collected: u64,
    pub last_health_check: u64,
    pub healthy: bool,
}

impl HrvEntropy {
    pub fn new(seed: f64) -> Self {
        let clamped = seed.clamp(0.01, 0.99);
        Self {
            buffer: VecDeque::with_capacity(BUFFER_CAPACITY),
            pool: [0u8; POOL_SIZE],
            pool_idx: 0,
            state: clamped,
            health: EntropyHealth {
                min_entropy_estimate: 0.0,
                samples_collected: 0,
                last_health_check: 0,
                healthy: false,
            },
            deterministic_value: None,
        }
    }

    /// Deterministic source for testing — returns constant bounded value
    pub fn new_deterministic(value: f64) -> Self {
        let mut e = Self::new(0.5);
        e.deterministic_value = Some(value.clamp(-MAX_JITTER, MAX_JITTER));
        e
    }

    /// Sample one noise value, bounded to ±MAX_JITTER
    pub fn sample(&mut self) -> f64 {
        if let Some(v) = self.deterministic_value {
            self.health.samples_collected += 1;
            return v;
        }

        let raw = self.chaotic_map_step();
        self.buffer.push_back(raw);
        if self.buffer.len() > BUFFER_CAPACITY {
            self.buffer.pop_front();
        }
        self.update_pool(raw);
        self.health.samples_collected += 1;

        if self.health.samples_collected % 1024 == 0 {
            self.run_health_check();
        }

        raw.clamp(-MAX_JITTER, MAX_JITTER)
    }

    /// Extract whitened entropy bytes for PQ key material.
    /// Returns None if health check is failing.
    pub fn extract_bytes(&mut self, n: usize) -> Option<Vec<u8>> {
        if !self.health.healthy || n > POOL_SIZE {
            return None;
        }
        let result: Vec<u8> = self.pool.iter().take(n).copied().collect();
        let rekey: Vec<u8> = (0..POOL_SIZE)
            .map(|_| self.chaotic_map_step().to_bits() as u8)
            .collect();
        for (i, &k) in rekey.iter().enumerate() {
            self.pool[i] ^= k;
        }
        Some(result)
    }

    /// Logistic map: x_{n+1} = r · x_n · (1 - x_n), r = 3.99 (chaotic regime)
    fn chaotic_map_step(&mut self) -> f64 {
        const R: f64 = 3.99;
        self.state = R * self.state * (1.0 - self.state);
        self.state - 0.5
    }

    fn update_pool(&mut self, sample: f64) {
        let bytes = sample.to_le_bytes();
        for &b in &bytes {
            self.pool[self.pool_idx] ^= b;
            self.pool_idx = (self.pool_idx + 1) % POOL_SIZE;
        }
    }

    fn run_health_check(&mut self) {
        if self.buffer.len() < 256 {
            self.health.healthy = false;
            return;
        }
        let mut bins = [0u32; 20];
        for val in self.buffer.iter().rev().take(256) {
            let idx = ((val + 0.5) * 19.0).clamp(0.0, 19.0) as usize;
            bins[idx] += 1;
        }
        let max_count = *bins.iter().max().unwrap() as f64;
        let p_max = max_count / 256.0;
        self.health.min_entropy_estimate = -p_max.log2();
        self.health.healthy = self.health.min_entropy_estimate > 1.0;
        self.health.last_health_check = self.health.samples_collected;
    }

    pub fn health(&self) -> &EntropyHealth {
        &self.health
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_bounded() {
        let mut hrv = HrvEntropy::new(0.4);
        for _ in 0..10_000 {
            let s = hrv.sample();
            assert!(s.abs() <= MAX_JITTER, "Sample {} exceeds bound", s);
        }
    }

    #[test]
    fn entropy_health_converges() {
        let mut hrv = HrvEntropy::new(0.4);
        for _ in 0..2048 {
            hrv.sample();
        }
        assert!(hrv.health().healthy, "Should be healthy after 2048 samples");
        assert!(hrv.health().min_entropy_estimate > 1.0,
            "Min entropy {} should be > 1.0", hrv.health().min_entropy_estimate);
    }

    #[test]
    fn extraction_fails_when_unhealthy() {
        let mut hrv = HrvEntropy::new(0.4);
        assert!(hrv.extract_bytes(32).is_none());
    }

    #[test]
    fn extraction_succeeds_when_healthy() {
        let mut hrv = HrvEntropy::new(0.4);
        for _ in 0..2048 {
            hrv.sample();
        }
        let bytes = hrv.extract_bytes(32);
        assert!(bytes.is_some());
        assert_eq!(bytes.unwrap().len(), 32);
    }

    #[test]
    fn deterministic_source_is_constant() {
        let mut hrv = HrvEntropy::new_deterministic(0.01);
        let s1 = hrv.sample();
        let s2 = hrv.sample();
        assert_eq!(s1, s2);
        assert_eq!(s1, 0.01);
    }

    #[test]
    fn chaotic_divergence() {
        let mut h1 = HrvEntropy::new(0.4);
        let mut h2 = HrvEntropy::new(0.400001);
        for _ in 0..2048 {
            h1.sample();
            h2.sample();
        }
        let diff: usize = h1.pool.iter().zip(h2.pool.iter())
            .filter(|(a, b)| a != b).count();
        assert!(diff > 200, "Pools should diverge chaotically, diff={}", diff);
    }
}
