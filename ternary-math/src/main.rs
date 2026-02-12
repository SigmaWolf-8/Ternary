// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use ternary_math::clifford;
use ternary_math::radix;
use ternary_math::torus;

fn main() {
    println!("{}", radix::full_benchmark_report());
    println!("{}", torus::full_topology_report());

    // Clifford algebra summary
    println!("═══════════════════════════════════════════════════════");
    println!("  CLIFFORD ALGEBRA Cl(3,0)/GF(3) — PlenumNET");
    println!("═══════════════════════════════════════════════════════\n");

    let rotors = clifford::all_invertible_rotors();
    println!("Invertible rotors: {} / 81 even-grade elements\n", rotors.len());

    // Show a few example rotors and their compositions
    if rotors.len() >= 3 {
        let r0 = &rotors[1];
        let r1 = &rotors[2];
        let r2 = &rotors[3];

        println!("Example gate composition:");
        println!("  R₀ = {:?}", r0);
        println!("  R₁ = {:?}", r1);
        println!("  R₂ = {:?}", r2);

        let composed = clifford::Multivector::compose_chain(&[*r0, *r1, *r2]);
        println!("  R₂·R₁·R₀ = {:?}", composed);
        println!();
        println!("  Three sequential gate applications → one rotor multiplication.");
        println!("  This scales to any N: compose N rotors once, apply the result in O(1).");
    }
}
