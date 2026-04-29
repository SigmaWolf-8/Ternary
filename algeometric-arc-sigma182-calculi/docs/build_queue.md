# AASC Build Queue — modules mapped on the canonical map but not yet in `src/`

This file is the **persistent reconciliation ledger** between
`client/public/aasc_canonical_map.svg` (the published canonical map)
and `algeometric-arc-sigma182-calculi/src/lib.rs` (the ground truth).

When a module name appears on the map but does not exist as a `.rs`
file, it is queued here with a pointer to the user-delivered
specification document(s) that define it.

The map is **not** trimmed to remove queued names. Names are removed
from this queue only when the corresponding module ships and is added
to `lib.rs`.

---

## Ground-truth corpus (canonical specs delivered by the user)

These are the named delivery documents. When the queue says a module
is "covered by [SOURCE]", it means the formula or definition lives in
one of these files.

- `attached_assets/Pasted-Here-is-the-full-corrected-document-with-the-missing-At_1777464567016.txt`
  — **Inertissimum Iώτα Nona — Codex Unificationis** (Compendium §I.A,
  Arc1.1.33.3, 2026-04-27, full synthesis with §10.1 Attosecond/HPTP)
- `attached_assets/Pasted--Inertissimum-I-Nona-Codex-Unificationis-Salvi-Arc-Fram_1777389577344.txt`
  — prior Inertissimum revision (still authoritative for §§unchanged)
- `attached_assets/Pasted--Task-130-Final-Specification-ParaCalculi-GeoPrimus-Lab_1776981041771.txt`
  — Task-130 Final Specification (ParaCalculi · GeoPrimus-Lab)
- `attached_assets/CM-260422_TM-2026-017_v11.21_1776875193176.md` — TM-2026-017 v11.21
- `attached_assets/CM-260422_Nona-State_v0.0.25_1776875193175.md` — Nona-State
- `attached_assets/CM-260422_Optimus-Paraprime-v1.4.9_1776875193175.md` — Optimus Paraprime
- `attached_assets/CM-260422_Rational-Invariants_v0.5_1776875193176.md` — Rational Invariants
- `attached_assets/CM-260422_Triadis-Orientation_v0.10_1776875193176.md` — Triadis Orientation
- `attached_assets/CM-260422-039_v2.2_Wieferich-Infinitude_1776930119310.md` — Wieferich Infinitude (already shipped)
- `attached_assets/CM-260422-039-JD_v1.0_The-Joint-Distribution-Lemma_1776940730614.md` — Joint Distribution Lemma
- `attached_assets/forma-codex-18-spec-final_1776877928690.md` — Forma Codex 18 spec final
- `attached_assets/forma-codex-classification-engine-v1.6_1776920472204.md` — Classification Engine
- `attached_assets/Pasted--PlenumLAN-Technical-Manifest-One-Language-One-Binary-F_1773844540809.txt` — PlenumLAN Technical Manifest
- `attached_assets/Pasted--I-A-The-Great-Circles-within-the-Compendium-Companion-_1777093149834.txt` — Great Circles companion
- `attached_assets/Pasted--The-Codex-of-the-Parabolic-Fulcrum-Complete-Compendium_1776664937960.txt` — Parabolic Fulcrum Codex

---

## Cluster A — Inertissimum §§0–4 (Walk · Torus · Vesica · Worldline · Frame Square · Discriminant · Earth)

All formulas closed-form in terms of `(p,q,r) = (7,11,13)`, `b = 3`,
`π_geom = 14`, `r = 13`, `R₂ = 4`, `R₄ = 40`, `R₆ = 364`.

| Module | SVG address | Spec source | Status |
|---|---|---|---|
| `pqr_asymmetry` | 7.5.1.UX4.1 (SE) | Inertissimum §0, §3.7 | **shipped** |
| `discriminant` | 5.2.1.UX5.1 (algebra) | Inertissimum §3.7, eq 391 | **shipped** |
| `discriminant_identity` | 9.8.1.UX4.1 (NW) | Inertissimum §3.7, eq 403 | **shipped** |
| `cone_point` | 5.H.1.UX5.2 (algebra) | Inertissimum §3.6, last paragraph | **shipped** |
| `mass_quadratic` | 8.5.1.UX4.1 (SW) | Inertissimum §3.7 (`x² − R₄·x + R₆`) | queued |
| `gabriels_horn` | (NW vessel) | Inertissimum §2 (`V_Horn = 14`, `S_Horn = ∞`) | queued |
| `vesica` | (algebra) | Inertissimum §3 (`ℓ_V² = r³/p`) | queued |
| `volumetric_ceiling` | (algebra) | Inertissimum §3 Theorem 2 (`V_torus = ℓ_V²·π³`) | queued |
| `grand_circle` | (geometry) | Inertissimum §2.1 (`C_Grand = 2π·b³ = 756`) | queued |
| `frame_square` | (algebra) | Inertissimum §3.6 (`(bq)² = 1089` + 4 partitions) | queued |
| `earth_circumference` | (geometry) | Inertissimum §4.1 (cone-point projection chain) | queued |
| `tropical_year` | (calendar) | Inertissimum §5.2 Theorem 4 (`τ = 1 096 822 / 3003`) | queued |
| `crt` | (algebra) | Inertissimum §3.6 (`Z₇₅₆ ≅ Z₂₇ × Z₂₈`) | queued |

## Cluster B — Inertissimum §§5–10 (Calendar · c · Phase Impedance · Spectral)

| Module | SVG address | Spec source | Status |
|---|---|---|---|
| `fine_structure` | 9.5.1.UX4.1 (NW) | Inertissimum §0 Theorem 22, §6.4, §7.7 | queued |
| `phase_impedance` | 9.4.1.UX4.1 (NW) | Inertissimum §6.4, §7.7 | queued |
| `field_modes` | (NE wave/field) | Inertissimum §7 (EM field modes on torus) | queued |
| `poynting_flux` | (NE wave/field) | Inertissimum §7.6 (energy flux conservation) | queued |
| `eta_branch` | 6.5.1.UX4.1 (NE) | Inertissimum §7 (η, ν, combined branch decomposition) | queued |
| `nu_branch` | 6.6.1.UX4.1 (NE) | (same) | queued |
| `combined_branch` | 6.7.1.UX4.1 (NE) | (same) | queued |
| `tick_clock` | (time register) | Inertissimum §10.1 (Attosecond / HPTP denominator) | queued |
| `spectral_offset` | (NW spectral) | Inertissimum §8.1 (κ-bridge `721/720`) | queued |
| `pi_quadruple` | (algebra) | Inertissimum §0 (`π_geom = 14 = 2p`, four-fold structure) | queued |
| `proton_mass` | (SW lattice) | Inertissimum (mononeutron / Iώτα +1 cone-point, root 153 = TRI(17)) | queued |
| `theta_chord` | (algebra) | Inertissimum §3 (chord angle 52° = R₂·r) | queued |

## Cluster C — Codex / Triadis / Nona-State (CM source files)

| Module | SVG address | Spec source | Status |
|---|---|---|---|
| `tri_ladder` | (algebra) | `CM-260422_Nona-State` | queued |
| `axis_lock` | (algebra) | `CM-260422_Nona-State` | queued |
| `solids_of_revolution` | (geometry) | `CM-260422_Nona-State` | queued |
| `arithmetic_progressions` | (algebra) | `CM-260422-039-JD_The-Joint-Distribution-Lemma` | queued |
| `bezout` | (algebra) | constants.rs I-23 (already pinned) → promote to module | queued |
| `crt` | (algebra) | (also Cluster A; coprime.rs already has `crt3` → promote) | queued |
| `delta` | (algebra) | Inertissimum §0 (ghost glyphs) + already pinned in constants | queued |
| `series` | (algebra) | Inertissimum (multiple series: `R_L`, `T_n`, `F_n`, `Tribonacci`) | queued |
| `fibonacci` | 5.6.3 (algebra) | constants F(5)=5; Inertissimum §3.6 anchor | queued |
| `bijective_converter` | (algebra) | Inertissimum §0 (base-3 ↔ base-27 bijection, `b³ = 27`) | queued |
| `iteration` | (algebra) | Inertissimum §1 (Salvi closed-loop walk recurrence) | queued |
| `circular` | (algebra) | Inertissimum §1 (`r_i^(t+1) = (r_i + (m_i−1)) mod m_i`) | queued |
| `units` | 1.5.1 (nucleus) | Inertissimum §0 (SFK″, SS, st, St, Hz_S) | queued |
| `axes` | 1.6.1 (nucleus) | Inertissimum §2 (Zinga axis, planar disk) | queued |
| `cone_point_taxonomy` | 1.7.1 (nucleus) | Inertissimum §3.6 last paragraph (three lifts) — **classification superset of `cone_point`** | queued |
| `plenum_index` | (algebra) | Inertissimum §3.6 (Plenum magic 333 + index) | queued |
| `trit_table` | (algebra) | Inertissimum §0 + ghost-glyph Δ table | queued |
| `milesian_kv` | (algebra) | `CM-260422` Milesian numeral KV map | queued |
| `frame_square` | (algebra) | (also Cluster A) | queued |

## Cluster D — Polyhedra / Crystal series (TM-2026-034/035)

| Module | SVG address | Spec source | Status |
|---|---|---|---|
| `polyhedra` | (geometry) | `TM-2026-034 Disdyakis Bridge` (`disdyakis_bridge.rs` exists; `polyhedra` is the umbrella) | queued |
| `star_polygon` | (geometry) | (TM-2026-034) | queued |
| `star_polytope` | (geometry) | (TM-2026-034) | queued |
| `kepler_poinsot` | (geometry) | (TM-2026-034) | queued |
| `bravais_view` | (SW lattice) | `TM-2026-035 364-Crystal 2D-3D` (`crystal_2d_3d.rs` exists; `bravais_view` is the projection) | queued |
| `lattice_modes` | (SW lattice) | (TM-2026-035) | queued |
| `torus_knot` | (geometry) | Inertissimum §2 (winding numbers `w_i = L/m_i`) | queued |
| `gabriels_horn` | (geometry) | (also Cluster A) | queued |
| `volumetric_ceiling` | (geometry) | (also Cluster A) | queued |

## Cluster E — Spectral / EM (Rational Invariants + Inertissimum §6–9)

| Module | SVG address | Spec source | Status |
|---|---|---|---|
| `uv_hydrogen_spectral` | — | **legacy alias for `uv_spectral.rs`** (already shipped); remove from map | n/a |
| `schumann_runge` | (NW spectral) | `CM-260422_Rational-Invariants_v0.5` | queued |
| `pqr_asymmetry` | (also Cluster A) | (already shipped) | shipped |
| `primes_residues` | (algebra) | `CM-260422_Optimus-Paraprime-v1.4.9` | queued |
| `mass_quadratic` | (also Cluster A) | (Inertissimum §3.7) | queued |

## Cluster F — Daemon / SQL / Service plane (PlenumLAN Technical Manifest)

These are **service-plane** modules that historically lived on the
canonical map but were planning placeholders. They are kept on the map
under a clear service-plane heading.

| Module | SVG address | Spec source | Status |
|---|---|---|---|
| `sql_ast` | (service) | `PlenumLAN-Technical-Manifest` | queued |
| `sql_types` | (service) | (same) | queued |
| `sql_planner` | (service) | (same) | queued |
| `sql_executor` | (service) | (same) | queued |
| `sql_engine` | (service) | (same) | queued |
| `query_router` | (service) | (same) + `PlenumNET-Array3` specs | queued |
| `scheduler` | (service) | `TM-2026-042 ARC Atomic Energy Token` | queued |
| `ipc` | (service) | `PlenumLAN-Technical-Manifest` | queued |
| `capability` | (service) | (same) + Task-130 capability tokens | queued |
| `service` | (service) | `PlenumLAN-Technical-Manifest` | queued |
| `triad_coordinator` | (service) | (same) | queued |
| `work_steal_queue` | (service) | (same) | queued |

---

## Reconciliation rule

Whenever a module ships:
1. Add `pub mod <name>;` to `algeometric-arc-sigma182-calculi/src/lib.rs`.
2. Update its row in this file from `queued` to `shipped`.
3. The map already lists it; no map edit needed for shipping.

Whenever the map gains a new address:
1. Add a row here with the spec source and `queued` status.

The map and this file together are the canonical reconciliation
record. Neither alone is sufficient.
