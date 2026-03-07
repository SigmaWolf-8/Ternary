// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use crate::TernaryTrit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToroidalAxis {
    Eta,
    Theta,
    Psi,
}

pub struct GradientNeighbor {
    pub id: u64,
    pub field_value: TernaryTrit,
    pub dominant_axis: ToroidalAxis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TernaryGradient {
    pub eta: TernaryTrit,
    pub theta: TernaryTrit,
    pub psi: TernaryTrit,
}

impl TernaryGradient {
    pub fn zero() -> Self {
        Self { eta: TernaryTrit::Zero, theta: TernaryTrit::Zero, psi: TernaryTrit::Zero }
    }

    pub fn magnitude(&self) -> u8 {
        (self.eta != TernaryTrit::Zero) as u8
        + (self.theta != TernaryTrit::Zero) as u8
        + (self.psi != TernaryTrit::Zero) as u8
    }
}

#[inline]
pub fn gf3_sub(a: TernaryTrit, b: TernaryTrit) -> TernaryTrit {
    let raw = ((a.to_i8() as i16 - b.to_i8() as i16) % 3 + 3) % 3;
    match raw {
        0 => TernaryTrit::Zero,
        1 => TernaryTrit::Pos,
        2 => TernaryTrit::Neg,
        _ => unreachable!(),
    }
}

#[inline]
pub fn gf3_add(a: TernaryTrit, b: TernaryTrit) -> TernaryTrit {
    let raw = ((a.to_i8() as i16 + b.to_i8() as i16) % 3 + 3) % 3;
    match raw {
        0 => TernaryTrit::Zero,
        1 => TernaryTrit::Pos,
        2 => TernaryTrit::Neg,
        _ => unreachable!(),
    }
}

#[inline]
pub fn gf3_neg(a: TernaryTrit) -> TernaryTrit {
    match a {
        TernaryTrit::Neg => TernaryTrit::Pos,
        TernaryTrit::Zero => TernaryTrit::Zero,
        TernaryTrit::Pos => TernaryTrit::Neg,
    }
}

pub fn ternary_gradient(
    local_value: TernaryTrit,
    neighbors: &[GradientNeighbor],
) -> TernaryGradient {
    let mut eta_diffs: Vec<TernaryTrit> = Vec::new();
    let mut theta_diffs: Vec<TernaryTrit> = Vec::new();
    let mut psi_diffs: Vec<TernaryTrit> = Vec::new();

    for n in neighbors {
        let diff = gf3_neg(gf3_sub(n.field_value, local_value));
        match n.dominant_axis {
            ToroidalAxis::Eta => eta_diffs.push(diff),
            ToroidalAxis::Theta => theta_diffs.push(diff),
            ToroidalAxis::Psi => psi_diffs.push(diff),
        }
    }

    TernaryGradient {
        eta: majority_vote(&eta_diffs),
        theta: majority_vote(&theta_diffs),
        psi: majority_vote(&psi_diffs),
    }
}

pub fn majority_vote(trits: &[TernaryTrit]) -> TernaryTrit {
    if trits.is_empty() {
        return TernaryTrit::Zero;
    }
    let sum: i16 = trits.iter().map(|t| t.to_i8() as i16).sum();
    if sum > 0 {
        TernaryTrit::Pos
    } else if sum < 0 {
        TernaryTrit::Neg
    } else {
        TernaryTrit::Zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gf3_sub_complete_table() {
        assert_eq!(gf3_sub(TernaryTrit::Zero, TernaryTrit::Zero), TernaryTrit::Zero);
        assert_eq!(gf3_sub(TernaryTrit::Pos, TernaryTrit::Zero), TernaryTrit::Pos);
        assert_eq!(gf3_sub(TernaryTrit::Neg, TernaryTrit::Zero), TernaryTrit::Neg);
        assert_eq!(gf3_sub(TernaryTrit::Zero, TernaryTrit::Pos), TernaryTrit::Neg);
        assert_eq!(gf3_sub(TernaryTrit::Zero, TernaryTrit::Neg), TernaryTrit::Pos);
        assert_eq!(gf3_sub(TernaryTrit::Pos, TernaryTrit::Pos), TernaryTrit::Zero);
        assert_eq!(gf3_sub(TernaryTrit::Neg, TernaryTrit::Neg), TernaryTrit::Zero);
        assert_eq!(gf3_sub(TernaryTrit::Pos, TernaryTrit::Neg), TernaryTrit::Neg);
        assert_eq!(gf3_sub(TernaryTrit::Neg, TernaryTrit::Pos), TernaryTrit::Pos);
    }

    #[test]
    fn gf3_add_complete_table() {
        assert_eq!(gf3_add(TernaryTrit::Zero, TernaryTrit::Zero), TernaryTrit::Zero);
        assert_eq!(gf3_add(TernaryTrit::Pos, TernaryTrit::Zero), TernaryTrit::Pos);
        assert_eq!(gf3_add(TernaryTrit::Neg, TernaryTrit::Zero), TernaryTrit::Neg);
        assert_eq!(gf3_add(TernaryTrit::Pos, TernaryTrit::Pos), TernaryTrit::Neg);
        assert_eq!(gf3_add(TernaryTrit::Neg, TernaryTrit::Neg), TernaryTrit::Pos);
        assert_eq!(gf3_add(TernaryTrit::Pos, TernaryTrit::Neg), TernaryTrit::Zero);
    }

    #[test]
    fn gf3_negation() {
        assert_eq!(gf3_neg(TernaryTrit::Pos), TernaryTrit::Neg);
        assert_eq!(gf3_neg(TernaryTrit::Neg), TernaryTrit::Pos);
        assert_eq!(gf3_neg(TernaryTrit::Zero), TernaryTrit::Zero);
    }

    #[test]
    fn gf3_sub_is_add_neg() {
        let vals = [TernaryTrit::Neg, TernaryTrit::Zero, TernaryTrit::Pos];
        for &a in &vals {
            for &b in &vals {
                assert_eq!(gf3_sub(a, b), gf3_add(a, gf3_neg(b)),
                    "sub({:?},{:?}) != add({:?},neg({:?}))", a, b, a, b);
            }
        }
    }

    #[test]
    fn gradient_uniform_field_is_zero() {
        let neighbors = vec![
            GradientNeighbor { id: 1, field_value: TernaryTrit::Zero, dominant_axis: ToroidalAxis::Eta },
            GradientNeighbor { id: 2, field_value: TernaryTrit::Zero, dominant_axis: ToroidalAxis::Theta },
            GradientNeighbor { id: 3, field_value: TernaryTrit::Zero, dominant_axis: ToroidalAxis::Psi },
        ];
        let grad = ternary_gradient(TernaryTrit::Zero, &neighbors);
        assert_eq!(grad, TernaryGradient::zero());
    }

    #[test]
    fn gradient_detects_rising_field_along_eta() {
        let neighbors = vec![
            GradientNeighbor { id: 1, field_value: TernaryTrit::Pos, dominant_axis: ToroidalAxis::Eta },
            GradientNeighbor { id: 2, field_value: TernaryTrit::Pos, dominant_axis: ToroidalAxis::Eta },
            GradientNeighbor { id: 3, field_value: TernaryTrit::Zero, dominant_axis: ToroidalAxis::Theta },
        ];
        let grad = ternary_gradient(TernaryTrit::Zero, &neighbors);
        assert_eq!(grad.eta, TernaryTrit::Neg);
    }

    #[test]
    fn gradient_no_neighbors_on_axis_gives_zero() {
        let neighbors = vec![
            GradientNeighbor { id: 1, field_value: TernaryTrit::Pos, dominant_axis: ToroidalAxis::Eta },
        ];
        let grad = ternary_gradient(TernaryTrit::Zero, &neighbors);
        assert_eq!(grad.theta, TernaryTrit::Zero);
        assert_eq!(grad.psi, TernaryTrit::Zero);
    }

    #[test]
    fn majority_vote_clear_winner() {
        assert_eq!(majority_vote(&[TernaryTrit::Pos, TernaryTrit::Pos, TernaryTrit::Neg]), TernaryTrit::Pos);
        assert_eq!(majority_vote(&[TernaryTrit::Neg, TernaryTrit::Neg, TernaryTrit::Zero]), TernaryTrit::Neg);
        assert_eq!(majority_vote(&[TernaryTrit::Zero, TernaryTrit::Zero, TernaryTrit::Pos]), TernaryTrit::Pos);
    }

    #[test]
    fn majority_vote_tie_returns_zero() {
        assert_eq!(majority_vote(&[TernaryTrit::Pos, TernaryTrit::Neg]), TernaryTrit::Zero);
        assert_eq!(majority_vote(&[TernaryTrit::Pos, TernaryTrit::Neg, TernaryTrit::Zero]), TernaryTrit::Zero);
        assert_eq!(majority_vote(&[]), TernaryTrit::Zero);
    }

    #[test]
    fn gradient_magnitude() {
        let g = TernaryGradient { eta: TernaryTrit::Pos, theta: TernaryTrit::Zero, psi: TernaryTrit::Neg };
        assert_eq!(g.magnitude(), 2);
        assert_eq!(TernaryGradient::zero().magnitude(), 0);
    }
}
