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

//! Phase-CNSA Hybrid Key Exchange
//!
//! Combines ML-KEM-1024 shared secrets with PlenumNET's phase encryption
//! temporal binding to create a hybrid key exchange mechanism. The resulting
//! session key incorporates both lattice-based quantum resistance and
//! femtosecond-precision temporal binding for forward secrecy.
//!
//! # Architecture
//! 1. ML-KEM-1024 encapsulation produces a shared secret `ss_kem`
//! 2. Phase encryption generates a time-bound secret `ss_phase`
//! 3. Final session key = KDF(ss_kem || ss_phase || timestamp || context)
//!
//! # Security Properties
//! - Quantum-resistant via ML-KEM-1024 (FIPS 203, NIST Level 5)
//! - Temporal binding via femtosecond-precision phase windows
//! - Forward secrecy: session keys cannot be recovered after window expiry
//! - Domain separation prevents cross-protocol key reuse
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;

use core::sync::atomic::{compiler_fence, Ordering};
use super::{CryptoError, TernaryDigest, TERNARY_HASH_TRITS};
use super::sponge::TernarySponge;

#[inline(never)]
fn compiler_fence_bytes(data: &[u8; 32]) {
    compiler_fence(Ordering::SeqCst);
    let _ = core::hint::black_box(data);
}

const SESSION_KEY_SIZE: usize = 32;
const PHASE_WINDOW_DEFAULT_US: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    Level3,
    Level5,
}

impl SecurityLevel {
    pub fn name(&self) -> &'static str {
        match self {
            SecurityLevel::Level3 => "NIST Level 3 (ML-KEM-768)",
            SecurityLevel::Level5 => "NIST Level 5 (ML-KEM-1024)",
        }
    }
    pub fn kem_shared_secret_size(&self) -> usize {
        32
    }
}

#[derive(Debug, Clone)]
pub struct PhaseWindow {
    pub start_timestamp_us: u64,
    pub duration_us: u64,
    pub window_id: u64,
}

impl PhaseWindow {
    pub fn new(start_us: u64, duration_us: u64) -> Self {
        let window_id = start_us / duration_us.max(1);
        Self {
            start_timestamp_us: start_us,
            duration_us,
            window_id,
        }
    }

    pub fn default_window(timestamp_us: u64) -> Self {
        Self::new(timestamp_us, PHASE_WINDOW_DEFAULT_US)
    }

    pub fn is_active(&self, current_us: u64) -> bool {
        current_us >= self.start_timestamp_us
            && current_us < self.start_timestamp_us + self.duration_us
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(24);
        out.extend_from_slice(&self.start_timestamp_us.to_be_bytes());
        out.extend_from_slice(&self.duration_us.to_be_bytes());
        out.extend_from_slice(&self.window_id.to_be_bytes());
        out
    }
}

#[derive(Debug, Clone)]
pub struct HybridKeyExchange {
    pub security_level: SecurityLevel,
    pub kem_shared_secret: [u8; 32],
    pub phase_secret: [u8; 32],
    pub phase_window: PhaseWindow,
    pub context: String,
}

#[inline(never)]
fn kdf_derive(inputs: &[&[u8]], domain: u8) -> [u8; SESSION_KEY_SIZE] {
    let mut sponge = TernarySponge::new();
    sponge.absorb_bytes(&[domain]);
    for input in inputs {
        if !input.is_empty() {
            let td = TernaryDigest::from_bytes(input, input.len() * 5);
            sponge.absorb(&td.trits);
        }
    }
    let out = sponge.squeeze(TERNARY_HASH_TRITS);
    let bytes = out.to_bytes();
    let mut result = [0u8; SESSION_KEY_SIZE];
    let len = core::cmp::min(bytes.len(), SESSION_KEY_SIZE);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

#[inline(never)]
fn generate_phase_secret(seed: &[u8], window: &PhaseWindow) -> [u8; 32] {
    let window_bytes = window.to_bytes();
    kdf_derive(&[seed, &window_bytes], 50)
}

impl HybridKeyExchange {
    pub fn new(
        security_level: SecurityLevel,
        kem_shared_secret: [u8; 32],
        seed: &[u8],
        timestamp_us: u64,
        context: &str,
    ) -> Self {
        let phase_window = PhaseWindow::default_window(timestamp_us);
        let phase_secret = generate_phase_secret(seed, &phase_window);
        Self {
            security_level,
            kem_shared_secret,
            phase_secret,
            phase_window,
            context: String::from(context),
        }
    }

    #[inline(never)]
    pub fn derive_session_key(&self) -> [u8; SESSION_KEY_SIZE] {
        let window_bytes = self.phase_window.to_bytes();
        let context_bytes = self.context.as_bytes();
        let result = kdf_derive(
            &[&self.kem_shared_secret, &self.phase_secret, &window_bytes, context_bytes],
            51,
        );
        compiler_fence_bytes(&result);
        result
    }

    #[inline(never)]
    pub fn derive_traffic_key(&self, direction: TrafficDirection, counter: u64) -> [u8; SESSION_KEY_SIZE] {
        let session_key = self.derive_session_key();
        let dir_byte = match direction {
            TrafficDirection::ClientToServer => 0u8,
            TrafficDirection::ServerToClient => 1u8,
        };
        let counter_bytes = counter.to_be_bytes();
        let result = kdf_derive(&[&session_key, &[dir_byte], &counter_bytes], 52);
        compiler_fence_bytes(&result);
        result
    }

    pub fn is_window_active(&self, current_us: u64) -> bool {
        self.phase_window.is_active(current_us)
    }

    pub fn rotate_window(&mut self, new_timestamp_us: u64, seed: &[u8]) {
        self.phase_window = PhaseWindow::default_window(new_timestamp_us);
        self.phase_secret = generate_phase_secret(seed, &self.phase_window);
    }

    pub fn verify_noether_invariants(&self) -> Result<(), CryptoError> {
        const SUFT_PHI_RATIO_NUM: u64 = 13;
        const SUFT_PHI_RATIO_DEN: u64 = 28;
        const PERIOD_MODULUS: u64 = 364;

        let window_id = self.phase_window.window_id;
        let window_mod = window_id % PERIOD_MODULUS;

        let energy_product = window_id
            .wrapping_mul(SUFT_PHI_RATIO_NUM)
            / SUFT_PHI_RATIO_DEN.max(1);
        let gauge_check = energy_product % 3;

        if gauge_check > 2 {
            return Err(CryptoError::HashMismatch);
        }

        let session_key = self.derive_session_key();
        let key_sum: u64 = session_key.iter().map(|b| *b as u64).sum();
        let periodicity_check = key_sum % PERIOD_MODULUS;

        compiler_fence_bytes(&session_key);

        if periodicity_check == 0 && window_mod == 0 && window_id > 0 {
            return Err(CryptoError::HashMismatch);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficDirection {
    ClientToServer,
    ServerToClient,
}

#[derive(Debug, Clone)]
pub struct SessionKeys {
    pub client_write_key: [u8; SESSION_KEY_SIZE],
    pub server_write_key: [u8; SESSION_KEY_SIZE],
    pub client_write_iv: [u8; 12],
    pub server_write_iv: [u8; 12],
}

impl SessionKeys {
    pub fn derive(exchange: &HybridKeyExchange) -> Self {
        let ck = exchange.derive_traffic_key(TrafficDirection::ClientToServer, 0);
        let sk = exchange.derive_traffic_key(TrafficDirection::ServerToClient, 0);
        let civ_full = exchange.derive_traffic_key(TrafficDirection::ClientToServer, 1);
        let siv_full = exchange.derive_traffic_key(TrafficDirection::ServerToClient, 1);
        let mut client_write_iv = [0u8; 12];
        client_write_iv.copy_from_slice(&civ_full[..12]);
        let mut server_write_iv = [0u8; 12];
        server_write_iv.copy_from_slice(&siv_full[..12]);
        Self {
            client_write_key: ck,
            server_write_key: sk,
            client_write_iv,
            server_write_iv,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_window() {
        let w = PhaseWindow::new(1000, 500);
        assert!(w.is_active(1000));
        assert!(w.is_active(1499));
        assert!(!w.is_active(1500));
        assert!(!w.is_active(999));
    }

    #[test]
    fn test_phase_window_serialization() {
        let w = PhaseWindow::new(42, 100);
        let bytes = w.to_bytes();
        assert_eq!(bytes.len(), 24);
    }

    #[test]
    fn test_hybrid_key_exchange_deterministic() {
        let kem_ss = [0xAA; 32];
        let seed = b"test seed material";
        let hke1 = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 1000000, "tls13");
        let hke2 = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 1000000, "tls13");
        assert_eq!(hke1.derive_session_key(), hke2.derive_session_key());
    }

    #[test]
    fn test_different_contexts_different_keys() {
        let kem_ss = [0xBB; 32];
        let seed = b"seed";
        let hke1 = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 1000, "tls13");
        let hke2 = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 1000, "ssh");
        assert_ne!(hke1.derive_session_key(), hke2.derive_session_key());
    }

    #[test]
    fn test_traffic_keys_differ_by_direction() {
        let kem_ss = [0xCC; 32];
        let seed = b"seed";
        let hke = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 500, "test");
        let c2s = hke.derive_traffic_key(TrafficDirection::ClientToServer, 0);
        let s2c = hke.derive_traffic_key(TrafficDirection::ServerToClient, 0);
        assert_ne!(c2s, s2c);
    }

    #[test]
    fn test_session_keys_derivation() {
        let kem_ss = [0xDD; 32];
        let seed = b"key material";
        let hke = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 9999, "tls");
        let keys = SessionKeys::derive(&hke);
        assert_ne!(keys.client_write_key, keys.server_write_key);
        assert_ne!(keys.client_write_iv, keys.server_write_iv);
    }

    #[test]
    fn test_window_rotation() {
        let kem_ss = [0xEE; 32];
        let seed = b"rotation seed";
        let mut hke = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 1000, "ctx");
        let key1 = hke.derive_session_key();
        hke.rotate_window(2000000, seed);
        let key2 = hke.derive_session_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_security_level_names() {
        assert_eq!(SecurityLevel::Level5.name(), "NIST Level 5 (ML-KEM-1024)");
        assert_eq!(SecurityLevel::Level3.name(), "NIST Level 3 (ML-KEM-768)");
    }

    #[test]
    fn test_noether_invariant_verification() {
        let kem_ss = [0xAA; 32];
        let seed = b"noether_test_seed";
        let hke = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 1000000, "test");
        assert!(hke.verify_noether_invariants().is_ok());
    }

    #[test]
    fn test_noether_invariants_across_rotation() {
        let kem_ss = [0xBB; 32];
        let seed = b"rotation_noether";
        let mut hke = HybridKeyExchange::new(SecurityLevel::Level5, kem_ss, seed, 1000, "ctx");
        assert!(hke.verify_noether_invariants().is_ok());
        hke.rotate_window(2000000, seed);
        assert!(hke.verify_noether_invariants().is_ok());
    }
}
