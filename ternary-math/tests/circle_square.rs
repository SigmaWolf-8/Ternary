// ════════════════════════════════════════════════════════════════════════
// Circle-and-Square Bijection — Acceptance tests for Spec v3.3.33.
//
// Coverage:
//   - The nine §6 worked examples (empty + 8 numeric).
//   - The four representation projections (A, B, C, D).
//   - The repunit ladder anchor (§1, R_1..R_6).
//   - The 27-position Milesian register (ghost positions and glyphs).
//   - The cumulative-delta identity 4995 / 3999 / 3699 = 27 × 137,
//     computed via the closed-form G(p) (§1, §4.5).
// ════════════════════════════════════════════════════════════════════════

use ternary_math::repx::{
    encode_bytes_to_trits_and_glyphs, ByteString, Representation, TritString,
    MilesianGlyphString,
};
use ternary_math::constants::{
    T_MILESIAN_REGISTER, T_GHOST_POSITIONS,
};
use ternary_math::trit_int::TritInt;

// ── Helpers ─────────────────────────────────────────────────────────────

/// Convenience: encode and return Rep-C view + glyph string.
fn encode_c(host_bytes: &[u8], k: usize) -> (Vec<u8>, String) {
    let bs = ByteString::from_host_bytes(host_bytes);
    let (trits, glyphs) = encode_bytes_to_trits_and_glyphs(&bs, Representation::C, k);
    (trits.as_rep_c(), glyphs.as_string())
}

/// §1 closed form for the Milesian numeral value at register position p.
fn g_of_p(p: u32) -> u32 {
    match p {
        1..=9   => p,
        10..=18 => 10 * (p - 9),
        19..=27 => 100 * (p - 18),
        _       => panic!("g_of_p: p out of range 1..=27 (got {})", p),
    }
}

// ── §6 Example 1 — empty input ──────────────────────────────────────────

#[test]
fn example_1_empty_input() {
    let bs = ByteString::empty();
    let (trits, glyphs) =
        encode_bytes_to_trits_and_glyphs(&bs, Representation::C, 1);
    assert!(trits.is_empty(), "empty input must give empty trit string");
    assert!(glyphs.is_empty(), "empty input must give empty glyph string");
    assert_eq!(trits.block_size_k(), 1);
    assert_eq!(trits.as_rep_c(), Vec::<u8>::new());
    assert_eq!(glyphs.as_string(), "");
}

// ── §6 Example 2 — byte 0 (N=1), k=1 ────────────────────────────────────

#[test]
fn example_2_byte_zero_k1() {
    let (trits_c, glyphs) = encode_c(&[0], 1);
    assert_eq!(trits_c, vec![1u8], "Rep-C trits for N=1, k=1");
    assert_eq!(glyphs, "α", "glyph for N=1");

    // Representation shifts spelled out by the spec: Rep-B [0], Rep-A [-1].
    let bs = ByteString::from_host_bytes(&[0]);
    let (trits, _) =
        encode_bytes_to_trits_and_glyphs(&bs, Representation::B, 1);
    assert_eq!(trits.as_rep_b(), vec![0u8]);
    let (trits, _) =
        encode_bytes_to_trits_and_glyphs(&bs, Representation::A, 1);
    assert_eq!(trits.as_rep_a(), vec![-1i8]);
}

// ── §6 Example 3 — byte 0 (N=1), k=5 ────────────────────────────────────

#[test]
fn example_3_byte_zero_k5() {
    let (trits_c, glyphs) = encode_c(&[0], 5);
    assert_eq!(trits_c, vec![1u8, 1, 1, 1, 1], "Rep-C trits for N=1, k=5");
    assert_eq!(glyphs, "α", "glyph independent of k");
}

// ── §6 Example 4 — byte 3 (N=4), k=1 ────────────────────────────────────

#[test]
fn example_4_n4_k1() {
    let (trits_c, glyphs) = encode_c(&[3], 1);
    assert_eq!(trits_c, vec![1u8, 1], "Rep-C trits for N=4, k=1");
    assert_eq!(glyphs, "δ", "glyph for N=4");
}

// ── §6 Example 5 — byte 3 (N=4), k=5 ────────────────────────────────────

#[test]
fn example_5_n4_k5() {
    let (trits_c, glyphs) = encode_c(&[3], 5);
    assert_eq!(trits_c, vec![1u8, 1, 1, 2, 1], "Rep-C trits for N=4, k=5");
    assert_eq!(glyphs, "δ", "glyph stable across k");
}

// ── §6 Example 6 — bytes [0, 0] (N=257), k=1 ────────────────────────────

#[test]
fn example_6_n257_k1() {
    let (trits_c, glyphs) = encode_c(&[0, 0], 1);
    assert_eq!(trits_c, vec![2u8, 3, 1, 1, 2], "Rep-C trits for N=257, k=1");
    assert_eq!(glyphs, "θν", "glyph for N=257");
}

// ── §6 Example 7 — bytes [0, 0], k=5 ────────────────────────────────────

#[test]
fn example_7_n257_k5() {
    let (trits_c, glyphs) = encode_c(&[0, 0], 5);
    assert_eq!(
        trits_c,
        vec![1u8, 1, 1, 1, 1, 1, 1, 2, 2, 2],
        "Rep-C trits for N=257, k=5"
    );
    assert_eq!(glyphs, "θν", "glyph identical across k");
}

// ── §6 Example 8 — bytes [0, 0], k=2 ────────────────────────────────────

#[test]
fn example_8_n257_k2() {
    let (trits_c, glyphs) = encode_c(&[0, 0], 2);
    assert_eq!(
        trits_c,
        vec![1u8, 3, 1, 1, 2, 2],
        "Rep-C trits for N=257, k=2"
    );
    assert_eq!(glyphs, "θν", "glyph universal");
}

// ── §1 — All four representation projections agree on the same input ───

#[test]
fn four_representation_projections() {
    // Use Example 6 (N=257, k=1) — Rep-C [2,3,1,1,2].
    let bs = ByteString::from_host_bytes(&[0, 0]);

    let (ts_c, g_c) =
        encode_bytes_to_trits_and_glyphs(&bs, Representation::C, 1);
    let (ts_b, g_b) =
        encode_bytes_to_trits_and_glyphs(&bs, Representation::B, 1);
    let (ts_a, g_a) =
        encode_bytes_to_trits_and_glyphs(&bs, Representation::A, 1);
    let (ts_d, g_d) =
        encode_bytes_to_trits_and_glyphs(&bs, Representation::D, 1);

    // Glyphs are the eternal Milesian name — same in all four reps.
    assert_eq!(g_c.as_string(), "θν");
    assert_eq!(g_b.as_string(), "θν");
    assert_eq!(g_a.as_string(), "θν");
    assert_eq!(g_d.as_string(), "θν");

    // Rep-C: bijective {1, 2, 3}.
    assert_eq!(ts_c.as_rep_c(), vec![2u8, 3, 1, 1, 2]);
    // Rep-B: {0, 1, 2} = Rep-C - 1.
    assert_eq!(ts_b.as_rep_b(), vec![1u8, 2, 0, 0, 1]);
    // Rep-A: {-1, 0, +1} = Rep-C - 2.
    assert_eq!(ts_a.as_rep_a(), vec![0i8, 1, -1, -1, 0]);
    // Rep-D: identity-on-{1,2,3} per the encoder wrapper (Spec §1).
    assert_eq!(ts_d.project(Representation::D), vec![2i8, 3, 1, 1, 2]);
}

// ── §1 — Repunit ladder anchor: R_1=1, R_2=4, R_3=13, R_4=40,
//                               R_5=121, R_6=364. ──────────────────────

#[test]
fn repunit_ladder_anchor() {
    let expected: [u64; 6] = [1, 4, 13, 40, 121, 364];
    for (i, &want) in expected.iter().enumerate() {
        let n = i + 1;
        let r = TritInt::repunit(n);
        assert_eq!(
            r.host_u64(),
            want,
            "R_{} (base 3) should equal {}",
            n, want
        );
    }
}

// ── §1, §4.5 — 27-position Milesian register, including the three ghosts.

#[test]
fn milesian_register_has_27_positions() {
    assert_eq!(T_MILESIAN_REGISTER.len(), 27, "register length must be 27");
    // Positions monotonic 1..=27.
    for (i, (p, _g)) in T_MILESIAN_REGISTER.iter().enumerate() {
        assert_eq!(*p as usize, i + 1, "register position must equal index+1");
    }
}

#[test]
fn milesian_ghost_positions_and_glyphs() {
    // Spec §1 declares ghost positions 6 (digamma ϛ), 18 (koppa ϙ), 27 (sampi ϡ).
    assert_eq!(T_GHOST_POSITIONS, [6u32, 18, 27]);
    let (p6,  g6)  = T_MILESIAN_REGISTER[5];
    let (p18, g18) = T_MILESIAN_REGISTER[17];
    let (p27, g27) = T_MILESIAN_REGISTER[26];
    assert_eq!((p6,  g6),  (6,  'ϛ'),  "digamma at register position 6");
    assert_eq!((p18, g18), (18, 'ϙ'),  "koppa at register position 18");
    assert_eq!((p27, g27), (27, 'ϡ'),  "sampi at register position 27");
}

// ── §1, §4.5 — Cumulative-delta identity (4995 / 3999 / 3699 = 27·137).
//
// Computation per the closed-form G(p):
//   p ∈ 1..=9  → G(p) = p           (no delta against position)
//   p ∈ 10..=18 → G(p) = 10·(p−9)
//   p ∈ 19..=27 → G(p) = 100·(p−18)
//
// • 4995 = Σ_{p=1..27} G(p)                    — full 27-symbol sum
// • 3999 = Σ over the 24 non-ghost positions    — ghost subtraction = 996
// • 3699 = Σ_{i=1..24} (G(p_i) − i)             — cumulative ghost delta
//        = 27 × 137                             — fine-structure anchor
// ───────────────────────────────────────────────────────────────────────

#[test]
fn cumulative_delta_chain_4995_3999_3699() {
    let ghosts: [u32; 3] = T_GHOST_POSITIONS;

    // 4995 — full sum.
    let total: u32 = (1..=27u32).map(g_of_p).sum();
    assert_eq!(total, 4995, "Σ G(p) for p=1..=27 must equal 4995");

    // 3999 — non-ghost sum.
    let ghost_sum: u32 = ghosts.iter().copied().map(g_of_p).sum();
    assert_eq!(ghost_sum, 996, "ghost-glyph values sum to 996");
    let non_ghost_sum = total - ghost_sum;
    assert_eq!(
        non_ghost_sum, 3999,
        "Σ G(p) over the 24 non-ghost positions must equal 3999"
    );

    // 3699 = Σ Δ_i over i = 1..24, where Δ_i = G(p_i) − i and p_i is the
    //         i-th non-ghost original position.
    let mut delta_sum: i64 = 0;
    let mut compressed_index: u32 = 0;
    for p in 1..=27u32 {
        if ghosts.contains(&p) { continue; }
        compressed_index += 1;
        delta_sum += g_of_p(p) as i64 - compressed_index as i64;
    }
    assert_eq!(compressed_index, 24, "non-ghost letters count must be 24");
    assert_eq!(delta_sum, 3699,
        "cumulative delta Σ_(i=1..24) (G(p_i) − i) must equal 3699");

    // 3999 − 300 = 3699.
    assert_eq!(non_ghost_sum as i64 - 300, delta_sum,
        "delta chain 3999 − 300 = 3699");

    // 27 × 137 = 3699.
    assert_eq!(27 * 137, 3699u32, "fine-structure anchor 27·137 = 3699");
    assert_eq!(delta_sum as u32, 27 * 137,
        "delta sum equals the fine-structure anchor");
}

// ── §1 — Verify TritString carries its block size k correctly. ─────────

#[test]
fn trit_string_carries_k() {
    let bs = ByteString::from_host_bytes(&[0, 0]);
    for k in [1usize, 2, 3, 5, 7] {
        let (trits, _) =
            encode_bytes_to_trits_and_glyphs(&bs, Representation::C, k);
        assert_eq!(trits.block_size_k(), k);
        // Length must always be a multiple of k (Spec §4.4).
        assert!(trits.len() % k == 0,
            "trit length {} not multiple of k={}", trits.len(), k);
    }
}

// ── §4.5 — Glyph universality: same input, different k → same glyphs. ──

#[test]
fn glyph_string_universal_across_k() {
    let bs = ByteString::from_host_bytes(&[0, 0]);
    let mut prior: Option<MilesianGlyphString> = None;
    for k in [1usize, 2, 3, 5, 7, 11] {
        let (_trits, glyphs) =
            encode_bytes_to_trits_and_glyphs(&bs, Representation::C, k);
        if let Some(p) = &prior {
            assert_eq!(p.as_string(), glyphs.as_string(),
                "glyph string must be identical across k (k={})", k);
        }
        prior = Some(glyphs);
    }
    assert_eq!(prior.unwrap().as_string(), "θν");
}

// ── §3.1 + §4.4 — Round-anchor: trit length is exactly k·D. ────────────

#[test]
fn trit_length_equals_k_times_d() {
    // Example 6: N=257, k=1, D=5 → 5 trits.
    let (t, _) = encode_c(&[0, 0], 1);
    assert_eq!(t.len(), 5);
    // Example 7: k=5, D=2 → 10 trits.
    let (t, _) = encode_c(&[0, 0], 5);
    assert_eq!(t.len(), 10);
    // Example 8: k=2, D=3 → 6 trits.
    let (t, _) = encode_c(&[0, 0], 2);
    assert_eq!(t.len(), 6);
}

// ── Smoke: encoder is total over the byte alphabet for k=1. ────────────

#[test]
fn smoke_single_byte_alphabet_k1() {
    for b in 0u8..=255 {
        let bs = ByteString::from_host_bytes(&[b]);
        let (trits, glyphs) =
            encode_bytes_to_trits_and_glyphs(&bs, Representation::C, 1);
        // Trit string length must be a multiple of k=1 (trivially true) and
        // every Rep-C value must be in {1, 2, 3}.
        let _ = TritString::from_trits(trits.trits().to_vec(), 1);
        for c in trits.as_rep_c() {
            assert!((1..=3).contains(&c), "Rep-C trit out of range for byte {}", b);
        }
        // Glyph string must be non-empty for any single byte (N ≥ 1).
        assert!(!glyphs.is_empty(), "non-empty glyph string for byte {}", b);
    }
}
