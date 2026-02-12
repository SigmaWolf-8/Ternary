//! # Ternary Circle — The Canonical Geometry of Base-3
//!
//! Re-founds circle geometry upon the ternary radix:
//!
//! - Full circle = 364° = (3⁶ − 1)/2 = 111111₃
//! - π_ternary = 14
//! - 1 radian = 13° = T(7) = 111₃ (Tribonacci repunit)
//! - Full circle = 28 radians ⇒ cyclic group Z₂₈
//!
//! The Tribonacci word, walked in these radians, draws a quasicrystalline
//! spiral — the Rauzy fractal reborn in the canonical ternary circle.
//!
//! ## Key types
//!
//! - [`TernaryAngle`]: An angle in ternary degrees (mod 364)
//! - [`TernaryRadian`]: An angle measured in ternary radians (mod 28)
//! - [`TribonacciWord`]: Infinite Tribonacci substitution sequence generator
//! - [`TribonacciSpiral`]: The Tribonacci radian spiral point set

use crate::gf3::Gf3;
use std::fmt;

pub const TERNARY_CIRCLE_DEGREES: u32 = 364;
pub const TERNARY_PI: u32 = 14;
pub const TERNARY_RADIAN_DEG: u32 = 13;
pub const FULL_CIRCLE_RADIANS: u32 = 28;
pub const TAU_TRIBONACCI: f64 = 1.8392867552141612;
pub const TRIBONACCI_GOLDEN_ANGLE_DEG: f64 = 58.50637090738759;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TernaryAngle {
    degrees: u32,
}

impl TernaryAngle {
    pub fn new(degrees: u32) -> Self {
        TernaryAngle { degrees: degrees % TERNARY_CIRCLE_DEGREES }
    }

    pub fn zero() -> Self {
        TernaryAngle { degrees: 0 }
    }

    pub fn from_radians(radians: u32) -> Self {
        TernaryAngle::new(radians * TERNARY_RADIAN_DEG)
    }

    pub fn degrees(&self) -> u32 {
        self.degrees
    }

    pub fn to_radians_exact(&self) -> Option<u32> {
        if self.degrees % TERNARY_RADIAN_DEG == 0 {
            Some(self.degrees / TERNARY_RADIAN_DEG)
        } else {
            None
        }
    }

    pub fn to_radians_f64(&self) -> f64 {
        self.degrees as f64 / TERNARY_RADIAN_DEG as f64
    }

    pub fn to_standard_radians(&self) -> f64 {
        self.degrees as f64 * std::f64::consts::PI / 180.0
    }

    pub fn add(self, other: TernaryAngle) -> TernaryAngle {
        TernaryAngle::new(self.degrees + other.degrees)
    }

    pub fn sub(self, other: TernaryAngle) -> TernaryAngle {
        TernaryAngle::new(self.degrees + TERNARY_CIRCLE_DEGREES - other.degrees)
    }

    pub fn negate(self) -> TernaryAngle {
        TernaryAngle::new(TERNARY_CIRCLE_DEGREES - self.degrees)
    }

    pub fn scale(self, n: u32) -> TernaryAngle {
        TernaryAngle::new(self.degrees * n)
    }

    pub fn to_base3_digits(&self) -> Vec<u8> {
        if self.degrees == 0 {
            return vec![0];
        }
        let mut n = self.degrees;
        let mut digits = Vec::new();
        while n > 0 {
            digits.push((n % 3) as u8);
            n /= 3;
        }
        digits.reverse();
        digits
    }

    pub fn cos(&self) -> f64 {
        self.to_standard_radians().cos()
    }

    pub fn sin(&self) -> f64 {
        self.to_standard_radians().sin()
    }
}

impl fmt::Debug for TernaryAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°₃", self.degrees)
    }
}

impl fmt::Display for TernaryAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°₃", self.degrees)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TernaryRadian {
    value: u32,
}

impl TernaryRadian {
    pub fn new(value: u32) -> Self {
        TernaryRadian { value: value % FULL_CIRCLE_RADIANS }
    }

    pub fn zero() -> Self {
        TernaryRadian { value: 0 }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn to_angle(&self) -> TernaryAngle {
        TernaryAngle::from_radians(self.value)
    }

    pub fn add(self, other: TernaryRadian) -> TernaryRadian {
        TernaryRadian::new(self.value + other.value)
    }

    pub fn sub(self, other: TernaryRadian) -> TernaryRadian {
        TernaryRadian::new(self.value + FULL_CIRCLE_RADIANS - other.value)
    }

    pub fn negate(self) -> TernaryRadian {
        TernaryRadian::new(FULL_CIRCLE_RADIANS - self.value)
    }

    pub fn scale(self, n: u32) -> TernaryRadian {
        TernaryRadian::new(self.value * n)
    }
}

impl fmt::Debug for TernaryRadian {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} rad₃", self.value)
    }
}

impl fmt::Display for TernaryRadian {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} rad₃", self.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TribonacciSymbol {
    A = 0,
    B = 1,
    C = 2,
}

impl TribonacciSymbol {
    pub fn to_gf3(self) -> Gf3 {
        Gf3::new(self as u8)
    }

    pub fn to_radian(self) -> TernaryRadian {
        TernaryRadian::new(self as u32)
    }

    pub fn to_angle(self) -> TernaryAngle {
        TernaryAngle::from_radians(self as u32)
    }

    pub fn substitute(self) -> Vec<TribonacciSymbol> {
        match self {
            TribonacciSymbol::A => vec![TribonacciSymbol::A, TribonacciSymbol::B],
            TribonacciSymbol::B => vec![TribonacciSymbol::A, TribonacciSymbol::C],
            TribonacciSymbol::C => vec![TribonacciSymbol::A],
        }
    }

    pub fn from_gf3(g: Gf3) -> Self {
        match g.value() {
            0 => TribonacciSymbol::A,
            1 => TribonacciSymbol::B,
            2 => TribonacciSymbol::C,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for TribonacciSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TribonacciSymbol::A => write!(f, "A"),
            TribonacciSymbol::B => write!(f, "B"),
            TribonacciSymbol::C => write!(f, "C"),
        }
    }
}

pub struct TribonacciWord {
    buffer: Vec<TribonacciSymbol>,
    generation: usize,
}

impl TribonacciWord {
    pub fn new() -> Self {
        TribonacciWord {
            buffer: vec![TribonacciSymbol::A],
            generation: 0,
        }
    }

    pub fn grow(&mut self) {
        let mut next = Vec::with_capacity(self.buffer.len() * 2);
        for &sym in &self.buffer {
            next.extend(sym.substitute());
        }
        self.buffer = next;
        self.generation += 1;
    }

    pub fn grow_to_length(&mut self, min_length: usize) {
        while self.buffer.len() < min_length {
            self.grow();
        }
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<TribonacciSymbol> {
        self.buffer.get(index).copied()
    }

    pub fn symbols(&self) -> &[TribonacciSymbol] {
        &self.buffer
    }

    pub fn to_gf3_vec(&self) -> Vec<Gf3> {
        self.buffer.iter().map(|s| s.to_gf3()).collect()
    }

    pub fn symbol_counts(&self) -> (usize, usize, usize) {
        let mut a = 0;
        let mut b = 0;
        let mut c = 0;
        for &s in &self.buffer {
            match s {
                TribonacciSymbol::A => a += 1,
                TribonacciSymbol::B => b += 1,
                TribonacciSymbol::C => c += 1,
            }
        }
        (a, b, c)
    }
}

impl Default for TribonacciWord {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct SpiralPoint {
    pub x: f64,
    pub y: f64,
    pub angle_deg: u32,
    pub radius_scale: f64,
    pub symbol: TribonacciSymbol,
    pub step: usize,
}

pub struct TribonacciSpiral {
    points: Vec<SpiralPoint>,
}

impl TribonacciSpiral {
    pub fn generate(num_steps: usize) -> Self {
        let mut word = TribonacciWord::new();
        word.grow_to_length(num_steps);

        let mut points = Vec::with_capacity(num_steps);
        let mut x = 0.0_f64;
        let mut y = 0.0_f64;

        let tau = TAU_TRIBONACCI;

        for k in 0..num_steps {
            let symbol = word.get(k).unwrap_or(TribonacciSymbol::A);
            let angle = symbol.to_angle();
            let scale = tau.powi(-((k + 1) as i32));

            let dx = angle.cos() * scale;
            let dy = angle.sin() * scale;
            x += dx;
            y += dy;

            points.push(SpiralPoint {
                x,
                y,
                angle_deg: angle.degrees(),
                radius_scale: scale,
                symbol,
                step: k,
            });
        }

        TribonacciSpiral { points }
    }

    pub fn points(&self) -> &[SpiralPoint] {
        &self.points
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn bounding_box(&self) -> (f64, f64, f64, f64) {
        if self.points.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for p in &self.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        (min_x, min_y, max_x, max_y)
    }

    pub fn direction_histogram(&self) -> [usize; 28] {
        let mut hist = [0usize; 28];
        for p in &self.points {
            let radian_pos = (p.angle_deg / TERNARY_RADIAN_DEG) % FULL_CIRCLE_RADIANS;
            hist[radian_pos as usize] += 1;
        }
        hist
    }
}

pub struct Z28 {
    value: u32,
}

impl Z28 {
    pub fn new(value: u32) -> Self {
        Z28 { value: value % 28 }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn add(self, other: Z28) -> Z28 {
        Z28::new(self.value + other.value)
    }

    pub fn sub(self, other: Z28) -> Z28 {
        Z28::new(self.value + 28 - other.value)
    }

    pub fn negate(self) -> Z28 {
        Z28::new(28 - self.value)
    }

    pub fn scale(self, n: u32) -> Z28 {
        Z28::new(self.value * n)
    }

    pub fn to_angle(&self) -> TernaryAngle {
        TernaryAngle::from_radians(self.value)
    }

    pub fn to_radian(&self) -> TernaryRadian {
        TernaryRadian::new(self.value)
    }

    pub fn order(&self) -> u32 {
        if self.value == 0 {
            return 1;
        }
        let mut current = self.value;
        for k in 1..=28 {
            if current % 28 == 0 {
                return k;
            }
            current += self.value;
        }
        28
    }

    pub fn generates_group(&self) -> bool {
        self.order() == 28
    }

    pub fn all_generators() -> Vec<Z28> {
        (1..28).filter(|&v| Z28::new(v).generates_group()).map(Z28::new).collect()
    }

    pub fn orbit(&self) -> Vec<u32> {
        let mut result = Vec::new();
        let mut current = 0u32;
        for _ in 0..28 {
            result.push(current % 28);
            current += self.value;
            if current % 28 == 0 && !result.is_empty() {
                break;
            }
        }
        result
    }
}

impl fmt::Debug for Z28 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]₂₈", self.value)
    }
}

impl fmt::Display for Z28 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]₂₈", self.value)
    }
}

pub fn tribonacci_braid_word(length: usize) -> Vec<Z28> {
    let mut word = TribonacciWord::new();
    word.grow_to_length(length);

    let mut cumulative = 0u32;
    let mut braid = Vec::with_capacity(length);

    for k in 0..length {
        let sym = word.get(k).unwrap_or(TribonacciSymbol::A);
        cumulative = (cumulative + sym as u32) % 28;
        braid.push(Z28::new(cumulative));
    }

    braid
}

pub fn golden_angle_sequence(length: usize) -> Vec<TernaryAngle> {
    let golden_deg = TRIBONACCI_GOLDEN_ANGLE_DEG;
    (0..length)
        .map(|k| {
            let angle = (k as f64 * golden_deg) % (TERNARY_CIRCLE_DEGREES as f64);
            TernaryAngle::new(angle.round() as u32)
        })
        .collect()
}

pub fn ternary_circle_economy() -> (Vec<u8>, Vec<u8>) {
    let ternary_364 = {
        let mut n = 364u32;
        let mut digits = Vec::new();
        while n > 0 {
            digits.push((n % 3) as u8);
            n /= 3;
        }
        digits.reverse();
        digits
    };
    let ternary_360 = {
        let mut n = 360u32;
        let mut digits = Vec::new();
        while n > 0 {
            digits.push((n % 3) as u8);
            n /= 3;
        }
        digits.reverse();
        digits
    };
    (ternary_364, ternary_360)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_is_base3_repunit() {
        assert_eq!(TERNARY_CIRCLE_DEGREES, (3u32.pow(6) - 1) / 2);
        let mut n = 364u32;
        let mut digits = Vec::new();
        while n > 0 { digits.push((n % 3) as u8); n /= 3; }
        assert_eq!(digits.len(), 6, "364 should have exactly 6 base-3 digits");
        assert!(digits.iter().all(|&x| x == 1), "364 = 111111₃");
    }

    #[test]
    fn radian_is_tribonacci_repunit() {
        assert_eq!(TERNARY_RADIAN_DEG, 13);
        assert_eq!(TERNARY_RADIAN_DEG, 1 + 3 + 9);
    }

    #[test]
    fn full_circle_is_28_radians() {
        assert_eq!(TERNARY_CIRCLE_DEGREES / TERNARY_RADIAN_DEG, 28);
        assert_eq!(FULL_CIRCLE_RADIANS, 2 * TERNARY_PI);
    }

    #[test]
    fn angle_arithmetic_mod_364() {
        let a = TernaryAngle::new(300);
        let b = TernaryAngle::new(100);
        let sum = a.add(b);
        assert_eq!(sum.degrees(), (300 + 100) % 364);

        let c = TernaryAngle::new(0);
        let d = TernaryAngle::new(50);
        let diff = c.sub(d);
        assert_eq!(diff.degrees(), 364 - 50);
    }

    #[test]
    fn angle_negation() {
        let a = TernaryAngle::new(100);
        let neg = a.negate();
        assert_eq!(a.add(neg).degrees(), 0);
    }

    #[test]
    fn radian_arithmetic_mod_28() {
        let a = TernaryRadian::new(20);
        let b = TernaryRadian::new(15);
        assert_eq!(a.add(b).value(), (20 + 15) % 28);

        let c = TernaryRadian::new(5);
        let d = TernaryRadian::new(10);
        assert_eq!(c.sub(d).value(), (5 + 28 - 10) % 28);
    }

    #[test]
    fn radian_to_angle_conversion() {
        let r = TernaryRadian::new(1);
        assert_eq!(r.to_angle().degrees(), 13);

        let r2 = TernaryRadian::new(14);
        assert_eq!(r2.to_angle().degrees(), 14 * 13);
    }

    #[test]
    fn angle_to_radian_exact() {
        let a = TernaryAngle::new(26);
        assert_eq!(a.to_radians_exact(), Some(2));

        let b = TernaryAngle::new(27);
        assert_eq!(b.to_radians_exact(), None);
    }

    #[test]
    fn tribonacci_symbol_gf3_roundtrip() {
        for sym in [TribonacciSymbol::A, TribonacciSymbol::B, TribonacciSymbol::C] {
            let g = sym.to_gf3();
            let back = TribonacciSymbol::from_gf3(g);
            assert_eq!(sym, back);
        }
    }

    #[test]
    fn tribonacci_substitution_rules() {
        assert_eq!(
            TribonacciSymbol::A.substitute(),
            vec![TribonacciSymbol::A, TribonacciSymbol::B]
        );
        assert_eq!(
            TribonacciSymbol::B.substitute(),
            vec![TribonacciSymbol::A, TribonacciSymbol::C]
        );
        assert_eq!(
            TribonacciSymbol::C.substitute(),
            vec![TribonacciSymbol::A]
        );
    }

    #[test]
    fn tribonacci_word_growth() {
        let mut word = TribonacciWord::new();
        assert_eq!(word.len(), 1); // [A]
        word.grow(); // [A,B]
        assert_eq!(word.len(), 2);
        word.grow(); // [A,B,A,C]
        assert_eq!(word.len(), 4);
        word.grow(); // [A,B,A,C,A,B,A]
        assert_eq!(word.len(), 7);
        word.grow(); // length 13
        assert_eq!(word.len(), 13);
    }

    #[test]
    fn tribonacci_word_lengths_are_tribonacci_numbers() {
        let mut word = TribonacciWord::new();
        let expected_lengths = [1, 2, 4, 7, 13, 24, 44];
        for &expected in &expected_lengths {
            assert_eq!(word.len(), expected,
                "Gen {} should have length {}", word.generation(), expected);
            word.grow();
        }
    }

    #[test]
    fn tribonacci_word_symbol_ratios_converge_to_tau() {
        let mut word = TribonacciWord::new();
        word.grow_to_length(1000);
        let (a_count, b_count, c_count) = word.symbol_counts();
        let total = word.len() as f64;
        let a_ratio = a_count as f64 / total;
        let tau = TAU_TRIBONACCI;
        let expected_a_ratio = 1.0 / tau;
        assert!((a_ratio - expected_a_ratio).abs() < 0.01,
            "A-ratio {a_ratio} should approach 1/τ ≈ {expected_a_ratio}");
        assert!(b_count > 0 && c_count > 0);
    }

    #[test]
    fn spiral_generates_points() {
        let spiral = TribonacciSpiral::generate(100);
        assert_eq!(spiral.len(), 100);
        let (min_x, min_y, max_x, max_y) = spiral.bounding_box();
        assert!(max_x > min_x || max_y > min_y, "Spiral should have nonzero extent");
    }

    #[test]
    fn spiral_points_converge() {
        let spiral = TribonacciSpiral::generate(50);
        let points = spiral.points();
        let last = &points[points.len() - 1];
        let second_last = &points[points.len() - 2];
        let delta = ((last.x - second_last.x).powi(2) + (last.y - second_last.y).powi(2)).sqrt();
        assert!(delta < 1e-10, "Later spiral steps should be vanishingly small: {delta}");
    }

    #[test]
    fn spiral_uses_only_three_directions() {
        let spiral = TribonacciSpiral::generate(100);
        for p in spiral.points() {
            assert!(
                p.angle_deg == 0 || p.angle_deg == 13 || p.angle_deg == 26,
                "Angle {} is not a valid Tribonacci word direction", p.angle_deg
            );
        }
    }

    #[test]
    fn z28_group_arithmetic() {
        let a = Z28::new(10);
        let b = Z28::new(25);
        assert_eq!(a.add(b).value(), (10 + 25) % 28);
        assert_eq!(a.sub(b).value(), (10 + 28 - 25) % 28);
        assert_eq!(a.negate().add(a).value(), 0);
    }

    #[test]
    fn z28_identity() {
        let zero = Z28::new(0);
        assert_eq!(zero.order(), 1);
        let one = Z28::new(1);
        assert_eq!(one.order(), 28);
        assert!(one.generates_group());
    }

    #[test]
    fn z28_generators() {
        let gens = Z28::all_generators();
        assert!(!gens.is_empty());
        for g in &gens {
            assert_eq!(g.order(), 28, "[{}]₂₈ claims to generate but has order {}", g.value(), g.order());
        }
        let gen_values: Vec<u32> = gens.iter().map(|g| g.value()).collect();
        assert!(gen_values.contains(&1));
        assert!(gen_values.contains(&3));
        assert!(gen_values.contains(&5));
        assert!(gen_values.contains(&9));
        assert!(gen_values.contains(&11));
        assert!(gen_values.contains(&13));
    }

    #[test]
    fn z28_orbit_covers_group() {
        let g = Z28::new(1);
        let orbit = g.orbit();
        assert_eq!(orbit.len(), 28);
        let mut sorted = orbit.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 28, "Generator 1 should visit all 28 elements");
    }

    #[test]
    fn z28_non_generator_subgroups() {
        let g = Z28::new(7);
        assert_eq!(g.order(), 4); // 7 generates Z₄ subgroup: {0, 7, 14, 21}
        assert!(!g.generates_group());

        let g2 = Z28::new(14);
        assert_eq!(g2.order(), 2); // {0, 14}
    }

    #[test]
    fn tribonacci_braid_word_mod_28() {
        let braid = tribonacci_braid_word(100);
        assert_eq!(braid.len(), 100);
        for z in &braid {
            assert!(z.value() < 28);
        }
    }

    #[test]
    fn golden_angle_sequence_mod_364() {
        let seq = golden_angle_sequence(50);
        assert_eq!(seq.len(), 50);
        for a in &seq {
            assert!(a.degrees() < TERNARY_CIRCLE_DEGREES,
                "Angle {} should be < 364", a.degrees());
        }
    }

    #[test]
    fn ternary_circle_economy_comparison() {
        let (rep364, rep360) = ternary_circle_economy();
        assert!(rep364.iter().all(|&d| d == 1),
            "364 in base 3 should be all 1s: {:?}", rep364);
        assert!(!rep360.iter().all(|&d| d == 1),
            "360 in base 3 should NOT be all 1s: {:?}", rep360);
    }

    #[test]
    fn tribonacci_golden_angle_value() {
        let expected = TERNARY_CIRCLE_DEGREES as f64 / (TAU_TRIBONACCI.powi(3));
        assert!((TRIBONACCI_GOLDEN_ANGLE_DEG - expected).abs() < 1e-6);
    }

    #[test]
    fn angle_from_radians_roundtrip() {
        for r in 0..FULL_CIRCLE_RADIANS {
            let angle = TernaryAngle::from_radians(r);
            let back = angle.to_radians_exact();
            assert_eq!(back, Some(r), "Radian {} should roundtrip through angle", r);
        }
    }
}
