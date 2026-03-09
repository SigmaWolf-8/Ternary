// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// PlenumNET Rust Performance Benchmark
// Tests actual production code (tis_sponge.rs, gf3_algebra.rs) against
// industry-standard Rust crates (sha2, sha3, blake2, aes-gcm, hkdf).

mod tis_sponge;
mod gf3_algebra;

use std::time::Instant;
use std::hint::black_box;

use sha2::{Sha256, Sha384, Sha512, Digest as Sha2Digest};
use sha3::Sha3_256;
use blake2::{Blake2b512, Digest as Blake2Digest};
use hmac::{Hmac, Mac};
use hkdf::Hkdf;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};

// ── Benchmark harness ──────────────────────────────────────────────

struct BenchResult {
    category: &'static str,
    name: &'static str,
    ns_per_op: f64,
    ops_per_sec: f64,
    is_plenum: bool,
}

fn bench<F: FnMut()>(iters: u64, mut f: F) -> f64 {
    // Warmup
    for _ in 0..iters / 10 { f(); }
    // Measure
    let start = Instant::now();
    for _ in 0..iters { f(); }
    let elapsed = start.elapsed().as_nanos() as f64;
    elapsed / iters as f64
}

macro_rules! run_bench {
    ($results:expr, $cat:expr, $name:expr, $iters:expr, $is_pn:expr, $code:expr) => {{
        let ns = bench($iters, || { black_box($code); });
        let ops = 1e9 / ns;
        println!("  {:44} {:8.1} ns {:>10.0}/s  {}",
            $name, ns, ops, if $is_pn { "◀ PLENUM" } else { "  industry" });
        $results.push(BenchResult {
            category: $cat, name: $name, ns_per_op: ns, ops_per_sec: ops, is_plenum: $is_pn,
        });
    }};
}

fn main() {
    let mut results: Vec<BenchResult> = Vec::new();

    // Test data
    let raw27: Vec<u8> = (0..27).map(|i| ((i * 7 + 3) & 0xFF) as u8).collect();
    let raw81: Vec<u8> = (0..81).map(|i| ((i * 7 + 3) & 0xFF) as u8).collect();
    let input_gf3_27: Vec<u8> = (0..27).map(|i| (i % 3) as u8).collect();
    let input_gf3_81: Vec<u8> = (0..81).map(|i| (i % 3) as u8).collect();
    let addr_a: Vec<u8> = (0..27).map(|i| (i % 3) as u8).collect();
    let addr_b: Vec<u8> = (0..27).map(|i| ((i * 7) % 3) as u8).collect();
    let valid_rep_c: Vec<u8> = (0..27).map(|i| (i % 3 + 1) as u8).collect();
    let key32: [u8; 32] = [42u8; 32];
    let nonce12: [u8; 12] = [0u8; 12];

    let n: u64 = 2_000_000;
    let nm: u64 = 500_000;

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  PLENUMNET RUST BENCHMARK                                                ║");
    println!("║  Production code (tis_sponge.rs, gf3_algebra.rs) vs industry crates      ║");
    println!("║  Capomastro Holdings Ltd. — Applied Physics Division                     ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

    // ═══════════════════════ 1. HASHING ═══════════════════════
    println!("── 1. HASHING ──────────────────────────────────────────────────────────────");

    run_bench!(results, "Hash", "TIS-27 (4r, 7-neighbor extended theta)", n, true, {
        tis_sponge::tis27_hash(&input_gf3_27, 27)
    });

    run_bench!(results, "Hash", "TIS-81 (4r, 7-neighbor, post-quantum)", nm, true, {
        tis_sponge::tis81_hash(&input_gf3_81, 81)
    });

    run_bench!(results, "Hash", "SHA-256 (sha2 crate, 27B)", n, false, {
        let mut h = Sha256::new();
        h.update(&raw27);
        h.finalize()
    });

    run_bench!(results, "Hash", "SHA-384 (sha2 crate, 27B)", n, false, {
        let mut h = Sha384::new();
        h.update(&raw27);
        h.finalize()
    });

    run_bench!(results, "Hash", "SHA-512 (sha2 crate, 27B)", n, false, {
        let mut h = Sha512::new();
        h.update(&raw27);
        h.finalize()
    });

    run_bench!(results, "Hash", "SHA3-256 (sha3 crate, 27B)", n, false, {
        let mut h = Sha3_256::new();
        h.update(&raw27);
        h.finalize()
    });

    run_bench!(results, "Hash", "SHA3-256 (sha3 crate, 81B)", nm, false, {
        let mut h = Sha3_256::new();
        h.update(&raw81);
        h.finalize()
    });

    run_bench!(results, "Hash", "BLAKE2b-512 (blake2 crate, 27B)", n, false, {
        let mut h = Blake2b512::new();
        h.update(&raw27);
        h.finalize()
    });

    // ═══════════════════════ 2. ADDRESS DERIVATION ═══════════════════════
    println!("\n── 2. ADDRESS DERIVATION ────────────────────────────────────────────────────");

    run_bench!(results, "Address", "TIS-27 → Rep C (native, one step)", n, true, {
        let gf3 = tis_sponge::tis27_hash(&input_gf3_27, 27);
        let addr: Vec<u8> = gf3.iter().map(|&t| t + 1).collect();
        addr
    });

    run_bench!(results, "Address", "SHA-256 → Rep C (hash + convert)", n, false, {
        let mut h = Sha256::new();
        h.update(&raw27);
        let digest = h.finalize();
        let addr: Vec<u8> = digest.iter().take(27).map(|&b| (b % 3) + 1).collect();
        addr
    });

    // ═══════════════════════ 3. ROUTING ═══════════════════════
    println!("\n── 3. ROUTING ──────────────────────────────────────────────────────────────");

    run_bench!(results, "Routing", "Hamming GF(3) Σ(a-b)² mod 3, 27-trit", n, true, {
        gf3_algebra::hamming_distance(&addr_a, &addr_b)
    });

    run_bench!(results, "Routing", "CRT decompose (sector + slot)", n * 5, true, {
        let pos = 123456789u64;
        (pos % 13, pos % 28)
    });

    run_bench!(results, "Routing", "CRT reconstruct", n * 5, true, {
        (196u64 * 7 + 169u64 * 15) % 364
    });

    // ═══════════════════════ 4. INTEGRITY ═══════════════════════
    println!("\n── 4. INTEGRITY ────────────────────────────────────────────────────────────");

    run_bench!(results, "Integrity", "Forgery check (Π mod 7, 27 trits)", n, true, {
        gf3_algebra::has_forgery(&valid_rep_c)
    });

    run_bench!(results, "Integrity", "Repunit checksum (Horner mod 364)", n, true, {
        gf3_algebra::repunit_checksum(&valid_rep_c)
    });

    run_bench!(results, "Integrity", "HMAC-SHA256 (hmac crate, 27B)", nm, false, {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&key32).unwrap();
        Mac::update(&mut mac, &raw27);
        mac.finalize()
    });

    // ═══════════════════════ 5. ENCRYPTION ═══════════════════════
    println!("\n── 5. ENCRYPTION ───────────────────────────────────────────────────────────");

    let gf3_key: Vec<u8> = (0..27).map(|i| (i % 3) as u8).collect();
    let gf3_plain: Vec<u8> = (0..27).map(|i| (i % 3) as u8).collect();

    run_bench!(results, "Encrypt", "Phase encrypt GF(3), 27 trits", n, true, {
        let cipher: Vec<u8> = gf3_plain.iter().zip(gf3_key.iter())
            .map(|(&p, &k)| { let s = p + k; if s >= 3 { s - 3 } else { s } })
            .collect();
        cipher
    });

    run_bench!(results, "Encrypt", "AES-256-GCM encrypt 27B (aes-gcm crate)", nm, false, {
        let key = Key::<Aes256Gcm>::from_slice(&key32);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce12);
        cipher.encrypt(nonce, raw27.as_ref()).unwrap()
    });

    // ═══════════════════════ 6. KEY DERIVATION ═══════════════════════
    println!("\n── 6. KEY DERIVATION ───────────────────────────────────────────────────────");

    let kdf_ctx: Vec<u8> = (0..16).map(|i| (i % 3) as u8).collect();
    let kdf_mat: Vec<u8> = (0..11).map(|i| (i % 3) as u8).collect();

    run_bench!(results, "KDF", "TIS-27 KDF (context + material → 32B)", nm, true, {
        tis_sponge::tis27_derive_key(&kdf_ctx, &kdf_mat, 32)
    });

    run_bench!(results, "KDF", "HKDF-SHA256 (hkdf crate, 32B out)", nm, false, {
        let hk = Hkdf::<Sha256>::new(Some(&raw27[..16]), &key32);
        let mut out = [0u8; 32];
        hk.expand(b"ctx", &mut out).unwrap();
        out
    });

    // ═══════════════════════ 7. CAPABILITY TOKENS ═══════════════════════
    println!("\n── 7. CAPABILITY TOKENS ────────────────────────────────────────────────────");

    run_bench!(results, "Token", "Capability token (TIS-27 based)", n, true, {
        let mut buf = vec![0u8; 27];
        for i in 0..16 { buf[i] = input_gf3_27[i]; }
        buf[24] = 1; buf[25] = 2; buf[26] = 0;
        tis_sponge::tis27_hash(&buf, 27)
    });

    run_bench!(results, "Token", "HMAC-SHA256 token (JWT-style)", nm, false, {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&key32).unwrap();
        Mac::update(&mut mac, &raw27);
        mac.finalize()
    });

    // ═══════════════════════ 8. GF(3) PRIMITIVES ═══════════════════════
    println!("\n── 8. GF(3) ELEMENT OPERATIONS ─────────────────────────────────────────────");

    run_bench!(results, "GF3", "gf3_add (division-free)", n * 10, true, {
        gf3_algebra::gf3_add(1, 2)
    });

    run_bench!(results, "GF3", "gf3_mul (division-free)", n * 10, true, {
        gf3_algebra::gf3_mul(2, 2)
    });

    run_bench!(results, "GF3", "gf3_square (Hamming indicator)", n * 10, true, {
        gf3_algebra::gf3_square(2)
    });

    // ═══════════════════════ 9. VECTOR OPERATIONS ═══════════════════════
    println!("\n── 9. GF(3) VECTOR OPERATIONS (27-dim) ─────────────────────────────────────");

    let mut vec_out = vec![0u8; 27];

    run_bench!(results, "VecOps", "gf3_vec_add (27 elements)", n, true, {
        gf3_algebra::gf3_vec_add(&addr_a, &addr_b, &mut vec_out);
        vec_out[0]
    });

    run_bench!(results, "VecOps", "gf3_dot (27 elements)", n, true, {
        gf3_algebra::gf3_dot(&addr_a, &addr_b)
    });

    // ═══════════════════════ 10. FULL PIPELINES ═══════════════════════
    println!("\n── 10. FULL PIPELINES ──────────────────────────────────────────────────────");

    run_bench!(results, "Pipeline", "TDNS: raw → routable address (TIS-27)", n, true, {
        let gf3 = tis_sponge::tis27_hash(&input_gf3_27, 27);
        let addr: Vec<u8> = gf3.iter().map(|&t| t + 1).collect();
        addr
    });

    run_bench!(results, "Pipeline", "TDNS: raw → routable address (SHA-256)", n, false, {
        let mut h = Sha256::new();
        h.update(&raw27);
        let digest = h.finalize();
        let addr: Vec<u8> = digest.iter().take(27).map(|&b| (b % 3) + 1).collect();
        addr
    });

    run_bench!(results, "Pipeline", "Route: address → sector + slot + distance", n, true, {
        let ck = gf3_algebra::repunit_checksum(&valid_rep_c);
        let moon = ck % 13;
        let day = ck % 28;
        let dist = gf3_algebra::hamming_distance(&addr_a, &addr_b);
        (moon, day, dist)
    });

    // ═══════════════════════ SCORECARD ═══════════════════════
    println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║                    PLENUMNET RUST PERFORMANCE CARD                        ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════╣");

    let mut last_cat = "";
    for r in &results {
        if r.category != last_cat {
            println!("║  ── {:68} ║", r.category);
            last_cat = r.category;
        }
        let ops_str = if r.ops_per_sec > 1e6 {
            format!("{:.1}M", r.ops_per_sec / 1e6)
        } else {
            format!("{:.0}K", r.ops_per_sec / 1e3)
        };
        println!("║  {} {:44} {:7.0} ns {:>8}/s ║",
            if r.is_plenum { "▶" } else { "·" },
            r.name, r.ns_per_op, ops_str);
    }
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
}
