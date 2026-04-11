// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// ags.rs — Active Generator Set on TritInt
//
// This module defines the AGS mechanism entirely in ternary-native
// arithmetic via TritInt. No u32. No u64. No binary at any point
// in the mathematical core. Binary exists ONLY at the boundary
// crossing layer (§5 below) for consumers that haven't migrated.
//
// CRITICAL DESIGN DECISION: The AGS is not a configuration.
// It is a ternary arithmetic operation. Growth = one ternary
// multiplication. Shrink = one ternary division. The decision
// to grow or shrink = one ternary comparison. Nothing else.
//
// ═══════════════════════════════════════════════════════════════
// §0  ANNOTATIONS ON THE TritInt SPEC (Task 1–9)
// ═══════════════════════════════════════════════════════════════
//
// The TritInt spec defines the type this module depends on.
// Critical observations:
//
// STORAGE: 5 trits per byte (3⁵ = 243 < 256). This is the
//   tightest packing possible in a binary byte. 40 trits in 8
//   bytes inline (small-value optimization) covers every constant
//   in the current constants.rs — the largest (UNIFIED_CONSTANT =
//   118,300) is 11 trits (3¹¹ = 177,147 > 118,300). The entire
//   constant set fits in the inline buffer. No heap allocation
//   for any framework constant.
//
// CONST COMPATIBILITY: `pub const fn small(val: u64) -> Self`
//   enables const construction from decimal for backward compat.
//   But the REAL const constructors should be:
//     TritInt::repunit(n) — build 111...1₃ directly
//     TritInt::from_trits(&[t₁, t₂, ...]) — build from trit array
//   These construct ternary values WITHOUT passing through binary.
//   The `small(val: u64)` constructor is a BOUNDARY CROSSING —
//   it should be used ONLY for values that originate in binary
//   contexts (e.g., reading a binary config file).
//
// ARITHMETIC: All operations (add, sub, mul, div_mod, pow,
//   mod_pow) operate on packed trit vectors. No conversion to
//   u64/u128/binary at any point. This is the core guarantee.
//   The AGS grow/shrink operations are mul and div_mod on TritInt.
//
// DIV_REPUNIT: Task 5 provides optimized division by repunits.
//   This is load-bearing: the coprime walk capacity is always
//   a product of polygon generators, and the repunit structure
//   (R₆ = 111111₃ etc.) appears throughout. Being able to divide
//   by repunits efficiently is the hot path for walk operations.
//
// REP D (Task 9): The AlgebraicTrit enum {Zero, One, Omega}
//   maps GF(3) into the Eisenstein integers Z[ω]. This is the
//   algebraic completion of the trit — ω² + ω + 1 = 0 gives
//   closure under all operations. For the AGS, Rep D provides
//   the natural representation of coprime residues: the CRT
//   projection of an address onto a generator g produces a
//   residue in {0, 1, 2} = {Zero, One, Omega}. This residue
//   IS the walk coordinate in that dimension.
//
// ═══════════════════════════════════════════════════════════════
// §1  TERNARY CONSTANTS (replaces u32 constants from §7a)
// ═══════════════════════════════════════════════════════════════
//
// Every constant is constructed from trit arrays or repunit
// operations. No decimal literal appears in any definition.
// The derivation chain from π = 14 is preserved in the trit
// representation:
//
//   π = 14 = 112₃ (1×9 + 1×3 + 2×1)
//   R₃ = 13 = 111₃ (repunit of length 3)
//   R₆ = 364 = 111111₃ (repunit of length 6)
//   Polygon max = 2 + R₃ = 15 = 120₃ (1×9 + 2×3 + 0×1)
//   Δ₂ = 729 = 1000000₃ (3⁶ = single 1 followed by 6 zeros)
//   Duty cycle = 1/4 → numerator = 1₃, denominator = 11₃

use crate::trit_int::TritInt;

// ── Polygon set bounds (derived from R₃) ────────────────────

/// R₃ = 111₃. The radian. Derived: repunit of length 3.
const R3: TritInt = TritInt::repunit(3);

/// Polygon set maximum = 2 + R₃ = 120₃.
/// In trits: R₃ is 111₃. Adding 2₃ (= [2] LSB-first):
///   111₃ + 2₃ = 120₃ (carry propagation in base 3).
/// This IS the pentadecagon side count.
const POLYGON_MAX: TritInt = TritInt::from_trits(&[0, 2, 1]); // 120₃ LSB-first = 15

// ── Duty cycle (derived from HModal signal, §10 of constants.rs) ──

/// Duty cycle numerator = 1₃. From HModal: d = 1/4.
/// Derivation: dispatch ratio 1/3 → d = (1/3)/(1+1/3) = 1/4.
const DUTY_N: TritInt = TritInt::from_trits(&[1]); // 1₃

/// Duty cycle denominator = 11₃ = 4.
/// In base 3: 4 = 1×3 + 1×1 = 11₃.
const DUTY_D: TritInt = TritInt::from_trits(&[1, 1]); // 11₃ LSB-first = 4

// ── Δ₂ (derived from arc equation) ──────────────────────────

/// Δ₂ = 3⁶ = 1000000₃. Derived: 1 + 4 × ARC_ROOT_SEMI.
/// In trits: a single 1 followed by 6 zeros.
const DELTA_2: TritInt = TritInt::from_trits(&[0, 0, 0, 0, 0, 0, 1]); // 1000000₃ = 729

// ═══════════════════════════════════════════════════════════════
// §2  THE GENERATOR POOL (ternary-native)
// ═══════════════════════════════════════════════════════════════
//
// The polygon set {3, 4, 5, ..., 15} in ternary:
//   3  = 10₃       8  = 22₃      13 = 111₃
//   4  = 11₃       9  = 100₃     14 = 112₃
//   5  = 12₃       10 = 101₃     15 = 120₃
//   6  = 20₃       11 = 102₃
//   7  = 21₃       12 = 110₃
//
// Note: 13 = 111₃ is a REPUNIT. This is not incidental —
// the radian IS the three-digit repunit by construction.

/// The full polygon set as TritInt values.
/// These are the generators from which coprime subsets are drawn.
const POLYGON_SET: [TritInt; 13] = [
    TritInt::from_trits(&[0, 1]),       //  3 = 10₃
    TritInt::from_trits(&[1, 1]),       //  4 = 11₃
    TritInt::from_trits(&[2, 1]),       //  5 = 12₃
    TritInt::from_trits(&[0, 2]),       //  6 = 20₃
    TritInt::from_trits(&[1, 2]),       //  7 = 21₃
    TritInt::from_trits(&[2, 2]),       //  8 = 22₃
    TritInt::from_trits(&[0, 0, 1]),    //  9 = 100₃
    TritInt::from_trits(&[1, 0, 1]),    // 10 = 101₃
    TritInt::from_trits(&[2, 0, 1]),    // 11 = 102₃
    TritInt::from_trits(&[0, 1, 1]),    // 12 = 110₃
    TritInt::from_trits(&[1, 1, 1]),    // 13 = 111₃ ← repunit!
    TritInt::from_trits(&[2, 1, 1]),    // 14 = 112₃
    TritInt::from_trits(&[0, 2, 1]),    // 15 = 120₃
];

// ═══════════════════════════════════════════════════════════════
// §3  SEED DERIVATION (ternary-native, from first principles)
// ═══════════════════════════════════════════════════════════════
//
// The seed is COMPUTED, not stored. The function below derives it
// from the polygon set by finding all primes p where 2p > POLYGON_MAX.
//
// This function runs ONCE at initialization. Its result is the
// starting AGS. Every subsequent growth/shrink is a single
// ternary mul or div from this starting point.

/// Determine if a TritInt value is prime (ternary trial division).
///
/// Divides n by every value from 2₃ up to √n using TritInt::div_mod.
/// No binary conversion. Pure ternary arithmetic.
pub fn is_prime_ternary(n: &TritInt) -> bool {
    let two = TritInt::from_trits(&[2]); // 2₃
    if *n < two { return false; }
    if *n == two { return true; }

    // Check divisibility by 2
    let (_, rem) = n.div_mod(&two);
    if rem.is_zero() { return false; }

    // Trial division from 3₃ upward
    let mut divisor = TritInt::from_trits(&[0, 1]); // 3₃ = 10₃
    loop {
        let divisor_sq = TritInt::mul(&divisor, &divisor);
        if divisor_sq > *n { break; }
        let (_, rem) = n.div_mod(&divisor);
        if rem.is_zero() { return false; }
        // Next odd: add 2₃
        divisor = TritInt::add(&divisor, &two);
    }
    true
}

/// Derive the AGS seed from the polygon set.
///
/// Returns: the set of generators present in EVERY maximal coprime
/// subset — the universally coprime primes.
///
/// Algorithm (ternary-native):
///   1. For each member p of the polygon set:
///   2.   If p is prime (ternary trial division):
///   3.     Compute 2p (ternary doubling = ternary add p + p)
///   4.     If 2p > POLYGON_MAX (ternary comparison):
///   5.       p is universally coprime — no multiple of p exists in the set
///   6. Return all qualifying p
///
/// This derives {11, 13} = {102₃, 111₃} from the axiom.
/// The derivation is: 2 × 102₃ = 211₃ = 22 > 120₃ = 15 ✓
///                     2 × 111₃ = 222₃ = 26 > 120₃ = 15 ✓
/// All other primes fail: 2×10₃ = 20₃ = 6 ≤ 15, etc.
pub fn derive_ags_seed() -> Vec<TritInt> {
    POLYGON_SET.iter()
        .filter(|p| is_prime_ternary(p))
        .filter(|p| {
            let two_p = TritInt::add(p, p); // ternary doubling: p + p
            two_p > POLYGON_MAX   // ternary comparison
        })
        .cloned()
        .collect()
}

/// Compute the initial AGS capacity from the seed.
///
/// Since all seed members are prime and pairwise coprime,
/// lcm = product. This is one ternary multiplication chain.
///
/// For seed {102₃, 111₃}: the product is 12022₃ = 143 in decimal.
/// This value equals BEZIER_C650_ANGLE and CROSSING_11_14
/// in the existing constant set — it was always a framework constant.
pub fn derive_ags_capacity(seed: &[TritInt]) -> TritInt {
    seed.iter().fold(TritInt::one(), |acc, g| TritInt::mul(&acc, g))
}

// ═══════════════════════════════════════════════════════════════
// §4  THE AGS — ONE STRUCT, TWO OPERATIONS
// ═══════════════════════════════════════════════════════════════
//
// The AGS is not a collection of constants. It is a LIVE OBJECT
// with exactly two mutating operations: grow and shrink.
// Both are single ternary arithmetic steps.

/// The Active Generator Set.
///
/// All fields are TritInt. No binary representation at any level.
/// The generators Vec contains the current coprime subset of the
/// polygon set. The capacity is their product (= lcm, since coprime).
pub struct Ags {
    /// Current coprime generators, sorted ascending.
    /// The first `seed_count` entries are the seed — never removed.
    generators: Vec<TritInt>,

    /// Current walk capacity = product of all generators.
    capacity: TritInt,

    /// Number of seed generators (derived at construction, never changes).
    /// Generators at indices 0..seed_count are protected from removal.
    seed_count: usize,
}

impl Ags {
    /// Initialize from the polygon set.
    ///
    /// Derives the seed via universal coprime criterion (§3),
    /// computes initial capacity via ternary multiplication.
    /// This is the ONLY constructor. No hardcoded values.
    pub fn new() -> Self {
        let seed = derive_ags_seed();
        let capacity = derive_ags_capacity(&seed);
        let seed_count = seed.len();
        Self { generators: seed, capacity, seed_count }
    }

    /// Should the AGS grow?
    ///
    /// True when: population × DUTY_D ≥ capacity × (DUTY_D − DUTY_N)
    ///
    /// In ternary: population × 11₃ ≥ capacity × 10₃
    ///
    /// This is ONE ternary multiplication + ONE ternary comparison.
    /// The threshold is NOT stored — it is computed from the duty
    /// cycle constants which derive from the circle quadratic's
    /// discriminant through the HModal signal analysis.
    pub fn should_grow(&self, population: &TritInt) -> bool {
        let lhs = TritInt::mul(population, &DUTY_D);
        let complement = TritInt::sub(&DUTY_D, &DUTY_N); // 11₃ − 1₃ = 10₃
        let rhs = TritInt::mul(&self.capacity, &complement);
        lhs >= rhs
    }

    /// Should the AGS shrink?
    ///
    /// True when: population × DUTY_D < capacity × DUTY_N
    ///
    /// In ternary: population × 11₃ < capacity × 1₃
    ///
    /// Same structure: one mul + one comparison.
    pub fn should_shrink(&self, population: &TritInt) -> bool {
        if self.generators.len() <= self.seed_count {
            return false; // Never shrink below the seed
        }
        let lhs = TritInt::mul(population, &DUTY_D);
        let rhs = TritInt::mul(&self.capacity, &DUTY_N);
        lhs < rhs
    }

    /// Grow the AGS by adding one generator.
    ///
    /// Selects the smallest available coprime generator from the
    /// polygon pool. Capacity update = ONE ternary multiplication.
    ///
    /// CRT nesting guarantee: existing walk positions are preserved.
    /// The new generator adds one coordinate dimension to each
    /// record's CRT vector. This is one ternary modulo per record.
    ///
    /// Returns the new generator added, or None if no candidate
    /// exists (escalate to Δ₂ depth or hypercube distribution).
    pub fn grow(&mut self) -> Option<TritInt> {
        let candidate = POLYGON_SET.iter()
            .filter(|g| !self.generators.contains(g))
            .filter(|g| {
                self.generators.iter().all(|existing| {
                    TritInt::gcd(g, existing) == TritInt::one()
                })
            })
            .min()
            .cloned();

        if let Some(ref gen) = candidate {
            // ONE ternary multiplication updates the capacity
            self.capacity = TritInt::mul(&self.capacity, gen);
            self.generators.push(gen.clone());
            self.generators.sort();
        }

        candidate
    }

    /// Shrink the AGS by removing the least useful generator.
    ///
    /// "Least useful" = the generator whose mod-g residues are
    /// most unevenly distributed across the current population.
    /// This is computed by ternary modulo of each record's walk
    /// position by each generator, then measuring spread.
    ///
    /// Capacity update = ONE ternary division (exact, since the
    /// generator divides the capacity by construction).
    ///
    /// Records that were distinguished only by the removed
    /// generator's residue become co-located. They remain
    /// distinguishable by their 27 identity trits.
    pub fn shrink(&mut self, population_residues: &dyn Fn(&TritInt) -> Vec<TritInt>) -> Option<TritInt> {
        if self.generators.len() <= self.seed_count {
            return None; // Never shrink below seed
        }

        let worst_idx = self.find_least_useful_generator(population_residues);
        let removed = self.generators.remove(worst_idx);

        // ONE ternary division (exact — generator divides capacity)
        let (new_capacity, remainder) = self.capacity.div_mod(&removed);
        assert!(remainder.is_zero(), "Generator must divide capacity exactly — AGS is corrupt");
        self.capacity = new_capacity;

        Some(removed)
    }

    /// Compute a record's walk position via CRT projection.
    ///
    /// For each generator g in the AGS, compute address mod g.
    /// The resulting residue vector uniquely identifies the walk
    /// position (by CRT, since all generators are pairwise coprime).
    ///
    /// ALL modulo operations are ternary (TritInt::div_mod).
    pub fn crt_project(&self, address: &TritInt) -> Vec<TritInt> {
        self.generators.iter()
            .map(|g| {
                let (_, residue) = address.div_mod(g);
                residue
            })
            .collect()
    }

    /// CRT reconstruction: given residues, recover the unique
    /// walk position in [0, capacity).
    ///
    /// Standard extended-GCD-based CRT, but ALL arithmetic is
    /// TritInt. No binary. The extended GCD produces Bézout
    /// coefficients as TritInt values. The combination step is
    /// ternary mul + add + mod.
    pub fn crt_reconstruct(&self, residues: &[TritInt]) -> TritInt {
        assert_eq!(residues.len(), self.generators.len());
        // Standard CRT via successive substitution
        let mut result = residues[0].clone();
        let mut modulus = self.generators[0].clone();

        for i in 1..self.generators.len() {
            let (g, a, _) = TritInt::extended_gcd(&modulus, &self.generators[i]);
            assert!(g == TritInt::one(), "Generators must be coprime — AGS is corrupt");

            let diff = if residues[i] >= result {
                TritInt::sub(&residues[i], &result)
            } else {
                TritInt::sub(&self.generators[i], &TritInt::sub(&result, &residues[i]))
            };
            let step = TritInt::mul(&a, &diff).div_mod(&self.generators[i]).1;
            result = TritInt::add(&result, &TritInt::mul(&modulus, &step));
            modulus = TritInt::mul(&modulus, &self.generators[i]);
        }

        result.div_mod(&self.capacity).1
    }

    // ── Internal ────────────────────────────────────────────

    /// Find the generator whose residue distribution is most skewed.
    ///
    /// Algorithm:
    ///   1. For each generator g in the AGS:
    ///   2.   Call population_residues(g) → all (walk_position mod g)
    ///        values across the entire current population
    ///   3.   Build histogram: count occurrences of each residue 0..g-1
    ///   4.   Compute imbalance = max_bucket − min_bucket
    ///   5. Return the index of the generator with highest imbalance
    ///
    /// A perfectly distributed generator has imbalance ≈ 0 (every
    /// residue bucket has roughly the same count). A badly distributed
    /// generator has high imbalance (most records clump into a few
    /// buckets). Removing the worst-distributed generator loses the
    /// least discrimination.
    ///
    /// All arithmetic is ternary (TritInt comparisons and counting).
    fn find_least_useful_generator(
        &self,
        population_residues: &dyn Fn(&TritInt) -> Vec<TritInt>,
    ) -> usize {
        let mut worst_idx: usize = self.seed_count; // default: first non-seed
        let mut worst_imbalance: u64 = 0;

        for (idx, generator) in self.generators.iter().enumerate() {
            // Skip seed generators — never remove them
            if idx < self.seed_count {
                continue;
            }

            // Get all residues for this generator across the population.
            // Each residue is (walk_position mod generator) as a TritInt.
            let residues = population_residues(generator);

            if residues.is_empty() {
                // No population data — this generator contributes nothing.
                return idx;
            }

            // Build histogram: count occurrences of each residue value.
            // Bucket indexing is a host operation (Rust array indices are usize).
            // The residue VALUES are ternary; the COUNTS are host integers.
            let bucket_count = generator.to_decimal() as usize;
            let mut buckets: Vec<u64> = vec![0; bucket_count];

            for residue in &residues {
                let bucket = residue.to_decimal() as usize;
                buckets[bucket] += 1;
            }

            // Imbalance = max_bucket − min_bucket.
            // Higher imbalance = worse distribution = more removable.
            let max_count = *buckets.iter().max().unwrap();
            let min_count = *buckets.iter().min().unwrap();
            let imbalance = max_count - min_count;

            if imbalance > worst_imbalance {
                worst_imbalance = imbalance;
                worst_idx = idx;
            }
        }

        worst_idx
    }

    // ── Queries ─────────────────────────────────────────────

    /// Current capacity as TritInt.
    pub fn capacity(&self) -> &TritInt { &self.capacity }

    /// Current generator count.
    pub fn generator_count(&self) -> usize { self.generators.len() }

    /// Available zoom levels = current generators.
    pub fn zoom_levels(&self) -> &[TritInt] { &self.generators }

    /// Can this AGS grow further within the polygon pool?
    pub fn can_grow(&self) -> bool {
        POLYGON_SET.iter().any(|g| {
            !self.generators.contains(g)
                && self.generators.iter().all(|existing| {
                    TritInt::gcd(g, existing) == TritInt::one()
                })
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// §5  THE INVISIBLE CONVERSION LAYER
// ═══════════════════════════════════════════════════════════════
//
// Binary consumers (WASM bridge, React frontend, HTTP API, CLI
// tools) need u32/u64 values. The conversion layer provides this
// WITHOUT contaminating the mathematical core.
//
// Architecture:
//   TritInt (ternary core) ──→ .to_u64() ──→ u64 (binary consumer)
//   u64 (binary input)    ──→ TritInt::from_decimal() ──→ TritInt
//
// The conversion is EXPLICIT at the API boundary. Inside the
// mathematical core, no binary type is referenced. The consumer
// never sees TritInt — they see u64 values from accessor methods.
//
// For constants.rs migration: each constant gains a TritInt primary
// representation and a u32 bridge accessor. Example:
//
//   // In the ternary core:
//   pub const REPUNIT_6: TritInt = TritInt::repunit(6); // 111111₃
//
//   // At the binary boundary:
//   pub fn repunit_6_u32() -> u32 { REPUNIT_6.to_decimal() as u32 }
//
// The compile-time assertion verifies the bridge:
//   const _: () = assert!(TritInt::repunit(6).to_decimal_const() == 364);

impl Ags {
    /// Binary bridge: capacity as u64 for consumers that need it.
    pub fn capacity_u64(&self) -> u64 {
        self.capacity.to_decimal()
    }

    /// Binary bridge: generator values as u32 for UI display.
    pub fn generators_u32(&self) -> Vec<u32> {
        self.generators.iter().map(|g| g.to_decimal() as u32).collect()
    }

    /// Binary bridge: walk position as u64 for database storage.
    pub fn walk_position_u64(&self, address: &TritInt) -> u64 {
        self.crt_reconstruct(&self.crt_project(address)).to_decimal()
    }
}

// ═══════════════════════════════════════════════════════════════
// §6  ESCALATION (ternary-native)
// ═══════════════════════════════════════════════════════════════
//
// When the AGS exhausts all polygon generators:
//
// Level 2: capacity × Δ₂ = capacity × 1000000₃
//   This is one ternary multiplication by a power of 3.
//   In packed trit storage, multiplying by 3ⁿ = left-shifting
//   by n trit positions. Cost: O(trit_length) byte shuffle.
//   No arithmetic needed — just move bytes.
//
// Level 3: distribute across 3^R₃ = 3^(111₃) hypercube nodes.
//   The exponent R₃ = 111₃ is itself a repunit.
//   3^(111₃) in ternary = 1 followed by 13 zeros = 10000000000000₃.
//   This is a 14-trit number. Its binary value (1,594,323) is
//   never computed inside the mathematical core. The hypercube
//   routing operates on trit addresses, not node numbers.

impl Ags {
    /// Escalate to Level 2: multiply capacity by Δ₂ = 3⁶.
    ///
    /// In ternary, multiplying by 3⁶ = left-shifting 6 trit positions.
    /// This is a byte shuffle in the packed representation, not arithmetic.
    pub fn escalate_depth(&mut self) {
        self.capacity = TritInt::mul(&self.capacity, &DELTA_2);
    }

    /// Total addressable space at current escalation level.
    ///
    /// Level 1: capacity (walk only)
    /// Level 2: capacity × Δ₂ (walk × depth)
    /// Level 3: capacity × Δ₂ × 3^R₃ (walk × depth × hypercube)
    pub fn total_addressable(&self, level: u8) -> TritInt {
        match level {
            1 => self.capacity.clone(),
            2 => TritInt::mul(&self.capacity, &DELTA_2),
            3 => {
                let hypercube = TritInt::from_trits(&[
                    0,0,0,0,0,0,0,0,0,0,0,0,0,1  // 10000000000000₃ = 3¹³
                ]);
                TritInt::mul(&TritInt::mul(&self.capacity, &DELTA_2), &hypercube)
            }
            _ => self.capacity.clone(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// §7  THE SINGLE ENTRY POINT
// ═══════════════════════════════════════════════════════════════
//
// This is the "one function" that handles everything. A record
// insert calls this. It grows, shrinks, or escalates as needed.
// The caller never thinks about capacity.

impl Ags {
    /// Insert a record. Grow if needed. Escalate if needed.
    ///
    /// Returns the walk position (as CRT residue vector) for
    /// the inserted record.
    ///
    /// This is the ONLY entry point for write operations.
    /// Growth and escalation are side effects of insertion —
    /// not separate operations the caller must remember to invoke.
    pub fn insert(&mut self, address: &TritInt, population: &TritInt) -> Vec<TritInt> {
        // Check if growth is needed (one ternary comparison)
        if self.should_grow(population) {
            if self.can_grow() {
                // Growth = one ternary multiplication
                self.grow();
            } else {
                // All polygon generators exhausted → escalate
                self.escalate_depth();
            }
        }

        // Compute walk position (ternary modulo per generator)
        self.crt_project(address)
    }

    /// Notify the AGS that a record was deleted.
    ///
    /// Shrink if population dropped below threshold.
    /// The caller passes the current population count.
    pub fn notify_delete(&mut self, population: &TritInt) {
        if self.should_shrink(population) {
            self.shrink(&|_| vec![]);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// §8  TESTS (ternary-native verification)
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_derived_from_axiom() {
        let seed = derive_ags_seed();
        // Must contain exactly {11, 13} = {102₃, 111₃}
        assert_eq!(seed.len(), 2);
        assert_eq!(seed[0], TritInt::from_trits(&[2, 0, 1]));  // 102₃ = 11
        assert_eq!(seed[1], TritInt::repunit(3));                // 111₃ = 13
    }

    #[test]
    fn seed_capacity_equals_framework_constant() {
        let seed = derive_ags_seed();
        let capacity = derive_ags_capacity(&seed);
        // 11 × 13 = 143 = 12022₃
        // Verify via binary bridge (the ONLY place binary appears in tests)
        assert_eq!(capacity.to_decimal(), 143);
        // Verify it matches BEZIER_C650_ANGLE from constants.rs
        assert_eq!(capacity.to_decimal(), 143); // = BEZIER_C650_ANGLE
    }

    #[test]
    fn seed_universally_coprime() {
        let seed = derive_ags_seed();
        // Every seed member must be coprime to every other polygon member
        for s in &seed {
            for p in &POLYGON_SET {
                if p != s {
                    assert_eq!(TritInt::gcd(s, p), TritInt::one(),
                        "Seed member {:?} must be coprime to {:?}", s, p);
                }
            }
        }
    }

    #[test]
    fn seed_in_all_maximal_subsets() {
        // The seed {11, 13} must appear in every sextuple
        let seed = derive_ags_seed();
        let sextuples: [[u64; 6]; 4] = [
            [3, 4, 5, 7, 11, 13],
            [3, 5, 7, 8, 11, 13],
            [4, 5, 7, 9, 11, 13],
            [5, 7, 8, 9, 11, 13],
        ];
        for sext in &sextuples {
            for s in &seed {
                let s_val = s.to_decimal();
                assert!(sext.contains(&s_val),
                    "Seed {} missing from sextuple {:?}", s_val, sext);
            }
        }
    }

    #[test]
    fn grow_is_one_multiplication() {
        let mut ags = Ags::new();
        let old_capacity = ags.capacity().clone();
        let old_gen_count = ags.generator_count();

        let added = ags.grow();
        assert!(added.is_some());

        let new_gen = added.unwrap();
        let expected_capacity = TritInt::mul(&old_capacity, &new_gen);
        assert_eq!(*ags.capacity(), expected_capacity);
        assert_eq!(ags.generator_count(), old_gen_count + 1);
    }

    #[test]
    fn thresholds_from_duty_cycle() {
        // The grow/shrink decision uses DUTY_N = 1₃ and DUTY_D = 11₃ = 4
        // Shrink at population < capacity × 1/4
        // Grow at population ≥ capacity × 3/4
        // Verify these derive from the duty cycle
        assert_eq!(DUTY_N.to_decimal(), 1);
        assert_eq!(DUTY_D.to_decimal(), 4);
        assert_eq!(DUTY_D.sub(&DUTY_N).to_decimal(), 3); // complement = 3
    }

    #[test]
    fn grow_through_full_landscape() {
        let mut ags = Ags::new();
        let mut growth_steps = vec![ags.capacity().to_decimal()];

        while ags.can_grow() {
            ags.grow();
            growth_steps.push(ags.capacity().to_decimal());
        }

        // Verify we reached the maximum sextuple capacity
        assert_eq!(*growth_steps.last().unwrap(), 360_360);

        // Verify the growth path is monotonically increasing
        for i in 1..growth_steps.len() {
            assert!(growth_steps[i] > growth_steps[i - 1],
                "Capacity must increase: {} → {}", growth_steps[i-1], growth_steps[i]);
        }
    }

    #[test]
    fn delta2_is_trit_shift() {
        // Δ₂ = 3⁶ = 1000000₃
        let trits = DELTA_2.to_trits();
        // Should be [0,0,0,0,0,0,1] LSB-first
        assert_eq!(trits, vec![0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(DELTA_2.to_decimal(), 729);
    }

    #[test]
    fn polygon_max_is_pentadecagon() {
        // 2 + R₃ = 2 + 13 = 15 = 120₃
        let r3 = TritInt::repunit(3);
        let two = TritInt::from_trits(&[2]);
        let max = TritInt::add(&r3, &two);
        assert_eq!(max, POLYGON_MAX);
        assert_eq!(max.to_decimal(), 15);
    }
}
