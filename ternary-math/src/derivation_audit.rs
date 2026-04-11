// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// derivation_audit.rs — Single-axiom audit trail
//
// Every constant in the framework is COMPUTED from R_n = (3^n - 1) / 2.
// Nothing is hardcoded. The only inputs are the base (3) and the exponents.
//
// API:
//   DerivationGraph::new() — computes all constants from the axiom
//   .trace(name)           — derivation chain back to R_n
//   .trace_value(value)    — find all constants with that value, trace each
//   .format(name)          — human-readable audit output
//   .verify()              — recompute everything, confirm consistency
//
// Zero magic numbers. Zero external dependencies. One axiom.

/// The single axiom: R_n = (3^n - 1) / 2
pub fn repunit(n: u32) -> i64 {
    (3i64.pow(n) - 1) / 2
}

/// Integer square root. Returns None if not a perfect square.
fn isqrt(n: i64) -> Option<i64> {
    if n < 0 {
        return None;
    }
    let s = (n as f64).sqrt() as i64;
    // Check s-1, s, s+1 to handle floating-point imprecision
    for candidate in [s - 1, s, s + 1] {
        if candidate >= 0 && candidate * candidate == n {
            return Some(candidate);
        }
    }
    None
}

/// Integer GCD (Euclidean).
fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// A node in the derivation graph.
#[derive(Debug, Clone)]
pub struct DerivedConstant {
    pub name: &'static str,
    pub value: i64,
    pub derivation: String,
    pub depends_on: Vec<&'static str>,
}

/// The complete derivation graph, computed from the axiom.
pub struct DerivationGraph {
    pub constants: Vec<DerivedConstant>,
}

impl DerivationGraph {
    /// Build the full derivation graph. Every value is COMPUTED, not assigned.
    pub fn new() -> Self {
        let mut constants = Vec::new();

        // ── AXIOM ──
        constants.push(DerivedConstant {
            name: "R_n",
            value: 0,
            derivation: "AXIOM: R_n = (3^n - 1) / 2".into(),
            depends_on: vec![],
        });

        // ── REPUNIT INSTANCES ──
        let r3 = repunit(3);
        let r4 = repunit(4);
        let r5 = repunit(5);
        let r6 = repunit(6);

        constants.push(DerivedConstant {
            name: "R_3",
            value: r3,
            derivation: format!("R_3 = (3^3 - 1) / 2 = {} [radian unit]", r3),
            depends_on: vec!["R_n"],
        });
        constants.push(DerivedConstant {
            name: "R_4",
            value: r4,
            derivation: format!("R_4 = (3^4 - 1) / 2 = {} [quadratic sum]", r4),
            depends_on: vec!["R_n"],
        });
        constants.push(DerivedConstant {
            name: "R_5",
            value: r5,
            derivation: format!("R_5 = (3^5 - 1) / 2 = {} [triangle step]", r5),
            depends_on: vec!["R_n"],
        });
        constants.push(DerivedConstant {
            name: "R_6",
            value: r6,
            derivation: format!("R_6 = (3^6 - 1) / 2 = {} [full circle]", r6),
            depends_on: vec!["R_n"],
        });

        // ── CIRCLE QUADRATIC: x² - R_4·x + R_6 = 0 ──
        let delta = r4 * r4 - 4 * r6;
        let sqrt_delta = isqrt(delta)
            .expect("Circle quadratic discriminant must be a perfect square");

        constants.push(DerivedConstant {
            name: "DELTA",
            value: delta,
            derivation: format!(
                "Δ = R_4² - 4·R_6 = {} - {} = {} = {}²",
                r4 * r4, 4 * r6, delta, sqrt_delta
            ),
            depends_on: vec!["R_4", "R_6"],
        });
        constants.push(DerivedConstant {
            name: "SQRT_DELTA",
            value: sqrt_delta,
            derivation: format!("√Δ = √{} = {} [root spread, amplitude ratio]", delta, sqrt_delta),
            depends_on: vec!["DELTA"],
        });

        let pi = (r4 - sqrt_delta) / 2;
        let x2 = (r4 + sqrt_delta) / 2;

        constants.push(DerivedConstant {
            name: "PI",
            value: pi,
            derivation: format!("π = (R_4 - √Δ) / 2 = ({} - {}) / 2 = {} [smaller root]", r4, sqrt_delta, pi),
            depends_on: vec!["R_4", "SQRT_DELTA"],
        });
        constants.push(DerivedConstant {
            name: "X_2",
            value: x2,
            derivation: format!("x₂ = (R_4 + √Δ) / 2 = ({} + {}) / 2 = {} [larger root]", r4, sqrt_delta, x2),
            depends_on: vec!["R_4", "SQRT_DELTA"],
        });

        // ── UNIFIED EQUATION: arc² + b·arc + c = 0 ──
        let linear = 2 * r6 - r4 * (r4 - 1);
        let constant_term = r6 * (r6 - r4 + 1);

        constants.push(DerivedConstant {
            name: "LINEAR_COEFF",
            value: linear,
            derivation: format!("b = 2·R_6 - R_4·(R_4 - 1) = {} - {} = {}", 2 * r6, r4 * (r4 - 1), linear),
            depends_on: vec!["R_4", "R_6"],
        });
        constants.push(DerivedConstant {
            name: "CONSTANT_TERM",
            value: constant_term,
            derivation: format!("c = R_6·(R_6 - R_4 + 1) = {} × {} = {}", r6, r6 - r4 + 1, constant_term),
            depends_on: vec!["R_4", "R_6"],
        });

        let arc_disc = linear * linear - 4 * constant_term;
        let arc_sqrt = isqrt(arc_disc)
            .expect("Unified equation discriminant must be a perfect square");

        let arc1 = (-linear - arc_sqrt) / 2;
        let arc2 = (-linear + arc_sqrt) / 2;

        constants.push(DerivedConstant {
            name: "ARC_1",
            value: arc1,
            derivation: format!("arc₁ = (-b - √(b²-4c)) / 2 = {} [semicircle, π(π-1)]", arc1),
            depends_on: vec!["LINEAR_COEFF", "CONSTANT_TERM"],
        });
        constants.push(DerivedConstant {
            name: "ARC_2",
            value: arc2,
            derivation: format!("arc₂ = (-b + √(b²-4c)) / 2 = {} [complementary arc]", arc2),
            depends_on: vec!["LINEAR_COEFF", "CONSTANT_TERM"],
        });

        // ── SECONDARY DISCRIMINANT ──
        let delta2 = 1 + 4 * arc1;
        let sqrt_delta2 = isqrt(delta2)
            .expect("Secondary discriminant must be a perfect square");
        let delta2_exp = (0u32..20).find(|&k| 3i64.pow(k) == delta2).unwrap_or(0);

        constants.push(DerivedConstant {
            name: "DELTA_2",
            value: delta2,
            derivation: format!("Δ₂ = 1 + 4·arc₁ = {} = {}² = 3^{} [sponge width]", delta2, sqrt_delta2, delta2_exp),
            depends_on: vec!["ARC_1"],
        });

        // Verify π recovery: x² - x - arc1 = 0 → x = (1 + √Δ₂) / 2
        let pi_recovered = (1 + sqrt_delta2) / 2;
        assert_eq!(pi_recovered, pi, "π recovery: (1 + √Δ₂)/2 = {} ≠ π = {}", pi_recovered, pi);

        // ── DERIVED ANGULAR CONSTANTS ──
        let quarter_turn = arc1 / 2;
        let green_eff = ((arc2 % r6) + r6) % r6;
        let center = (arc1 + r4) / 2;

        constants.push(DerivedConstant {
            name: "QUARTER_TURN",
            value: quarter_turn,
            derivation: format!("{} = arc₁ / 2 = {} / 2 [C₁₈₂ angle, ionization threshold]", quarter_turn, arc1),
            depends_on: vec!["ARC_1"],
        });
        constants.push(DerivedConstant {
            name: "GREEN_ARC_EFF",
            value: green_eff,
            derivation: format!("{} = arc₂ mod R_6 = {} mod {} [UV-B boundary]", green_eff, arc2, r6),
            depends_on: vec!["ARC_2", "R_6"],
        });
        constants.push(DerivedConstant {
            name: "CENTER",
            value: center,
            derivation: format!("{} = (arc₁ + R_4) / 2 = ({} + {}) / 2 [diameter]", center, arc1, r4),
            depends_on: vec!["ARC_1", "R_4"],
        });

        // ── COPRIME STRUCTURE ──
        let arc_gcd = gcd(arc1, green_eff);
        let seven = arc1 / arc_gcd;
        let eleven = green_eff / arc_gcd;

        constants.push(DerivedConstant {
            name: "ARC_RATIO_GCD",
            value: arc_gcd,
            derivation: format!("gcd({}, {}) = {} = 2 × R_3", arc1, green_eff, arc_gcd),
            depends_on: vec!["ARC_1", "GREEN_ARC_EFF"],
        });
        constants.push(DerivedConstant {
            name: "SEVEN",
            value: seven,
            derivation: format!("{} = arc₁ / gcd = {} / {} [red arc reduced]", seven, arc1, arc_gcd),
            depends_on: vec!["ARC_1", "ARC_RATIO_GCD"],
        });
        constants.push(DerivedConstant {
            name: "ELEVEN",
            value: eleven,
            derivation: format!("{} = green_eff / gcd = {} / {} [green arc reduced]", eleven, green_eff, arc_gcd),
            depends_on: vec!["GREEN_ARC_EFF", "ARC_RATIO_GCD"],
        });

        // Verify pairwise coprimality
        assert_eq!(gcd(seven, eleven), 1, "{} and {} must be coprime", seven, eleven);
        assert_eq!(gcd(seven, r3), 1, "{} and {} must be coprime", seven, r3);
        assert_eq!(gcd(eleven, r3), 1, "{} and {} must be coprime", eleven, r3);

        let coprime_lcm = seven * eleven * r3;

        constants.push(DerivedConstant {
            name: "COPRIME_LCM",
            value: coprime_lcm,
            derivation: format!("lcm({},{},{}) = {} [Hamiltonian cycle]", seven, eleven, r3, coprime_lcm),
            depends_on: vec!["SEVEN", "ELEVEN", "R_3"],
        });

        // Pentadecagon: R_3 polygons inscribed (n=3..n=3+R_3-1), last = 3+R_3-1
        let fifteen = 3 + r3 - 1;

        constants.push(DerivedConstant {
            name: "FIFTEEN",
            value: fifteen,
            derivation: format!("{} = 3 + R_3 - 1 = last inscribed polygon [{} total]", fifteen, r3),
            depends_on: vec!["R_3"],
        });

        let quadruple_lcm = fifteen * coprime_lcm;

        constants.push(DerivedConstant {
            name: "QUADRUPLE_LCM",
            value: quadruple_lcm,
            derivation: format!("lcm({},{},{},{}) = {} × {} = {}", seven, eleven, r3, fifteen, fifteen, coprime_lcm, quadruple_lcm),
            depends_on: vec!["COPRIME_LCM", "FIFTEEN"],
        });

        // ── HMODAL SIGNALING ──
        let alpha_num = r6 / gcd(r6, delta);
        let alpha_den = delta / gcd(r6, delta);

        constants.push(DerivedConstant {
            name: "ALPHA_FRAC",
            value: alpha_num,
            derivation: format!("α = R_6 / Δ = {} / {} = {}/{} [idle amplitude]", r6, delta, alpha_num, alpha_den),
            depends_on: vec!["R_6", "DELTA"],
        });

        let dc_raw_num = r6 + coprime_lcm;
        let dc_gcd = gcd(dc_raw_num, delta);
        let dc_num = dc_raw_num / dc_gcd;
        let dc_den = delta / dc_gcd;

        constants.push(DerivedConstant {
            name: "DC_NUM",
            value: dc_num,
            derivation: format!("⟨H⟩ = (R_6 + lcm) / Δ = ({} + {}) / {} → {}/{}", r6, coprime_lcm, delta, dc_num, dc_den),
            depends_on: vec!["R_6", "COPRIME_LCM", "DELTA"],
        });

        // ── UV SPECTRAL ──
        constants.push(DerivedConstant {
            name: "UV_EUV",
            value: quarter_turn,
            derivation: format!("{} nm = quarter-turn [H/O ionization threshold]", quarter_turn),
            depends_on: vec!["QUARTER_TURN"],
        });
        constants.push(DerivedConstant {
            name: "UV_C",
            value: arc1,
            derivation: format!("{} nm = half-turn = π × radian [O₂ wall]", arc1),
            depends_on: vec!["ARC_1"],
        });
        constants.push(DerivedConstant {
            name: "UV_B",
            value: green_eff,
            derivation: format!(
                "{} nm = green arc [{}/{} = {}/{}] [O₃ bridge]",
                green_eff, green_eff, quarter_turn,
                green_eff / gcd(green_eff, quarter_turn),
                quarter_turn / gcd(green_eff, quarter_turn)
            ),
            depends_on: vec!["GREEN_ARC_EFF"],
        });
        constants.push(DerivedConstant {
            name: "UV_A",
            value: r6,
            derivation: format!("{} nm = full circle [atmospheric transmission]", r6),
            depends_on: vec!["R_6"],
        });

        // ── SEIFERT GENUS ──
        let genus = (pi - 1) * (fifteen - 1) / 2;

        constants.push(DerivedConstant {
            name: "GENUS_T_PI_15",
            value: genus,
            derivation: format!(
                "g(T({},{})) = ({}-1)({}-1)/2 = {}×{}/2 = {} [= quarter-turn]",
                pi, fifteen, pi, fifteen, pi - 1, fifteen - 1, genus
            ),
            depends_on: vec!["PI", "FIFTEEN"],
        });

        // ── HYPERBOLICITY ──
        let hyp_num = eleven * r3 + seven * r3 + seven * eleven;
        let hyp_den = seven * eleven * r3;

        constants.push(DerivedConstant {
            name: "HYPERBOLICITY",
            value: hyp_num,
            derivation: format!(
                "1/{} + 1/{} + 1/{} = {}/{} ≈ {:.4} < 1 [hyperbolic Coxeter]",
                seven, eleven, r3, hyp_num, hyp_den, hyp_num as f64 / hyp_den as f64
            ),
            depends_on: vec!["SEVEN", "ELEVEN", "R_3"],
        });

        assert!(hyp_num < hyp_den, "Hyperbolicity: {}/{} ≥ 1", hyp_num, hyp_den);
        assert_eq!(hyp_den, coprime_lcm, "Denominator must equal coprime walk");

        DerivationGraph { constants }
    }

    /// Find all constants with a given value.
    pub fn find_by_value(&self, value: i64) -> Vec<&DerivedConstant> {
        self.constants.iter().filter(|c| c.value == value).collect()
    }

    /// Find a constant by name.
    pub fn find_by_name(&self, name: &str) -> Option<&DerivedConstant> {
        self.constants.iter().find(|c| c.name == name)
    }

    /// Trace a constant back to the axiom (dependencies first, axiom at index 0).
    pub fn trace(&self, name: &str) -> Vec<&DerivedConstant> {
        let mut chain = Vec::new();
        let mut visited = Vec::new();
        self.collect(name, &mut chain, &mut visited);
        chain
    }

    /// Trace all constants matching a value.
    pub fn trace_value(&self, value: i64) -> Vec<Vec<&DerivedConstant>> {
        self.find_by_value(value)
            .iter()
            .map(|c| self.trace(c.name))
            .collect()
    }

    fn collect<'a>(
        &'a self,
        name: &str,
        chain: &mut Vec<&'a DerivedConstant>,
        visited: &mut Vec<String>,
    ) {
        if visited.contains(&name.to_string()) {
            return;
        }
        if let Some(c) = self.find_by_name(name) {
            visited.push(name.to_string());
            for dep in &c.depends_on {
                self.collect(dep, chain, visited);
            }
            chain.push(c);
        }
    }

    /// Human-readable derivation chain for auditors.
    pub fn format(&self, name: &str) -> String {
        let chain = self.trace(name);
        if chain.is_empty() {
            return format!("Unknown constant: {}", name);
        }

        let target = chain.last().unwrap();
        let mut out = format!("=== Derivation: {} = {} ===\n\n", target.name, target.value);

        for (i, step) in chain.iter().enumerate() {
            let tag = if i == 0 { "AXIOM".to_string() } else { format!("#{}", i) };
            out.push_str(&format!("{:>7} │ {}\n", tag, step.derivation));
            if !step.depends_on.is_empty() {
                out.push_str(&format!("        │ ← {}\n", step.depends_on.join(", ")));
            }
            out.push('\n');
        }
        out
    }

    /// Verify internal consistency by checking algebraic identities.
    /// Returns Ok(check_count) or Err(list of failures).
    pub fn verify(&self) -> Result<usize, Vec<String>> {
        let mut checks = 0usize;
        let mut errors = Vec::new();

        let v = |name: &str| -> i64 {
            self.find_by_name(name).map(|c| c.value).unwrap_or(-999999)
        };

        // Vieta's formulas
        Self::eq("Vieta sum", v("PI") + v("X_2"), v("R_4"), &mut checks, &mut errors);
        Self::eq("Vieta product", v("PI") * v("X_2"), v("R_6"), &mut checks, &mut errors);

        // Roots satisfy circle quadratic
        let pi = v("PI"); let r4 = v("R_4"); let r6 = v("R_6");
        Self::eq("π in quadratic", pi * pi - r4 * pi + r6, 0, &mut checks, &mut errors);
        Self::eq("x₂ in quadratic", v("X_2") * v("X_2") - r4 * v("X_2") + r6, 0, &mut checks, &mut errors);

        // Roots satisfy unified equation
        let b = v("LINEAR_COEFF"); let c = v("CONSTANT_TERM");
        let a1 = v("ARC_1"); let a2 = v("ARC_2");
        Self::eq("arc₁ in unified", a1 * a1 + b * a1 + c, 0, &mut checks, &mut errors);
        Self::eq("arc₂ in unified", a2 * a2 + b * a2 + c, 0, &mut checks, &mut errors);

        // π recovery: π² - π - arc₁ = 0
        Self::eq("π recovery", pi * pi - pi - a1, 0, &mut checks, &mut errors);

        // arc₁ = π(π-1)
        Self::eq("arc₁ = π(π-1)", a1, pi * (pi - 1), &mut checks, &mut errors);

        // Δ₂ = 3^6 (computed, not hardcoded)
        let r3 = v("R_3");
        let base_cubed = r3 * 2 + 1; // R_3 = (3^3-1)/2, so 3^3 = 2·R_3+1
        let three_to_six = base_cubed * base_cubed; // (3^3)^2 = 3^6
        Self::eq("Δ₂ = (2·R_3+1)²", v("DELTA_2"), three_to_six, &mut checks, &mut errors);

        // Quarter turn
        Self::eq("quarter = arc/2", v("QUARTER_TURN"), a1 / 2, &mut checks, &mut errors);

        // Coprime walk
        Self::eq("lcm product", v("COPRIME_LCM"), v("SEVEN") * v("ELEVEN") * r3, &mut checks, &mut errors);

        // Pairwise coprimality
        Self::eq("gcd(7,11)=1", gcd(v("SEVEN"), v("ELEVEN")), 1, &mut checks, &mut errors);
        Self::eq("gcd(7,R_3)=1", gcd(v("SEVEN"), r3), 1, &mut checks, &mut errors);
        Self::eq("gcd(11,R_3)=1", gcd(v("ELEVEN"), r3), 1, &mut checks, &mut errors);

        // Hyperbolicity
        let hyp = v("HYPERBOLICITY");
        let lcm = v("COPRIME_LCM");
        if hyp < lcm { checks += 1; } else {
            errors.push(format!("FAIL [hyperbolicity]: {}/{} ≥ 1", hyp, lcm));
        }

        // Seifert genus = quarter turn
        Self::eq("genus = quarter", v("GENUS_T_PI_15"), v("QUARTER_TURN"), &mut checks, &mut errors);

        // UV ratios (all derived, no literals)
        let qt = v("QUARTER_TURN");
        Self::eq("UV: arc₁ = 2·qt", a1, 2 * qt, &mut checks, &mut errors);
        Self::eq("UV: R_6 = 4·qt", r6, 4 * qt, &mut checks, &mut errors);
        // 22/7 check via cross-multiplication: green × seven = qt × (2·eleven)
        let seven = v("SEVEN"); let eleven = v("ELEVEN");
        Self::eq("UV: green×7 = qt×22", v("GREEN_ARC_EFF") * seven, qt * 2 * eleven, &mut checks, &mut errors);

        // DC = 5 × seven × R_3 (factor 5 uninvited)
        // Derive the 5: dc_num / (seven × R_3)
        let dc = v("DC_NUM");
        let expected_cofactor = dc / (seven * r3);
        Self::eq("DC cofactor", dc, expected_cofactor * seven * r3, &mut checks, &mut errors);

        // Full circle in custom radians
        Self::eq("2π·radian = R_6", 2 * pi * r3, r6, &mut checks, &mut errors);

        if errors.is_empty() { Ok(checks) } else { Err(errors) }
    }

    fn eq(label: &str, got: i64, expected: i64, checks: &mut usize, errors: &mut Vec<String>) {
        if got == expected {
            *checks += 1;
        } else {
            errors.push(format!("FAIL [{}]: got {} expected {}", label, got, expected));
        }
    }
}

impl Default for DerivationGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ─── TESTS ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_builds_from_axiom() {
        let g = DerivationGraph::new();
        assert!(g.constants.len() > 25);
    }

    #[test]
    fn test_verify_all_passes() {
        let g = DerivationGraph::new();
        match g.verify() {
            Ok(n) => assert!(n >= 20, "Expected ≥20 checks, got {}", n),
            Err(e) => panic!("Verification failed:\n{}", e.join("\n")),
        }
    }

    #[test]
    fn test_values_are_computed_not_assigned() {
        let g = DerivationGraph::new();
        let v = |name: &str| g.find_by_name(name).unwrap().value;
        // These are OUTPUTS — if you change repunit(), the graph recomputes
        assert_eq!(v("R_3"), repunit(3));
        assert_eq!(v("R_6"), repunit(6));
        assert_eq!(v("PI") + v("X_2"), v("R_4"));
        assert_eq!(v("PI") * v("X_2"), v("R_6"));
        assert_eq!(v("ARC_1"), v("PI") * (v("PI") - 1));
        assert_eq!(v("DELTA_2"), 1 + 4 * v("ARC_1"));
    }

    #[test]
    fn test_every_constant_traces_to_axiom() {
        let g = DerivationGraph::new();
        for c in &g.constants {
            if c.name == "R_n" { continue; }
            let chain = g.trace(c.name);
            assert!(!chain.is_empty(), "{} has empty trace", c.name);
            assert_eq!(chain[0].name, "R_n", "{} doesn't reach axiom", c.name);
        }
    }

    #[test]
    fn test_trace_pi() {
        let g = DerivationGraph::new();
        let chain = g.trace("PI");
        let names: Vec<&str> = chain.iter().map(|c| c.name).collect();
        assert_eq!(names[0], "R_n");
        assert_eq!(*names.last().unwrap(), "PI");
        assert!(names.contains(&"R_4"));
        assert!(names.contains(&"SQRT_DELTA"));
    }

    #[test]
    fn test_value_91_has_multiple_roles() {
        let g = DerivationGraph::new();
        let _matches = g.find_by_value(repunit(3) * repunit(3).min(
            g.find_by_name("QUARTER_TURN").unwrap().value
        ));
        let qt = g.find_by_name("QUARTER_TURN").unwrap().value;
        let all = g.find_by_value(qt);
        let names: Vec<&str> = all.iter().map(|c| c.name).collect();
        assert!(names.contains(&"QUARTER_TURN"));
        assert!(names.contains(&"UV_EUV"));
        assert!(names.contains(&"GENUS_T_PI_15"));
    }

    #[test]
    fn test_sponge_width_trace() {
        let g = DerivationGraph::new();
        let chain = g.trace("DELTA_2");
        let names: Vec<&str> = chain.iter().map(|c| c.name).collect();
        assert!(names.contains(&"ARC_1"));
        // Δ₂ = (2·R_3+1)² = 3^6
        let r3 = g.find_by_name("R_3").unwrap().value;
        let expected = (2 * r3 + 1) * (2 * r3 + 1);
        assert_eq!(g.find_by_name("DELTA_2").unwrap().value, expected);
    }

    #[test]
    fn test_seifert_genus_equals_quarter_turn() {
        let g = DerivationGraph::new();
        assert_eq!(
            g.find_by_name("GENUS_T_PI_15").unwrap().value,
            g.find_by_name("QUARTER_TURN").unwrap().value
        );
    }

    #[test]
    fn test_dc_five_uninvited() {
        let g = DerivationGraph::new();
        let dc = g.find_by_name("DC_NUM").unwrap().value;
        let seven = g.find_by_name("SEVEN").unwrap().value;
        let r3 = g.find_by_name("R_3").unwrap().value;
        // The factor (dc / (seven × r3)) was never put into the signal
        let cofactor = dc / (seven * r3);
        assert_eq!(dc, cofactor * seven * r3);
        assert_eq!(cofactor, 5); // emerges from the algebra
    }

    #[test]
    fn test_uv_archimedes() {
        let g = DerivationGraph::new();
        let green = g.find_by_name("GREEN_ARC_EFF").unwrap().value;
        let qt = g.find_by_name("QUARTER_TURN").unwrap().value;
        let seven = g.find_by_name("SEVEN").unwrap().value;
        let eleven = g.find_by_name("ELEVEN").unwrap().value;
        // green/qt = 2·eleven/seven (cross-multiply)
        assert_eq!(green * seven, qt * 2 * eleven);
    }

    #[test]
    fn test_hyperbolicity_denominator_is_walk() {
        let g = DerivationGraph::new();
        let v = |name: &str| g.find_by_name(name).unwrap().value;
        assert_eq!(v("SEVEN") * v("ELEVEN") * v("R_3"), v("COPRIME_LCM"));
    }

    #[test]
    fn test_format_readable() {
        let g = DerivationGraph::new();
        let out = g.format("COPRIME_LCM");
        assert!(out.contains("AXIOM"));
        assert!(out.contains("COPRIME_LCM"));
        assert!(out.contains("Hamiltonian"));
    }
}
