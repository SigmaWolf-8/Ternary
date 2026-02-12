//! SP 800-90A HMAC-DRBG (Deterministic Random Bit Generator)
//!
//! Implements HMAC-DRBG per SP 800-90A Rev.1 Section 10.1.2 using
//! HMAC-SHA-384 as the underlying PRF. This is the ONLY approved
//! random number generation mechanism within the FIPS 140-3 module
//! boundary.
//!
//! # Security Strength
//! 256 bits (CNSA 2.0 minimum), achieved via SHA-384 (384-bit output).
//!
//! # FIPS 140-3 Requirement
//! All cryptographic operations requiring randomness MUST consume
//! output exclusively from `drbg_generate()`. Raw entropy sources
//! feed INTO the DRBG via `drbg_instantiate()` and `drbg_reseed()`.
//!
//! # SP 800-90A Limits (Table 2, HMAC-DRBG)
//! - Max requests between reseeds: 2^48
//! - Max bits per request: 2^19 (65536 bytes)
//! - Reseed interval enforcement: mandatory
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;

use super::sha2::hmac_sha384;

pub const DRBG_SECURITY_STRENGTH: usize = 256;
pub const DRBG_SEED_LEN: usize = 48;
pub const DRBG_MAX_BYTES_PER_REQUEST: usize = 65536;
pub const DRBG_RESEED_INTERVAL: u64 = 1u64 << 48;
const OUTLEN: usize = 48;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrbgError {
    NotInstantiated,
    ReseedRequired,
    RequestTooLarge { requested: usize, max: usize },
    EntropyTooShort { provided: usize, minimum: usize },
    NonceTooShort { provided: usize, minimum: usize },
    ContinuousTestFailed,
    AlreadyUninstantiated,
    InvalidState(String),
}

impl core::fmt::Display for DrbgError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DrbgError::NotInstantiated => write!(f, "DRBG not instantiated"),
            DrbgError::ReseedRequired => write!(f, "DRBG reseed required: reseed counter exceeded"),
            DrbgError::RequestTooLarge { requested, max } => {
                write!(f, "DRBG request too large: {} bytes (max {})", requested, max)
            }
            DrbgError::EntropyTooShort { provided, minimum } => {
                write!(f, "Entropy input too short: {} bytes (min {})", provided, minimum)
            }
            DrbgError::NonceTooShort { provided, minimum } => {
                write!(f, "Nonce too short: {} bytes (min {})", provided, minimum)
            }
            DrbgError::ContinuousTestFailed => {
                write!(f, "DRBG continuous random number generator test failed")
            }
            DrbgError::AlreadyUninstantiated => write!(f, "DRBG already uninstantiated"),
            DrbgError::InvalidState(msg) => write!(f, "DRBG invalid state: {}", msg),
        }
    }
}

pub type DrbgResult<T> = core::result::Result<T, DrbgError>;

#[derive(Clone, Debug)]
pub struct DrbgState {
    key: [u8; OUTLEN],
    v: [u8; OUTLEN],
    reseed_counter: u64,
    prediction_resistance: bool,
    instantiated: bool,
    last_output_block: Option<[u8; OUTLEN]>,
}

impl DrbgState {
    fn zeroed() -> Self {
        Self {
            key: [0u8; OUTLEN],
            v: [0u8; OUTLEN],
            reseed_counter: 0,
            prediction_resistance: false,
            instantiated: false,
            last_output_block: None,
        }
    }
}

fn hmac_drbg_update(provided_data: Option<&[u8]>, key: &mut [u8; OUTLEN], v: &mut [u8; OUTLEN]) {
    let mut input = Vec::with_capacity(OUTLEN + 1 + provided_data.map_or(0, |d| d.len()));
    input.extend_from_slice(v);
    input.push(0x00);
    if let Some(data) = provided_data {
        input.extend_from_slice(data);
    }
    let new_key = hmac_sha384(key, &input);
    key.copy_from_slice(&new_key);

    let new_v = hmac_sha384(key, v);
    v.copy_from_slice(&new_v);

    if let Some(data) = provided_data {
        if !data.is_empty() {
            let mut input2 = Vec::with_capacity(OUTLEN + 1 + data.len());
            input2.extend_from_slice(v);
            input2.push(0x01);
            input2.extend_from_slice(data);
            let new_key2 = hmac_sha384(key, &input2);
            key.copy_from_slice(&new_key2);

            let new_v2 = hmac_sha384(key, v);
            v.copy_from_slice(&new_v2);
        }
    }
}

pub fn drbg_instantiate(
    entropy: &[u8],
    nonce: &[u8],
    personalization: Option<&[u8]>,
    prediction_resistance: bool,
) -> DrbgResult<DrbgState> {
    if entropy.len() < OUTLEN {
        return Err(DrbgError::EntropyTooShort {
            provided: entropy.len(),
            minimum: OUTLEN,
        });
    }
    if nonce.len() < 24 {
        return Err(DrbgError::NonceTooShort {
            provided: nonce.len(),
            minimum: 24,
        });
    }

    let mut state = DrbgState::zeroed();
    state.key = [0x00u8; OUTLEN];
    state.v = [0x01u8; OUTLEN];

    let mut seed_material = Vec::with_capacity(entropy.len() + nonce.len() + personalization.map_or(0, |p| p.len()));
    seed_material.extend_from_slice(entropy);
    seed_material.extend_from_slice(nonce);
    if let Some(pers) = personalization {
        seed_material.extend_from_slice(pers);
    }

    hmac_drbg_update(Some(&seed_material), &mut state.key, &mut state.v);

    for b in seed_material.iter_mut() {
        *b = 0;
    }

    state.reseed_counter = 1;
    state.prediction_resistance = prediction_resistance;
    state.instantiated = true;

    Ok(state)
}

pub fn drbg_reseed(
    state: &mut DrbgState,
    entropy: &[u8],
    additional_input: Option<&[u8]>,
) -> DrbgResult<()> {
    if !state.instantiated {
        return Err(DrbgError::NotInstantiated);
    }
    if entropy.len() < OUTLEN {
        return Err(DrbgError::EntropyTooShort {
            provided: entropy.len(),
            minimum: OUTLEN,
        });
    }

    let mut seed_material = Vec::with_capacity(entropy.len() + additional_input.map_or(0, |a| a.len()));
    seed_material.extend_from_slice(entropy);
    if let Some(ai) = additional_input {
        seed_material.extend_from_slice(ai);
    }

    hmac_drbg_update(Some(&seed_material), &mut state.key, &mut state.v);

    for b in seed_material.iter_mut() {
        *b = 0;
    }

    state.reseed_counter = 1;
    Ok(())
}

pub fn drbg_generate(
    state: &mut DrbgState,
    requested_bytes: usize,
    additional_input: Option<&[u8]>,
) -> DrbgResult<Vec<u8>> {
    if !state.instantiated {
        return Err(DrbgError::NotInstantiated);
    }
    if requested_bytes > DRBG_MAX_BYTES_PER_REQUEST {
        return Err(DrbgError::RequestTooLarge {
            requested: requested_bytes,
            max: DRBG_MAX_BYTES_PER_REQUEST,
        });
    }
    if state.reseed_counter > DRBG_RESEED_INTERVAL {
        return Err(DrbgError::ReseedRequired);
    }

    if let Some(ai) = additional_input {
        if !ai.is_empty() {
            hmac_drbg_update(Some(ai), &mut state.key, &mut state.v);
        }
    }

    let mut output = Vec::with_capacity(requested_bytes);
    while output.len() < requested_bytes {
        let new_v = hmac_sha384(&state.key, &state.v);
        state.v.copy_from_slice(&new_v);

        if let Some(ref last) = state.last_output_block {
            if *last == state.v {
                state.instantiated = false;
                return Err(DrbgError::ContinuousTestFailed);
            }
        }
        state.last_output_block = Some(state.v);

        let remaining = requested_bytes - output.len();
        let take = remaining.min(OUTLEN);
        output.extend_from_slice(&state.v[..take]);
    }

    output.truncate(requested_bytes);

    hmac_drbg_update(additional_input, &mut state.key, &mut state.v);

    state.reseed_counter += 1;

    Ok(output)
}

pub fn drbg_uninstantiate(state: &mut DrbgState) -> DrbgResult<()> {
    if !state.instantiated {
        return Err(DrbgError::AlreadyUninstantiated);
    }

    for b in state.key.iter_mut() {
        *b = 0;
    }
    for b in state.v.iter_mut() {
        *b = 0;
    }
    state.reseed_counter = 0;
    state.prediction_resistance = false;
    state.instantiated = false;
    state.last_output_block = None;

    Ok(())
}

pub fn drbg_instantiation_test() -> DrbgResult<bool> {
    let entropy = [0x06u8; 48];
    let nonce = [0x07u8; 24];
    let pers = b"HMAC-DRBG-SHA384-POST";

    let mut state = drbg_instantiate(&entropy, &nonce, Some(pers), false)?;
    let output = drbg_generate(&mut state, 48, None)?;

    if output.len() != 48 {
        return Ok(false);
    }
    if output.iter().all(|&b| b == 0) {
        return Ok(false);
    }

    let output2 = drbg_generate(&mut state, 48, None)?;
    if output == output2 {
        return Ok(false);
    }

    drbg_uninstantiate(&mut state)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entropy() -> [u8; 48] {
        let mut e = [0u8; 48];
        for i in 0..48 {
            e[i] = (i as u8).wrapping_mul(7).wrapping_add(13);
        }
        e
    }

    fn test_nonce() -> [u8; 24] {
        let mut n = [0u8; 24];
        for i in 0..24 {
            n[i] = (i as u8).wrapping_mul(11).wrapping_add(5);
        }
        n
    }

    #[test]
    fn test_instantiate_and_generate() {
        let entropy = test_entropy();
        let nonce = test_nonce();
        let mut state = drbg_instantiate(&entropy, &nonce, None, false).unwrap();
        assert!(state.instantiated);

        let output = drbg_generate(&mut state, 32, None).unwrap();
        assert_eq!(output.len(), 32);
        assert!(!output.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_deterministic_output() {
        let entropy = test_entropy();
        let nonce = test_nonce();

        let mut s1 = drbg_instantiate(&entropy, &nonce, Some(b"test"), false).unwrap();
        let mut s2 = drbg_instantiate(&entropy, &nonce, Some(b"test"), false).unwrap();

        let o1 = drbg_generate(&mut s1, 64, None).unwrap();
        let o2 = drbg_generate(&mut s2, 64, None).unwrap();
        assert_eq!(o1, o2, "HMAC-DRBG must be deterministic for same inputs");
    }

    #[test]
    fn test_different_entropy_different_output() {
        let nonce = test_nonce();

        let mut e1 = test_entropy();
        let mut e2 = test_entropy();
        e2[0] ^= 0xFF;

        let mut s1 = drbg_instantiate(&e1, &nonce, None, false).unwrap();
        let mut s2 = drbg_instantiate(&e2, &nonce, None, false).unwrap();

        let o1 = drbg_generate(&mut s1, 48, None).unwrap();
        let o2 = drbg_generate(&mut s2, 48, None).unwrap();
        assert_ne!(o1, o2);
    }

    #[test]
    fn test_reseed() {
        let entropy = test_entropy();
        let nonce = test_nonce();
        let mut state = drbg_instantiate(&entropy, &nonce, None, false).unwrap();

        let before = drbg_generate(&mut state, 32, None).unwrap();

        let mut reseed_entropy = test_entropy();
        reseed_entropy[0] = 0xFF;
        drbg_reseed(&mut state, &reseed_entropy, None).unwrap();
        assert_eq!(state.reseed_counter, 1);

        let after = drbg_generate(&mut state, 32, None).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn test_reseed_counter_increments() {
        let entropy = test_entropy();
        let nonce = test_nonce();
        let mut state = drbg_instantiate(&entropy, &nonce, None, false).unwrap();
        assert_eq!(state.reseed_counter, 1);

        let _ = drbg_generate(&mut state, 32, None).unwrap();
        assert_eq!(state.reseed_counter, 2);

        let _ = drbg_generate(&mut state, 32, None).unwrap();
        assert_eq!(state.reseed_counter, 3);
    }

    #[test]
    fn test_request_too_large() {
        let entropy = test_entropy();
        let nonce = test_nonce();
        let mut state = drbg_instantiate(&entropy, &nonce, None, false).unwrap();

        let result = drbg_generate(&mut state, DRBG_MAX_BYTES_PER_REQUEST + 1, None);
        assert!(result.is_err());
        match result {
            Err(DrbgError::RequestTooLarge { .. }) => {}
            other => panic!("Expected RequestTooLarge, got {:?}", other),
        }
    }

    #[test]
    fn test_entropy_too_short() {
        let result = drbg_instantiate(&[0u8; 10], &[0u8; 24], None, false);
        assert!(result.is_err());
        match result {
            Err(DrbgError::EntropyTooShort { .. }) => {}
            other => panic!("Expected EntropyTooShort, got {:?}", other),
        }
    }

    #[test]
    fn test_nonce_too_short() {
        let result = drbg_instantiate(&[0u8; 48], &[0u8; 10], None, false);
        assert!(result.is_err());
        match result {
            Err(DrbgError::NonceTooShort { .. }) => {}
            other => panic!("Expected NonceTooShort, got {:?}", other),
        }
    }

    #[test]
    fn test_uninstantiate_zeroizes() {
        let entropy = test_entropy();
        let nonce = test_nonce();
        let mut state = drbg_instantiate(&entropy, &nonce, None, false).unwrap();
        let _ = drbg_generate(&mut state, 32, None).unwrap();

        drbg_uninstantiate(&mut state).unwrap();
        assert!(!state.instantiated);
        assert!(state.key.iter().all(|&b| b == 0));
        assert!(state.v.iter().all(|&b| b == 0));
        assert_eq!(state.reseed_counter, 0);
    }

    #[test]
    fn test_generate_after_uninstantiate_fails() {
        let entropy = test_entropy();
        let nonce = test_nonce();
        let mut state = drbg_instantiate(&entropy, &nonce, None, false).unwrap();
        drbg_uninstantiate(&mut state).unwrap();

        let result = drbg_generate(&mut state, 32, None);
        assert!(result.is_err());
        match result {
            Err(DrbgError::NotInstantiated) => {}
            other => panic!("Expected NotInstantiated, got {:?}", other),
        }
    }

    #[test]
    fn test_additional_input() {
        let entropy = test_entropy();
        let nonce = test_nonce();

        let mut s1 = drbg_instantiate(&entropy, &nonce, None, false).unwrap();
        let mut s2 = drbg_instantiate(&entropy, &nonce, None, false).unwrap();

        let o1 = drbg_generate(&mut s1, 32, Some(b"additional")).unwrap();
        let o2 = drbg_generate(&mut s2, 32, None).unwrap();
        assert_ne!(o1, o2, "Additional input should change output");
    }

    #[test]
    fn test_instantiation_self_test() {
        let result = drbg_instantiation_test().unwrap();
        assert!(result, "DRBG instantiation self-test must pass");
    }

    #[test]
    fn test_multiple_generate_calls() {
        let entropy = test_entropy();
        let nonce = test_nonce();
        let mut state = drbg_instantiate(&entropy, &nonce, None, false).unwrap();

        let mut outputs = Vec::new();
        for _ in 0..10 {
            let out = drbg_generate(&mut state, 48, None).unwrap();
            assert!(!outputs.contains(&out), "DRBG outputs must be unique");
            outputs.push(out);
        }
    }

    #[test]
    fn test_error_display() {
        let err = DrbgError::ReseedRequired;
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("reseed"));
    }
}
