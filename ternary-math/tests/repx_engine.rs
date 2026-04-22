// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// task-133 — RepX engine integration test (grid invariants).

use ternary_math::repx::{
    at, registers_for_archetype, registers_for_element, AlgebraicTrit, Archetype, ComposeOp,
    Element, Engine, RepXTransferFunction, REGISTERS,
};

#[test]
fn grid_has_39_cells() {
    assert_eq!(REGISTERS.len(), 39);
}

#[test]
fn each_archetype_has_13_rows() {
    for archetype in [Archetype::Sun, Archetype::Gaia, Archetype::Moon] {
        assert_eq!(registers_for_archetype(archetype).count(), 13);
    }
    for row in 1u8..=13 {
        for archetype in [Archetype::Sun, Archetype::Gaia, Archetype::Moon] {
            assert!(at(archetype, row).is_some(), "{:?} #{} missing", archetype, row);
        }
    }
    assert!(at(Archetype::Sun, 0).is_none());
    assert!(at(Archetype::Sun, 14).is_none());
}

#[test]
fn pentadic_membership_matches_quintessence_canon() {
    // Spec §F3 cross-check (line 215-223): Fire 8, Water 7, Air 6,
    // Earth 7, Aether 8, Unmapped 3. Sum = 39.
    assert_eq!(registers_for_element(Element::Fire).count(), 8);
    assert_eq!(registers_for_element(Element::Water).count(), 7);
    assert_eq!(registers_for_element(Element::Air).count(), 6);
    assert_eq!(registers_for_element(Element::Earth).count(), 7);
    assert_eq!(registers_for_element(Element::Aether).count(), 8);
    let unmapped = REGISTERS.iter().filter(|r| r.elements.is_empty()).count();
    assert_eq!(unmapped, 3);
    assert_eq!(8 + 7 + 6 + 7 + 8 + unmapped, 39);
}

#[test]
fn fire_set_matches_canonical_sun_rows() {
    // F4 Fire ← Sun {1, 2, 3, 4, 5, 6, 7, 13}.
    let mut rows: Vec<u8> = registers_for_element(Element::Fire)
        .filter(|r| r.archetype == Archetype::Sun)
        .map(|r| r.row)
        .collect();
    rows.sort();
    assert_eq!(rows, vec![1, 2, 3, 4, 5, 6, 7, 13]);
}

#[test]
fn water_set_matches_quintessence_reconciliation() {
    // F4 reconciliation: Water = Gaia {2, 4, 6, 10, 11} ∪ Moon {8, 9}.
    // Moon #8 (drag) and Moon #9 (diffusion) MUST be Water, not Aether.
    let mut gaia: Vec<u8> = registers_for_element(Element::Water)
        .filter(|r| r.archetype == Archetype::Gaia)
        .map(|r| r.row)
        .collect();
    gaia.sort();
    assert_eq!(gaia, vec![2, 4, 6, 10, 11]);
    let mut moon: Vec<u8> = registers_for_element(Element::Water)
        .filter(|r| r.archetype == Archetype::Moon)
        .map(|r| r.row)
        .collect();
    moon.sort();
    assert_eq!(moon, vec![8, 9]);
}

#[test]
fn unmapped_set_is_exactly_sun_8_9_and_gaia_5() {
    let mut unmapped: Vec<(Archetype, u8)> = REGISTERS
        .iter()
        .filter(|r| r.elements.is_empty())
        .map(|r| (r.archetype, r.row))
        .collect();
    unmapped.sort_by_key(|(a, r)| (format!("{:?}", a), *r));
    assert_eq!(
        unmapped,
        vec![(Archetype::Gaia, 5), (Archetype::Sun, 8), (Archetype::Sun, 9)]
    );
}

#[test]
fn gaia_seven_carries_ot_1f_horn_radius_note() {
    let cell = at(Archetype::Gaia, 7).unwrap();
    assert!(
        cell.ot_notes.iter().any(|n| n.contains("OT-1f")),
        "Gaia #7 must mention OT-1f horn-radius reinterpretation"
    );
}

#[test]
fn sun_one_uses_horn_volume_form() {
    let cell = at(Archetype::Sun, 1).unwrap();
    assert!(cell.formula_doc.contains("ROOT_X1"));
    assert!(!cell.formula_doc.contains("4/3"));
}

#[test]
fn moon_eight_is_stokes_drag_water() {
    // Spec line 208: Moon #8 must be F_d = (6·ROOT_X1)·η·r·v, Water.
    let cell = at(Archetype::Moon, 8).unwrap();
    assert!(cell.formula_doc.contains("F_d") && cell.formula_doc.contains("6·ROOT_X1"));
    assert_eq!(cell.elements, &[Element::Water]);
}

#[test]
fn moon_thirteen_is_tidal_aether() {
    // Spec line 213: Moon #13 ΔF tidal pull, Aether.
    let cell = at(Archetype::Moon, 13).unwrap();
    assert!(cell.formula_doc.contains("ΔF"));
    assert_eq!(cell.elements, &[Element::Aether]);
}

#[test]
fn no_bare_pi_literals_in_formula_strings() {
    // Spec line 232: every formula uses ROOT_X1 / 2·ROOT_X1 / 4·ROOT_X1
    // / 6·ROOT_X1 — never bare 14 / 28 / 56 / 84.
    for r in REGISTERS.iter() {
        for bad in ["14", "28", "56", "84"] {
            // Skip allowed appearances (none of the canonical formulas
            // contain these as bare integers).
            assert!(
                !r.formula_doc.contains(bad),
                "{:?} #{} formula contains bare literal {}: {}",
                r.archetype, r.row, bad, r.formula_doc
            );
        }
    }
}

#[test]
fn read_si_is_total_over_all_39_cells() {
    // Spec G3 acceptance: every register evaluates to a finite,
    // non-NaN reading at the engine's reference displacement x = 1 m.
    let eng = Engine::new();
    for r in REGISTERS.iter() {
        let v = eng.read_si(r, 1.0);
        assert!(
            v.is_some_and(|x| x.is_finite()),
            "{:?} #{} did not evaluate at x=1m: got {:?}",
            r.archetype, r.row, v
        );
    }
}

#[test]
fn interchange_matrix_1521_pairs_total_or_explicit_noninvertible() {
    // Spec G3: the full 39×39 interchange matrix must produce either
    // a finite reading (invertible source path) or a clean None
    // (state-aug source with no x-coupling — explicit non-invertibility).
    // No panics, no NaN, no infinities allowed for any of the 1521 pairs.
    let eng = Engine::new();
    let x = 2.0;
    let mut pairs = 0usize;
    for src in REGISTERS.iter() {
        let value = eng.read_si(src, x).expect("read_si total");
        for tgt in REGISTERS.iter() {
            pairs += 1;
            let v = eng
                .convert_si(src, value, tgt)
                .unwrap_or_else(|| panic!(
                    "convert_si MUST be total over 39×39: {:?}#{} → {:?}#{} returned None",
                    src.archetype, src.row, tgt.archetype, tgt.row
                ));
            assert!(
                v.is_finite(),
                "{:?}#{} → {:?}#{} produced non-finite {}",
                src.archetype, src.row, tgt.archetype, tgt.row, v
            );
        }
    }
    assert_eq!(pairs, 39 * 39, "must exercise full 1521-pair matrix");
}

#[test]
fn rotate_is_total_over_all_39x39_pairs() {
    // Spec §D.2: rotate must be total (never panic) over every
    // (source register, target register) pair for every input character.
    let eng = Engine::new();
    let chars = [AlgebraicTrit::Zero, AlgebraicTrit::One, AlgebraicTrit::Omega];
    for src in REGISTERS.iter() {
        for tgt in REGISTERS.iter() {
            for &ch in &chars {
                // Must not panic; output is a valid AlgebraicTrit.
                let _ = eng.rotate((src, ch), tgt);
            }
        }
    }
}

#[test]
fn at_is_formula_aware_not_archetype_only() {
    // Reviewer-required: `at` must depend on the cell, not just the
    // archetype. Specifically Sun #4 (q = −k·dT/dx, negative leading
    // coefficient) MUST differ from Sun #2 (A = π_fw/x², positive).
    let eng = Engine::new();
    let sun_2 = at(Archetype::Sun, 2).unwrap();
    let sun_4 = at(Archetype::Sun, 4).unwrap();
    let p = AlgebraicTrit::One;
    assert_ne!(
        RepXTransferFunction::at(&eng, p, sun_2),
        RepXTransferFunction::at(&eng, p, sun_4),
        "at(point, R) must differ between Sun #2 and Sun #4 — formula-aware"
    );
    // Unmapped Sun #8 (no pentadic tag) projects to Zero regardless of point.
    let sun_8 = at(Archetype::Sun, 8).unwrap();
    assert_eq!(
        RepXTransferFunction::at(&eng, AlgebraicTrit::One, sun_8),
        AlgebraicTrit::Zero,
        "unmapped Sun #8 must Yoneda-project to Zero"
    );
}

#[test]
fn rotate_sun_to_moon_via_gaia_picks_up_omega() {
    // Sun → Moon is the canonical via-Gaia transit. Starting from
    // (Sun #3, point=One) with Sun #3 character One, target Moon #5
    // character One, the archetype rotor for any cross-archetype move
    // into Gaia or Moon is ω, so result = One · ω · One = ω.
    let eng = Engine::new();
    let result = eng.rotate(
        (at(Archetype::Sun, 3).unwrap(), AlgebraicTrit::One),
        at(Archetype::Moon, 5).unwrap(),
    );
    assert_eq!(result, AlgebraicTrit::Omega);
}

#[test]
fn compose_chain_multiply_associative_in_eisenstein() {
    // Reviewer-required: compose runs in real ℤ[ω] arithmetic.
    // ω · ω = ω² ≡ One in single-trit GF(3) projection.
    let eng = Engine::new();
    let r = at(Archetype::Sun, 1).unwrap();
    let result = eng.compose(
        &[(r, AlgebraicTrit::Omega), (r, AlgebraicTrit::Omega)],
        ComposeOp::ChainMultiply,
    );
    assert_eq!(result, AlgebraicTrit::One);
}

#[test]
fn xinv_cells_roundtrip_read_invert_read() {
    // Reviewer-required: every XInv (invertible) cell must satisfy
    // read_si(R, x) → invert_si(R, value) → read_si(R, x') with
    // x' ≈ x to high precision. Catches forward/inverse formula drift.
    let eng = Engine::new();
    let xs = [0.5_f64, 1.0, 2.0, 7.0, 100.0];
    for r in REGISTERS.iter() {
        for &x in &xs {
            let v = match eng.read_si(r, x) {
                Some(v) => v,
                None => continue,
            };
            let x_back = match eng.invert_si(r, v) {
                Some(x) => x,
                None => continue, // state-aug cell — invert returns None by design
            };
            let v_back = eng
                .read_si(r, x_back)
                .expect("re-read after invert must succeed");
            let denom = v.abs().max(1e-12);
            let rel = (v_back - v).abs() / denom;
            assert!(
                rel < 1e-6,
                "{:?} #{} roundtrip drift at x={}: v={:e} → x'={:e} → v'={:e} (rel={:e})",
                r.archetype, r.row, x, v, x_back, v_back, rel
            );
        }
    }
}

#[test]
fn moon_13_roundtrip_inverse_aligned_with_forward() {
    // Targeted regression: Moon #13 canonical tidal form is
    // ΔF = (dΓ/dx)·Δx·ρ·V with Γ = Γ₀/x², so dΓ/dx = -2·Γ₀/x³.
    // With ρ = ρ₀/x (Gaia #1), Δx = lever_arm, V = volume_default
    // this collapses to ΔF = -2·Γ₀·Δx·ρ₀·V / x⁴ — strictly negative,
    // and must roundtrip exactly through the matching inverse.
    let eng = Engine::new();
    let r = at(Archetype::Moon, 13).unwrap();
    for &x in &[0.5_f64, 1.0, 3.0, 17.0] {
        let v = eng.read_si(r, x).unwrap();
        assert!(v < 0.0, "Moon #13 tidal force must be negative; got {v}");
        let x_back = eng.invert_si(r, v).unwrap();
        assert!((x_back - x).abs() / x < 1e-9, "Moon #13 inverse mismatch at x={x}: got {x_back}");
    }
}

#[test]
fn gaia_7_uses_density_difference_per_ot1f() {
    // Reviewer-required: Gaia #7 must use (ρ_med − ρ_obj)·V·Γ. With
    // ρ_obj overridden, the reading must scale linearly with the
    // density difference.
    use ternary_math::repx::{Body, Calibration};
    let mut cal = Calibration::defaults();
    cal.rho_obj = 0.0;
    let eng_zero = Engine::with_calibration(Body::Sun, cal);
    let v0 = eng_zero.read_si(at(Archetype::Gaia, 7).unwrap(), 1.0).unwrap();

    // Pick ρ_obj = ρ_med/2 at x=1: ρ_med = RHO_0/x = RHO_0 (= 1.15e-2).
    cal.rho_obj = 1.15e-2 / 2.0;
    let eng_half = Engine::with_calibration(Body::Sun, cal);
    let v_half = eng_half.read_si(at(Archetype::Gaia, 7).unwrap(), 1.0).unwrap();

    let ratio = v_half / v0;
    assert!(
        (ratio - 0.5).abs() < 1e-9,
        "Gaia #7 must scale linearly with (ρ_med − ρ_obj); ratio = {ratio}"
    );
}

#[test]
fn repxtransform_engine_has_body_dependent_character() {
    use ternary_math::repx::{Body, RepXTransform};
    let sun_eng = Engine::new();
    let earth_eng = Engine::with_body(Body::Earth);
    assert_eq!(sun_eng.as_coefficient(), AlgebraicTrit::One);
    assert_eq!(earth_eng.as_coefficient(), AlgebraicTrit::Omega);
    assert_ne!(sun_eng.as_coefficient(), earth_eng.as_coefficient());
}

#[test]
fn repxtransform_parseval_energy_counts_nonzero_primitives() {
    use ternary_math::repx::RepXTransform;
    // Sun engine: triple (1, ω, ω) ⇒ N = 1 + 1 + 1 = 3.
    let sun_eng = Engine::new();
    assert_eq!(sun_eng.parseval_energy(), 3);
}

#[test]
fn nona_closure_evaluates_canonical_triadic_identity() {
    // Reviewer-required: the closure must encode 1·c₀ + ω·c₁ + ω²·c₂
    // = 0 in ℤ[ω] (single-trit GF(3) projection where ω ≡ 2 mod 3,
    // ω² = ω·ω = 4 ≡ 1 mod 3). With c₀=1, c₁=2, c₂=2 (Sun engine):
    //   1·1 + 2·2 + 1·2 = 1 + 4 + 2 = 7 ≡ 1 mod 3 ≠ 0 → false.
    // With c₀=2 (Earth engine):
    //   1·2 + 2·2 + 1·2 = 2 + 4 + 2 = 8 ≡ 2 mod 3 ≠ 0 → false.
    use ternary_math::repx::{Body, RepXTransform};
    let sun_eng = Engine::new();
    let earth_eng = Engine::with_body(Body::Earth);
    assert!(!sun_eng.nona_closure(), "Sun-engine triple must NOT close");
    assert!(!earth_eng.nona_closure(), "Earth-engine triple must NOT close");
}

#[test]
fn deprecated_alias_still_resolves() {
    // task-133 G2: legacy callers compile via the deprecated re-export.
    #[allow(deprecated)]
    use ternary_math::gf3_algebra::AlgebraicTrit;
    let _ = AlgebraicTrit::Zero;
}
