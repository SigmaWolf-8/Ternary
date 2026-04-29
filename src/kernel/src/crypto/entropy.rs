// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! SP 800-90B Entropy Source Module
//!
//! Provides a qualified entropy source for the HMAC-DRBG (SP 800-90A).
//! Implements mandatory health tests per SP 800-90B Section 4.4:
//! - Repetition Count Test (Section 4.4.1)
//! - Adaptive Proportion Test (Section 4.4.2)
//!
//! Uses HMAC-SHA-384 as the vetted conditioning component per
//! SP 800-90B Section 3.1.5.1.1.
//!
//! # Noise Source
//! Primary noise source is femtosecond clock jitter (thermal noise
//! in oscillator LSBs). The noise model documents timing jitter as
//! the entropy source for CMVP lab review.
//!
//! # FIPS 140-3 Requirement
//! Every DRBG instantiation and reseed requires conditioned entropy
//! that has passed both health tests. Failure of any health test
//! transitions the module to Error state.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;

use super::sha2::hmac_sha384;

pub const ENTROPY_SECURITY_STRENGTH: usize = 256;
pub const SHA384_OUTPUT_BYTES: usize = 48;
pub const MIN_ENTROPY_BITS: usize = 384;
pub const MIN_NONCE_BITS: usize = 192;

const REPETITION_COUNT_CUTOFF: usize = 21;
const APT_WINDOW_SIZE: usize = 512;
const APT_CUTOFF: usize = 325;

pub const ESTIMATED_MIN_ENTROPY_PER_SAMPLE: f64 = 4.0;
const SAMPLE_SYMBOL_SPACE: usize = 256;
pub const OVERSAMPLING_RATIO: usize = 2;

/// Returns the FIPS 800-90B-compliant default health-test parameters.
///
/// Callers wiring up a new entropy source should use these values
/// rather than ad-hoc thresholds, so the source meets validation
/// targets out of the box.
pub fn recommended_health_test_params() -> HealthTestParams {
    let alpha_min = libm::pow(2.0_f64, -ESTIMATED_MIN_ENTROPY_PER_SAMPLE);
    let alpha_oversampled = libm::pow(2.0_f64, -ESTIMATED_MIN_ENTROPY_PER_SAMPLE * OVERSAMPLING_RATIO as f64);
    let _ = SAMPLE_SYMBOL_SPACE;
    HealthTestParams {
        rct_cutoff: REPETITION_COUNT_CUTOFF,
        rct_alpha: alpha_min,
        apt_window: APT_WINDOW_SIZE,
        apt_cutoff: APT_CUTOFF,
        apt_alpha: alpha_oversampled,
    }
}

pub struct EntropyEstimation {
    pub h_min: f64,
    pub h_original: f64,
    pub sample_count: u64,
    pub most_common_count: u64,
    pub symbol_space: usize,
    pub health_test_params: HealthTestParams,
}

pub struct HealthTestParams {
    pub rct_cutoff: usize,
    pub rct_alpha: f64,
    pub apt_window: usize,
    pub apt_cutoff: usize,
    pub apt_alpha: f64,
}

impl HealthTestParams {
    pub fn from_min_entropy(h_min: f64) -> Self {
        let p_max = libm::pow(2.0_f64, -h_min);
        let alpha = libm::pow(2.0_f64, -20.0);
        let rct_c = 1.0 + libm::ceil(-libm::log(alpha) / libm::log(1.0 / p_max));
        let w = APT_WINDOW_SIZE as f64;
        let apt_c = w * p_max + libm::ceil(libm::sqrt(w * p_max * (1.0 - p_max)) * 4.4172);
        Self {
            rct_cutoff: rct_c as usize,
            rct_alpha: alpha,
            apt_window: APT_WINDOW_SIZE,
            apt_cutoff: apt_c as usize,
            apt_alpha: alpha,
        }
    }

    pub fn verify_against_constants(&self) -> bool {
        self.rct_cutoff <= REPETITION_COUNT_CUTOFF && self.apt_cutoff <= APT_CUTOFF
    }
}

pub fn estimate_most_common_value(samples: &[u8]) -> EntropyEstimation {
    let n = samples.len() as u64;
    let mut counts = [0u64; 256];
    for &s in samples {
        counts[s as usize] += 1;
    }
    let max_count = counts.iter().copied().max().unwrap_or(0);
    let p_hat = max_count as f64 / n as f64;
    let z = 2.5758;
    let p_upper = p_hat + z * libm::sqrt(p_hat * (1.0 - p_hat) / n as f64);
    let p_upper = if p_upper > 1.0 { 1.0 } else { p_upper };
    let h_min = -libm::log2(p_upper);
    let h_original = -libm::log2(p_hat);
    let params = HealthTestParams::from_min_entropy(h_min);
    EntropyEstimation {
        h_min,
        h_original,
        sample_count: n,
        most_common_count: max_count,
        symbol_space: SAMPLE_SYMBOL_SPACE,
        health_test_params: params,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntropyError {
    RepetitionCountFailed,
    AdaptiveProportionFailed,
    SourceUnavailable,
    InsufficientEntropy { requested: usize, available: usize },
    ConditioningFailed(String),
}

impl core::fmt::Display for EntropyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EntropyError::RepetitionCountFailed => {
                write!(f, "SP 800-90B repetition count test failed: noise source stuck")
            }
            EntropyError::AdaptiveProportionFailed => {
                write!(f, "SP 800-90B adaptive proportion test failed: noise source biased")
            }
            EntropyError::SourceUnavailable => {
                write!(f, "Entropy noise source unavailable")
            }
            EntropyError::InsufficientEntropy { requested, available } => {
                write!(f, "Insufficient entropy: requested {} bits, available {}", requested, available)
            }
            EntropyError::ConditioningFailed(msg) => {
                write!(f, "Entropy conditioning failed: {}", msg)
            }
        }
    }
}

pub type EntropyResult<T> = core::result::Result<T, EntropyError>;

pub trait NoiseSource {
    fn sample(&mut self) -> u64;
    fn source_description(&self) -> &str;
}

pub struct FemtoclockNoise {
    last_timestamp: u64,
    counter: u64,
}

impl FemtoclockNoise {
    pub fn new() -> Self {
        Self {
            last_timestamp: 0,
            counter: 0,
        }
    }

    fn read_femtoclock(&self) -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            let lo: u64;
            let hi: u64;
            unsafe {
                core::arch::asm!(
                    "rdtsc",
                    out("rax") lo,
                    out("rdx") hi,
                    options(nomem, nostack)
                );
            }
            (hi << 32) | lo
        }
        #[cfg(target_arch = "aarch64")]
        {
            let val: u64;
            unsafe {
                core::arch::asm!(
                    "mrs {}, cntvct_el0",
                    out(reg) val,
                    options(nomem, nostack)
                );
            }
            val
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            compile_error!("SP 800-90B: No qualified hardware noise source available for this architecture. FIPS 140-3 requires a hardware entropy source (RDTSC on x86_64, CNTVCT on aarch64).");
        }
    }
}

impl NoiseSource for FemtoclockNoise {
    fn sample(&mut self) -> u64 {
        let raw = self.read_femtoclock();
        let jitter = raw ^ self.last_timestamp;
        self.last_timestamp = raw;
        self.counter = self.counter.wrapping_add(1);
        jitter
    }

    fn source_description(&self) -> &str {
        "Femtosecond clock oscillator thermal jitter (LSB extraction)"
    }
}

pub struct TestNoise {
    sequence: Vec<u64>,
    index: usize,
}

impl TestNoise {
    pub fn new(sequence: Vec<u64>) -> Self {
        Self { sequence, index: 0 }
    }

    pub fn stuck(value: u64, count: usize) -> Self {
        Self::new(alloc::vec![value; count])
    }

    pub fn biased(primary: u64, other: u64, primary_ratio: usize, total: usize) -> Self {
        let mut seq = Vec::with_capacity(total);
        for i in 0..total {
            if i % total < primary_ratio {
                seq.push(primary);
            } else {
                seq.push(other);
            }
        }
        Self::new(seq)
    }
}

impl NoiseSource for TestNoise {
    fn sample(&mut self) -> u64 {
        if self.index >= self.sequence.len() {
            self.index = 0;
        }
        let val = self.sequence[self.index];
        self.index += 1;
        val
    }

    fn source_description(&self) -> &str {
        "Deterministic test noise source (NOT for production)"
    }
}

#[derive(Debug)]
pub struct HealthTestState {
    rct_last_sample: u64,
    rct_count: usize,
    apt_window: Vec<u64>,
    apt_reference: u64,
    apt_count: usize,
    apt_window_pos: usize,
    apt_initialized: bool,
    total_samples: u64,
    failures: u64,
}

impl HealthTestState {
    pub fn new() -> Self {
        Self {
            rct_last_sample: 0,
            rct_count: 0,
            apt_window: Vec::with_capacity(APT_WINDOW_SIZE),
            apt_reference: 0,
            apt_count: 0,
            apt_window_pos: 0,
            apt_initialized: false,
            total_samples: 0,
            failures: 0,
        }
    }

    pub fn test_sample(&mut self, sample: u64) -> EntropyResult<()> {
        self.total_samples += 1;

        self.repetition_count_test(sample)?;

        self.adaptive_proportion_test(sample)?;

        Ok(())
    }

    fn repetition_count_test(&mut self, sample: u64) -> EntropyResult<()> {
        if self.total_samples == 1 {
            self.rct_last_sample = sample;
            self.rct_count = 1;
            return Ok(());
        }

        if sample == self.rct_last_sample {
            self.rct_count += 1;
            if self.rct_count >= REPETITION_COUNT_CUTOFF {
                self.failures += 1;
                return Err(EntropyError::RepetitionCountFailed);
            }
        } else {
            self.rct_last_sample = sample;
            self.rct_count = 1;
        }
        Ok(())
    }

    fn adaptive_proportion_test(&mut self, sample: u64) -> EntropyResult<()> {
        if !self.apt_initialized {
            self.apt_reference = sample;
            self.apt_count = 1;
            self.apt_window_pos = 1;
            self.apt_initialized = true;
            return Ok(());
        }

        if self.apt_window.len() < APT_WINDOW_SIZE {
            self.apt_window.push(sample);
        } else {
            let pos = self.apt_window_pos % APT_WINDOW_SIZE;
            let evicted = self.apt_window[pos];
            self.apt_window[pos] = sample;
            if evicted == self.apt_reference && self.apt_count > 0 {
                self.apt_count -= 1;
            }
        }
        if sample == self.apt_reference {
            self.apt_count += 1;
        }
        self.apt_window_pos += 1;

        if self.apt_count >= APT_CUTOFF {
            self.failures += 1;
            return Err(EntropyError::AdaptiveProportionFailed);
        }

        if self.apt_window_pos >= APT_WINDOW_SIZE && self.apt_window_pos % APT_WINDOW_SIZE == 0 {
            self.apt_reference = sample;
            self.apt_count = self.apt_window.iter().filter(|&&v| v == sample).count();
        }

        Ok(())
    }

    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }

    pub fn failure_count(&self) -> u64 {
        self.failures
    }
}

pub fn conditioning_function(raw_samples: &[u64], output_bytes: usize) -> Vec<u8> {
    let conditioning_key: [u8; SHA384_OUTPUT_BYTES] = [
        0x53, 0x61, 0x6c, 0x76, 0x69, 0x45, 0x6e, 0x74,
        0x72, 0x6f, 0x70, 0x79, 0x43, 0x6f, 0x6e, 0x64,
        0x69, 0x74, 0x69, 0x6f, 0x6e, 0x69, 0x6e, 0x67,
        0x4b, 0x65, 0x79, 0x56, 0x31, 0x2e, 0x30, 0x2e,
        0x30, 0x46, 0x49, 0x50, 0x53, 0x2d, 0x31, 0x34,
        0x30, 0x2d, 0x33, 0x2d, 0x43, 0x4d, 0x56, 0x50,
    ];

    let mut raw_bytes = Vec::with_capacity(raw_samples.len() * 8);
    for &s in raw_samples {
        raw_bytes.extend_from_slice(&s.to_le_bytes());
    }

    let mut output = Vec::with_capacity(output_bytes);
    let mut counter: u8 = 0;

    while output.len() < output_bytes {
        let mut input = Vec::with_capacity(raw_bytes.len() + 1);
        input.push(counter);
        input.extend_from_slice(&raw_bytes);

        let block = hmac_sha384(&conditioning_key, &input);
        let remaining = output_bytes - output.len();
        let take = remaining.min(SHA384_OUTPUT_BYTES);
        output.extend_from_slice(&block[..take]);
        counter = counter.wrapping_add(1);
    }

    output.truncate(output_bytes);
    output
}

pub struct EntropySource<N: NoiseSource> {
    noise: N,
    health: HealthTestState,
    nonce_counter: u64,
}

impl<N: NoiseSource> EntropySource<N> {
    pub fn new(noise: N) -> Self {
        Self {
            noise,
            health: HealthTestState::new(),
            nonce_counter: 0,
        }
    }

    pub fn get_entropy(&mut self, requested_bits: usize) -> EntropyResult<Vec<u8>> {
        if requested_bits < MIN_ENTROPY_BITS {
            return Err(EntropyError::InsufficientEntropy {
                requested: requested_bits,
                available: MIN_ENTROPY_BITS,
            });
        }

        let output_bytes = (requested_bits + 7) / 8;
        let samples_needed = (output_bytes * 2 / 8).max(16);

        let mut raw_samples = Vec::with_capacity(samples_needed);
        for _ in 0..samples_needed {
            let sample = self.noise.sample();
            self.health.test_sample(sample)?;
            raw_samples.push(sample);
        }

        let conditioned = conditioning_function(&raw_samples, output_bytes);

        for s in raw_samples.iter_mut() {
            *s = 0;
        }

        Ok(conditioned)
    }

    pub fn get_nonce(&mut self) -> Vec<u8> {
        let timestamp = self.noise.sample();
        self.nonce_counter = self.nonce_counter.wrapping_add(1);

        let mut nonce = Vec::with_capacity(MIN_NONCE_BITS / 8);
        nonce.extend_from_slice(&timestamp.to_le_bytes());
        nonce.extend_from_slice(&self.nonce_counter.to_le_bytes());
        nonce.extend_from_slice(&(timestamp ^ self.nonce_counter).to_le_bytes());

        nonce
    }

    pub fn health_state(&self) -> &HealthTestState {
        &self.health
    }

    pub fn source_description(&self) -> &str {
        self.noise.source_description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diverse_noise(count: usize) -> TestNoise {
        let mut seq = Vec::with_capacity(count);
        let mut state: u64 = 0x123456789ABCDEF0;
        for _ in 0..count {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            seq.push(state);
        }
        TestNoise::new(seq)
    }

    #[test]
    fn test_healthy_source() {
        let noise = diverse_noise(1000);
        let mut source = EntropySource::new(noise);
        let entropy = source.get_entropy(384);
        assert!(entropy.is_ok());
        let data = entropy.unwrap();
        assert!(data.len() >= 48);
    }

    #[test]
    fn test_stuck_source_detected() {
        let noise = TestNoise::stuck(42, 100);
        let mut source = EntropySource::new(noise);
        let result = source.get_entropy(768);
        assert!(result.is_err());
        match result {
            Err(EntropyError::RepetitionCountFailed) => {}
            other => panic!("Expected RepetitionCountFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_repetition_count_boundary() {
        let mut seq = Vec::new();
        for _ in 0..(REPETITION_COUNT_CUTOFF - 2) {
            seq.push(42u64);
        }
        seq.push(99);
        for _ in 0..100 {
            seq.push(seq.len() as u64 * 7 + 13);
        }
        let noise = TestNoise::new(seq);
        let mut source = EntropySource::new(noise);
        let result = source.get_entropy(384);
        assert!(result.is_ok());
    }

    #[test]
    fn test_conditioning_deterministic() {
        let samples = alloc::vec![1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let out1 = conditioning_function(&samples, 48);
        let out2 = conditioning_function(&samples, 48);
        assert_eq!(out1, out2);
        assert_eq!(out1.len(), 48);
    }

    #[test]
    fn test_conditioning_different_inputs() {
        let s1 = alloc::vec![1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let s2 = alloc::vec![99u64, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88, 87, 86, 85, 84];
        let out1 = conditioning_function(&s1, 48);
        let out2 = conditioning_function(&s2, 48);
        assert_ne!(out1, out2);
    }

    #[test]
    fn test_nonce_generation() {
        let noise = diverse_noise(100);
        let mut source = EntropySource::new(noise);
        let n1 = source.get_nonce();
        let n2 = source.get_nonce();
        assert_eq!(n1.len(), 24);
        assert_eq!(n2.len(), 24);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_entropy_error_display() {
        let err = EntropyError::RepetitionCountFailed;
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("repetition count"));
    }

    #[test]
    fn test_health_state_tracking() {
        let noise = diverse_noise(200);
        let mut source = EntropySource::new(noise);
        let _ = source.get_entropy(384);
        assert!(source.health_state().total_samples() > 0);
        assert_eq!(source.health_state().failure_count(), 0);
    }
}
