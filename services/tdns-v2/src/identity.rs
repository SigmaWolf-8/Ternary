// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # TDNS Identity Layer
//!
//! Wraps the canonical TIS-27 sponge (`ternary_math::sponge`) with the
//! TDNS-specific concerns: URL canonicalization, Trit-typed output,
//! and Collision GUID encoding.
//!
//! All cryptographic work lives in `ternary_math::sponge`.  This module
//! is pure glue + domain logic.
//!
//! ## Address Space
//!
//! - Identity space: 3²⁷ = 7,625,597,484,987 unique identities
//! - With CGUID:     3²⁷ × 9 = 68,630,377,364,883 registry slots
//! - 50% birthday collision at ~3,251,357 entries
//!
//! ## Wire Format
//!
//! 54 functional trits + 2 CGUID trits = 56 trit slots = 14 bytes.
//! Version tag: 0x25.

pub use ternary_math::sponge::{derive_key, derive_link_keys, mac, verify_mac};

use crate::trit::Trit;

// ─── Identity Derivation ─────────────────────────────────────────────────────

/// Derive the 27-trit identity anchor for a canonical URL.
///
/// Output is in Rep C {1,2,3}.  Zero is structurally impossible.
pub fn derive_identity(canonical_url: &str) -> [Trit; 27] {
    let bytes = ternary_math::sponge::derive_key(
        b"tis27-identity-v1",
        canonical_url.as_bytes(),
        27, // one output byte per identity trit — values reduced mod 3 below
    );

    let mut out = [Trit::V1; 27];
    for (i, b) in bytes.iter().enumerate() {
        // Map byte → GF(3) → Rep C {1,2,3}: (b % 3) gives {0,1,2}; +1 gives {1,2,3}
        let gf = (b % 3) + 1; // 1, 2, or 3 — never 0
        out[i] = Trit::from_repr_c(gf).unwrap_or(Trit::V1);
    }
    out
}

// ─── URL Canonicalization ────────────────────────────────────────────────────

/// Canonicalize a URL for deterministic identity derivation.
///
/// Rules:
/// - Strip fragment (#...)
/// - Strip query string (?...)
/// - Lowercase scheme and host
/// - Remove default ports (443 for https, 80 for http)
/// - Remove trailing slash on bare origins
pub fn canonicalise_url(raw: &str) -> String {
    let no_fragment = raw.split('#').next().unwrap_or(raw);
    let no_query    = no_fragment.split('?').next().unwrap_or(no_fragment);
    if let Some(scheme_end) = no_query.find("://") {
        let scheme = no_query[..scheme_end].to_lowercase();
        let rest   = &no_query[scheme_end + 3..];
        let (authority, path) = if let Some(slash) = rest.find('/') {
            (&rest[..slash], &rest[slash..])
        } else {
            (rest, "/")
        };
        let (host, port) = if let Some(colon) = authority.rfind(':') {
            let (h, p) = (&authority[..colon], &authority[colon + 1..]);
            if p.chars().all(|c| c.is_ascii_digit()) { (h, p) } else { (authority, "") }
        } else {
            (authority, "")
        };
        let is_default = (scheme == "https" && (port == "443" || port.is_empty()))
                      || (scheme == "http"  && (port == "80"  || port.is_empty()));
        let port_str = if is_default || port.is_empty() { String::new() } else { format!(":{port}") };
        let path_str = if path == "/" { "" } else { path };
        return format!("{}://{}{}{}", scheme, host.to_lowercase(), port_str, path_str);
    }
    no_query.to_lowercase()
}

// ─── Collision GUID ──────────────────────────────────────────────────────────
//
// The two trailing trit slots in the wire format (byte 13, bits 3:0) encode
// a Collision GUID (1–9) that disambiguates registry entries sharing the same
// 27-trit identity anchor.
//
// Encoding: CGUID = (CG1 − 1) × 3 + CG2   where CG1, CG2 ∈ {1,2,3}
//
//   CGUID 1 → CG1=1 CG2=1  ← default, first registration
//   CGUID 9 → CG1=3 CG2=3
//
// Zero in either CG slot is forbidden — it is the forgery sentinel.

/// Encode a Collision GUID (1–9) into two Rep C trit bytes.
/// Panics if `cguid` is outside 1–9.
pub fn encode_cguid(cguid: u8) -> (u8, u8) {
    assert!(cguid >= 1 && cguid <= 9, "CGUID must be 1–9, got {cguid}");
    let z  = cguid - 1;
    (z / 3 + 1, z % 3 + 1)
}

/// Decode two Rep C trit values into a Collision GUID (1–9).
/// Returns `Err` if either value is 0 (forgery sentinel) or > 3.
pub fn decode_cguid(cg1: u8, cg2: u8) -> Result<u8, &'static str> {
    if cg1 == 0 || cg2 == 0 { return Err("CGUID trit is zero — forgery sentinel"); }
    if cg1 > 3  || cg2 > 3  { return Err("CGUID trit out of Rep C range"); }
    Ok((cg1 - 1) * 3 + cg2)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL canonicalization ────────────────────────────────────────────────

    #[test] fn canonical_trailing_slash()    { assert_eq!(canonicalise_url("https://google.com/"), "https://google.com"); }
    #[test] fn canonical_uppercase()         { assert_eq!(canonicalise_url("HTTPS://GOOGLE.COM"), "https://google.com"); }
    #[test] fn canonical_port_443()          { assert_eq!(canonicalise_url("https://google.com:443"), "https://google.com"); }
    #[test] fn canonical_port_80()           { assert_eq!(canonicalise_url("http://example.com:80/"), "http://example.com"); }
    #[test] fn canonical_nondefault_port()   { assert_eq!(canonicalise_url("https://example.com:8443/api"), "https://example.com:8443/api"); }
    #[test] fn canonical_fragment()          { assert_eq!(canonicalise_url("https://google.com/#r"), "https://google.com"); }
    #[test] fn canonical_query()             { assert_eq!(canonicalise_url("https://google.com/s?q=x"), "https://google.com/s"); }

    // ── Identity derivation ─────────────────────────────────────────────────

    #[test] fn identity_length()        { assert_eq!(derive_identity("https://google.com").len(), 27); }
    #[test] fn identity_deterministic() { assert_eq!(derive_identity("https://google.com"), derive_identity("https://google.com")); }
    #[test] fn identity_rep_c_only() {
        for t in derive_identity("https://google.com") {
            let v = t.to_repr_c();
            assert!(v >= 1 && v <= 3, "trit out of Rep C: {v}");
        }
    }
    #[test] fn identity_zero_never() {
        for url in &["https://google.com", "https://plenumnet.replit.app", "http://example.com"] {
            for (i, t) in derive_identity(url).iter().enumerate() {
                assert_ne!(t.to_repr_c(), 0, "zero at position {i} for {url}");
            }
        }
    }
    #[test] fn identity_distinct()            { assert_ne!(derive_identity("https://google.com"), derive_identity("https://bing.com")); }
    #[test] fn identity_scheme_matters()      { assert_ne!(derive_identity("https://example.com"), derive_identity("http://example.com")); }
    #[test] fn identity_subdomain_matters()   { assert_ne!(derive_identity("https://example.com"), derive_identity("https://www.example.com")); }
    #[test] fn identity_canonical_collapse() {
        let base = derive_identity(&canonicalise_url("https://google.com"));
        for v in &["https://google.com/", "HTTPS://GOOGLE.COM", "https://google.com:443", "https://google.com/#x"] {
            assert_eq!(derive_identity(&canonicalise_url(v)), base, "variant '{v}'");
        }
    }

    // ── CGUID ───────────────────────────────────────────────────────────────

    #[test] fn encode_decode_roundtrip() {
        for cguid in 1u8..=9 {
            let (cg1, cg2) = encode_cguid(cguid);
            assert_eq!(decode_cguid(cg1, cg2).unwrap(), cguid);
            assert!(cg1 >= 1 && cg1 <= 3);
            assert!(cg2 >= 1 && cg2 <= 3);
        }
    }
    #[test] fn default_is_1_1()    { assert_eq!(encode_cguid(1), (1, 1)); }
    #[test] fn max_is_3_3()        { assert_eq!(encode_cguid(9), (3, 3)); }
    #[test] fn zero_trit_forgery() {
        assert!(decode_cguid(0, 1).is_err());
        assert!(decode_cguid(1, 0).is_err());
    }
    #[test] fn all_9_unique() {
        let encoded: Vec<(u8,u8)> = (1..=9).map(encode_cguid).collect();
        let unique: std::collections::HashSet<_> = encoded.iter().collect();
        assert_eq!(unique.len(), 9);
    }
    #[test] fn effective_space() {
        let base: u64 = (0..27).fold(1u64, |a, _| a.saturating_mul(3));
        assert_eq!(base * 9, 68_630_377_364_883);
    }
}
