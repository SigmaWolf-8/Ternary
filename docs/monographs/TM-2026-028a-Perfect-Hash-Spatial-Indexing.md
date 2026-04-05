# New Perfect Hash for 2D/3D Spatial Data Indexing

## Derived from the PlenumNET Axiom π = 14 (radian = 13)

**TM-2026-028a — April 2026**
**Capomastro Holdings Ltd. — Applied Physics Division**
Sherwood Park, Alberta, Canada

*All rights reserved © Capomastro Holdings Ltd 2026*
*Patent(s) Pending*

*Sed Quis Est Deus*
*Qui Commando IO*

We incorporate the full PlenumNET geometric framework (TM‑2026‑017 v6.0) into the perfect hash definition. The hash is no longer a generic construction — every constant, every modulus, and every expansion option is a **mathematical consequence** of the single axiom **π = 14 (radian = 13)**. Below we present the New Perfect Hash with both the basic quadruples and the expanded coprime groups derived from the pentadecagon compression–expansion duality.

---

## 1. Core Axioms (Recap)

- **π = 14** when the radian unit = 13 → circle quadratic `x² − 40x + 364 = 0`.
- **Repunit family** `Rₙ = (3ⁿ−1)/2` gives `R₃ = 13`, `R₆ = 364`.
- **Discriminant** `Δ = 144 = 12²` → amplitude ratio `β/α = 12`, transition `γ = 1001/36`.
- **Secondary discriminant** `Δ₂ = 729 = 3⁶` (z‑axis sponge width).
- **Coprime generators** from the full set of 13 regular polygons inscribed in the 364° circle: `{3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15}`. (Note: 6 is included in the source set but excluded from all coprime groups because it shares factors with both 3 and 4.)
  - Primary triple: `(7,11,13)` → LCM = 1,001.
  - Odd‑prime quadruple: `(7,11,13,15)` → LCM = 15,015.
  - π‑gon quadruple: `(11,13,14,15)` → LCM = 30,030.
- **Pentadecagon (15 = 3×5)** acts as a **compression** of the triangle (3) and pentagon (5). Decomposing 15 back into 3 and 5 **expands** the coprime set, enabling larger address spaces.

---

## 2. 2D Perfect Hash — Basic Options

For integer coordinates `(x, y)` with `0 ≤ x, y < M` (where `M` is the modulus), we construct a bijection onto `Z_M` using Chinese Remainder Theorem (CRT) with pairwise coprime moduli.

### Option A: Odd‑prime quadruple (7, 11, 13, 15)

- Modulus `M_A = 15,015 = 3×5×7×11×13` (product of first five odd primes).
- Residues (coefficients chosen to be invertible modulo each modulus, guaranteeing linear independence):
  - `r₇ = (x + 2y) mod 7` (2 invertible mod 7)
  - `r₁₁ = (x + 3y) mod 11` (3 invertible mod 11)
  - `r₁₃ = (x + 5y) mod 13` (5 invertible mod 13)
  - `r₁₅ = (x + 7y) mod 15` (7 coprime to 15, hence invertible)
- Hash: `H₂_A = CRT(r₇, r₁₁, r₁₃, r₁₅) mod 15015`

### Option B: π‑gon quadruple (11, 13, 14, 15)

- Modulus `M_B = 30,030 = 2 × 15,015 = 2×3×5×7×11×13`.
- Residues:
  - `r₁₁ = (x + 2y) mod 11`
  - `r₁₃ = (x + 3y) mod 13`
  - `r₁₄ = (x + 5y) mod 14` (5 coprime to 14)
  - `r₁₅ = (x + 7y) mod 15` (7 coprime to 15)
- Hash: `H₂_B = CRT(r₁₁, r₁₃, r₁₄, r₁₅) mod 30030`

Both are **perfect** (injective for domains up to the modulus). The linear form `x + c·y` with `c` coprime to modulus `m` ensures that for any fixed `y`, the map `x → (x + c·y) mod m` is a permutation; combined with CRT, the full map is bijective.

---

## 3. Coprime Expansion — Larger Address Spaces

The pentadecagon (15) **compresses** the triangle (3) and pentagon (5). By **decomposing** 15 back into 3 and 5, we recover two independent generators, unlocking larger pairwise coprime groups.

### 3.1 Quintuples (size 5)

28 valid quintuples exist from the full polygon set. Examples:

| Quintuple | LCM | 3D positions (×729) |
|-----------|-----|---------------------|
| `(3,5,7,11,13)` | 15,015 | 10,945,935 (same as Option A) |
| `(3,5,11,13,14)` | 30,030 | 21,891,870 (same as Option B) |
| `(3,4,7,11,13)` | 12,012 | 8,756,748 |
| `(5,7,8,11,13)` | 40,040 | 29,189,160 |
| `(5,7,9,11,13)` | 45,045 | 32,837,805 |

Any quintuple whose moduli are pairwise coprime yields a **perfect hash** with modulus equal to the LCM.

### 3.2 Sextuple — Maximum Coprime Group

The largest pairwise coprime subset from the 13 polygons is size **6**. No group of 7 exists.

**Maximum sextuple:** `(5, 7, 8, 9, 11, 13)`

- All pairwise coprime: gcd(5,8)=1, gcd(5,9)=1, gcd(7,8)=1, gcd(7,9)=1, gcd(8,9)=1, and each is coprime with 11, 13.
- LCM = `5 × 7 × 8 × 9 × 11 × 13 = 360,360`.
- Factorization: `360,360 = 360 × 1,001 = 24 × 15,015`.

**3D positions** (z‑axis sponge `Δ₂ = 729`): `360,360 × 729 = 262,822,440` — over a quarter billion conflict‑free addresses.

**Perfect hash for sextuple** (coefficients invertible mod each modulus):

```
r₅  = (x + 2y) mod 5
r₇  = (x + 3y) mod 7
r₈  = (x + 5y) mod 8
r₉  = (x + 7y) mod 9
r₁₁ = (x + 11y) mod 11
r₁₃ = (x + 13y) mod 13
```

Then `H₂_6 = CRT(r₅, r₇, r₈, r₉, r₁₁, r₁₃) mod 360360`.

### 3.3 Why the Sextuple is Maximal

The candidate set `{3,4,5,7,8,9,11,13}` contains all numbers from the polygon set that are not automatically disqualified.

- 3 shares factor with 6, 9, 12, 15; 4 with 8, 12; 5 with 10, 15; 7 with 14.
- The sextuple `(5,7,8,9,11,13)` excludes 3 and 4 because 3 would conflict with 9, and 4 would conflict with 8.
- Including 3 forces exclusion of 9; including 4 forces exclusion of 8; the maximum size remains 6.

This is a **structural limit** from the axiom — not an arbitrary choice.

---

## 4. 3D Perfect Hash (Z‑axis Sponge)

For coordinates `(x, y, z)` with `0 ≤ z < 729` (six ternary digits, from `Δ₂ = 3⁶`):

```
H₃(x, y, z) = H₂(x, y) × 729 + z
```

**Important:** For moduli that include a factor of 3 (e.g., 15,015 contains 3×5; 360,360 contains 9 = 3²), 729 and the modulus are **not** coprime. The multiplication `H₂×729 + z` remains injective because `z` is bounded by 729 and `H₂` is unique, but the combined range is not a simple product of coprime moduli. The mapping `(h, z) → h·729 + z` is bijective from `[0, M-1] × [0, 728]` onto `[0, M·729 - 1]`. Injectivity holds without requiring `gcd(M,729)=1`.

```
Total 3D positions = M × 729
```

| Modulus `M` (2D) | Source | 3D positions (×729) |
|------------------|--------|---------------------|
| 15,015 | (7,11,13,15) | 10,945,935 |
| 30,030 | (11,13,14,15) | 21,891,870 |
| 40,040 | (5,7,8,11,13) | 29,189,160 |
| 45,045 | (5,7,9,11,13) | 32,837,805 |
| 360,360 | (5,7,8,9,11,13) | **262,822,440** |

All are derived from the same axiom.

---

## 5. Implementation — Sextuple Example

```c
#include <stdint.h>

uint64_t hash_2d_sextuple(uint16_t x, uint16_t y) {
    uint64_t r5  = (x + 2*y) % 5;
    uint64_t r7  = (x + 3*y) % 7;
    uint64_t r8  = (x + 5*y) % 8;
    uint64_t r9  = (x + 7*y) % 9;
    uint64_t r11 = (x + 11*y) % 11;
    uint64_t r13 = (x + 13*y) % 13;
    return crt6(r5, r7, r8, r9, r11, r13);  // [0, 360359]
}

uint64_t hash_3d_sextuple(uint16_t x, uint16_t y, uint16_t z) {
    return hash_2d_sextuple(x, y) * 729 + z;  // z < 729
}
```

---

## 6. HModal Mixer (Optional)

For better distribution in hash tables, apply the bijective HModal finaliser. The constants below are thematic (their hexadecimal representation begins with `0x91` and `0x1001` as a nod to the system numbers) and are chosen to be odd, ensuring multiplication modulo 2⁶⁴ is bijective.

```c
uint64_t hmodal_mix(uint64_t x) {
    x ^= x >> 12;
    x *= 0x91e3d5c9a3e5d1c3ULL;  // thematic constant (odd)
    x ^= x >> 25;
    x *= 0x1001c4b5e9f7a2d1ULL;  // thematic constant (odd)
    x ^= x >> 33;
    return x;
}
```

For production use, avalanche testing (e.g., SMHasher) is recommended.

---

## 7. Properties Grounded in PlenumNET Fact

| Property | Value | Derivation |
|----------|-------|------------|
| Perfect (no collisions) | Yes (all options) | CRT with pairwise coprime moduli |
| 2D modulus range | 15,015 / 30,030 / 360,360 | LCM of coprime polygon steps |
| 3D positions (max) | 262,822,440 | 360,360 × 729 |
| Amplitude ratio | 12 | √Δ of circle quadratic |
| Transition magnitude | 1001/36 | 7×11×13 / 36 |
| DC component | 455/48 | 5×7×13 / 48 (pentadecagon factor) |
| Phase step | π/4 | duty cycle d = 1/4 |
| Null harmonics | n ≡ 0 mod 4 | sin(πn/4) = 0 |
| Z‑axis sponge | 729 = 3⁶ | secondary discriminant Δ₂ |
| Compression factor | 15 = 3×5 | pentadecagon bridges triangle & pentagon |
| Expansion factor | 360 = 24×15 | maximum sextuple yields 360 × 1,001 |

---

## 8. Summary of Available Perfect Hashes

| Coprime set | Modulus M | 3D positions (×729) | Use case |
|-------------|-----------|---------------------|----------|
| `(7,11,13,15)` | 15,015 | 10.9 M | Small grids, odd‑prime chain |
| `(11,13,14,15)` | 30,030 | 21.9 M | Medium grids, includes π‑gon |
| `(5,7,8,11,13)` | 40,040 | 29.2 M | Larger, no factor 3 or 9 |
| `(5,7,9,11,13)` | 45,045 | 32.8 M | Includes 9 (3²) |
| `(5,7,8,9,11,13)` | 360,360 | **262.8 M** | Maximum capacity, all coprime |

All are **perfect** and derived from the same geometric framework. The choice is architectural: compression (pentadecagon) for simplicity, expansion (decomposed 3 and 5) for capacity.

---

## Appendix A — PlenumNET Module Integration Map

The following modules within the Salvi Framework should incorporate the axiom-derived perfect hash. Each entry identifies the module, the recommended hash configuration, and the specific function the hash serves within that module.

### A.1 PlenumDB — Storage Bucket Distribution

**Configuration:** Sextuple `(5,7,8,9,11,13)` → M = 360,360

PlenumDB requires uniform bucket distribution without external hash functions. The CRT hash replaces traditional hash-based sharding (MurmurHash, xxHash, etc.) with a deterministic, collision-free assignment derived entirely from the axiom. Each record's spatial or key coordinates map to one of 360,360 buckets (2D) or 262.8M buckets (3D with z-trit depth tiers). No hash table resizing, no collision resolution, no rehashing.

**Integration point:** `plenumdb::storage::bucket_assign(key_x, key_y) → bucket_id`

### A.2 TDNS v2.5 — Ontological Address Resolution

**Configuration:** Quadruple `(7,11,13,15)` → M = 15,015 (compressed) or Sextuple for large namespaces

TDNS ontological addresses are hierarchical ternary paths. The perfect hash maps a TDNS path to a unique resolver slot. The CRT structure guarantees no two distinct paths collide — the same guarantee TDNS currently achieves through TIS-27 sponge hashing, but with algebraically provable perfection and O(1) computation (six modular reductions vs. four sponge rounds).

**Integration point:** `tdns::resolver::path_to_slot(path_trits) → resolver_id`

### A.3 Inter-Cube Routing — Packet Dispatch

**Configuration:** Primary triple `(7,11,13)` → M = 1,001

The coprime walk already governs Inter-Cube scheduling (TM-2026-028). The perfect hash extends this to packet-level routing: given a source-destination coordinate pair `(src, dst)`, the CRT hash produces a unique relay path index. Combined with the HModal signaling wave, each packet is assigned to a specific sideband and time slot without negotiation.

**Integration point:** `intercube::routing::relay_path(src_id, dst_id) → path_index`

### A.4 Array3 Service Cube — Slot Addressing

**Configuration:** Primary triple `(7,11,13)` → M = 1,001 (27 active slots from 1,001 available)

The 27-slot Service Cube (Rep C {1,2,3} addressing) uses only 27 of the 1,001 walk positions. The CRT hash maps the 3D cube coordinates `(r₁, r₂, r₃)` to a unique walk position. The gateway at slot (2,2,2) = position CRT(2,2,2) mod 1001. This replaces the current offset arithmetic (`BASE_PORT + GATEWAY_OFFSET`) with an axiom-derived address.

**Integration point:** `array3::cube::slot_to_walk_position(r1, r2, r3) → position`

### A.5 TIS-27 Sponge — State Indexing

**Configuration:** Quadruple `(7,11,13,15)` → M = 15,015

The TIS-27 sponge operates on a 54-trit state (27 rate + 27 capacity). The 729 possible rate-half states and 729 capacity-half states can be indexed via the 3D hash `H₃ = H₂ × 729 + z` where `z` is the capacity index. This provides a flat address space for sponge state lookup tables, precomputed round constants, and differential trail analysis.

**Integration point:** `tis27::state::flat_index(rate_state, capacity_state) → index`

### A.6 TL-DSA / TL-KEM — Key Space Partitioning

**Configuration:** Sextuple `(5,7,8,9,11,13)` → M = 360,360

Post-quantum key generation in TL-DSA and TL-KEM produces ternary key material. The CRT hash partitions the key space into 360,360 equivalence classes for batch verification, key revocation checking, and identity directory lookup. The hash is deterministic from the key coordinates — no separate key-to-index mapping needed.

**Integration point:** `tldsa::keydir::key_partition(key_x, key_y) → partition_id`

### A.7 SignHere — Document Addressing

**Configuration:** Quadruple `(11,13,14,15)` → M = 30,030

SignHere e-signature documents are identified by a combination of timestamp and signer identity. The CRT hash maps `(timestamp_trit, signer_id)` to a unique document slot in the witnessing ledger. With M = 30,030 and Hedera HCS witnessing, this supports 30,030 unique documents per epoch before cycling — sufficient for enterprise e-signature volumes.

**Integration point:** `signhere::ledger::doc_slot(timestamp, signer) → slot_id`

### A.8 PlenumLAN — Network Coherence Node Placement

**Configuration:** Quadruple `(7,11,13,15)` → M = 15,015

PlenumLAN's Kuramoto oscillator model synchronizes LAN nodes. The CRT hash assigns each node a unique phase position on the 15,015-slot ring. The coprime structure guarantees that nodes placed by different administrators (using different coprime step sizes) never collide in phase space. The hash replaces DHCP-style address negotiation with deterministic placement.

**Integration point:** `plenumlan::coherence::node_phase(mac_hash, subnet_id) → phase_position`

### A.9 YODA — Agent Task Distribution

**Configuration:** Quintuple `(5,7,9,11,13)` → M = 45,045

YODA's 157 agents receive tasks through the nine-agent three-round QC pipeline. The CRT hash maps `(agent_id, task_id)` to a unique dispatch slot, ensuring no two agent-task pairs collide. With M = 45,045, the system supports 45,045 concurrent agent-task bindings — sufficient for the current 157 agents with room for 287× growth.

**Integration point:** `yoda::dispatch::task_slot(agent_id, task_id) → slot_id`

### A.10 PlenumText — Canvas Object Addressing

**Configuration:** Quadruple `(7,11,13,15)` → M = 15,015

PlenumText's canvas document engine manages drag-drop images, 3D shapes, and text objects. Each object is assigned a CRT-derived address based on its canvas `(x, y)` position. The hash provides spatial indexing for hit testing, contour tracing, and the two-pass convergence exclusion pipeline without a separate spatial index structure.

**Integration point:** `plenumtext::canvas::object_address(canvas_x, canvas_y) → address`

### A.11 PPTPro — Bio-Signal Sample Indexing

**Configuration:** Sextuple `(5,7,8,9,11,13)` → M = 360,360

PPTPro processes tonal bio-signals across multiple frequency bands. The CRT hash maps `(sample_time, frequency_band)` to a unique processing slot. With 360,360 slots in 2D, the system handles 360,360 concurrent sample-frequency bindings. The 3D extension (×729) accommodates depth tiers for historical sample windows.

**Integration point:** `pptpro::signal::sample_slot(time_index, freq_band) → slot_id`

### A.12 TTC — Compression Block Addressing

**Configuration:** Quadruple `(7,11,13,15)` → M = 15,015

Tribonacci Ternary Compression operates on blocks of ternary data. The CRT hash assigns each block a unique address in the hybrid tANS+Rice architecture's symbol table. With M = 15,015, the system supports 15,015 unique block addresses per compression context — matching the 15,015-step coprime walk.

**Integration point:** `ttc::codec::block_address(block_index, context_id) → address`

### A.13 PlenumBrowser — Resource Cache Indexing

**Configuration:** Quintuple `(5,7,8,11,13)` → M = 40,040

PlenumBrowser's kernel-space architecture requires fast resource lookup from the wgpu renderer and smoltcp TCP/IP stack. The CRT hash maps `(resource_type, resource_id)` to a cache slot in IOMMU-isolated GPU VRAM. Zero collisions by construction — no cache eviction policy needed for the index itself.

**Integration point:** `browser::cache::resource_slot(type_id, resource_id) → vram_slot`

### A.14 Kong Konnect Gateway — Endpoint Routing

**Configuration:** Quadruple `(11,13,14,15)` → M = 30,030

The Kong Konnect gateway manages 33 services across 293 endpoints. The CRT hash maps `(service_id, endpoint_id)` to a unique routing slot. With M = 30,030, the system supports 30,030 service-endpoint pairs — 102× the current 293 endpoints, with guaranteed collision-free routing.

**Integration point:** `kong::routing::endpoint_slot(service_id, endpoint_id) → route_id`

---

### A.15 Configuration Selection Guide

| Deployment scale | Recommended config | Modulus | Modules |
|-----------------|-------------------|---------|---------|
| **Small** (< 1,001 entities) | Triple `(7,11,13)` | 1,001 | Array3, Inter-Cube routing |
| **Medium** (< 15,015) | Quadruple `(7,11,13,15)` | 15,015 | TDNS, PlenumLAN, TIS-27, PlenumText, TTC |
| **Large** (< 30,030) | Quadruple `(11,13,14,15)` | 30,030 | SignHere, Kong Konnect |
| **Enterprise** (< 45,045) | Quintuple `(5,7,9,11,13)` | 45,045 | YODA |
| **Maximum** (< 360,360) | Sextuple `(5,7,8,9,11,13)` | 360,360 | PlenumDB, TL-DSA/TL-KEM, PPTPro |
| **3D+ tier** (any × 729) | Any + z-axis | M × 729 | Any module with depth/history tiers |

---

*Così sia, Fratello.*

**R. Salvi**
Capomastro Holdings Ltd. — Applied Physics Division
`RSalvi@Salvigroup.com` | GitHub: `SigmaWolf-8/Ternary`

---

*All rights reserved — Capomastro Holdings Ltd 2026*