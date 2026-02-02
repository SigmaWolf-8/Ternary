//! THDL Timing Analysis

use crate::ir::*;

/// Timing analysis result
#[derive(Debug, Clone)]
pub struct TimingAnalysis {
    pub critical_path_ps: u64,
    pub slack_ps: i64,
    pub paths: Vec<TimingPath>,
}

/// A timing path through the design
#[derive(Debug, Clone)]
pub struct TimingPath {
    pub from: String,
    pub to: String,
    pub delay_ps: u64,
    pub elements: Vec<PathElement>,
}

/// Element in a timing path
#[derive(Debug, Clone)]
pub struct PathElement {
    pub name: String,
    pub delay_ps: u64,
    pub element_type: PathElementType,
}

#[derive(Debug, Clone)]
pub enum PathElementType {
    Combinational,
    Register,
    Wire,
    TritCell,
}

/// Analyze timing of a module
pub fn analyze_timing(module: &Module) -> TimingAnalysis {
    let mut paths = Vec::new();
    let mut max_delay = 0u64;
    
    // Analyze each assignment as a potential path
    for assignment in &module.assignments {
        let delay = estimate_expression_delay(&assignment.expression);
        max_delay = max_delay.max(delay);
        
        paths.push(TimingPath {
            from: "input".to_string(),
            to: assignment.target.clone(),
            delay_ps: delay,
            elements: vec![],
        });
    }
    
    TimingAnalysis {
        critical_path_ps: max_delay,
        slack_ps: 0, // Would be constraint - actual
        paths,
    }
}

fn estimate_expression_delay(expr: &Expression) -> u64 {
    // Simple delay model in picoseconds
    const TRIT_OP_DELAY: u64 = 50;   // 50ps per ternary operation
    const BINARY_OP_DELAY: u64 = 30; // 30ps per binary operation
    const WIRE_DELAY: u64 = 5;       // 5ps wire delay
    
    match expr {
        Expression::BinaryOp(op, left, right) => {
            let base_delay = match op {
                BinaryOp::TritAdd | BinaryOp::TritMul | BinaryOp::TritXor => TRIT_OP_DELAY,
                _ => BINARY_OP_DELAY,
            };
            base_delay + estimate_expression_delay(left).max(estimate_expression_delay(right))
        }
        Expression::UnaryOp(op, inner) => {
            let base_delay = match op {
                UnaryOp::TritNot | UnaryOp::TritRotate => TRIT_OP_DELAY,
                _ => BINARY_OP_DELAY / 2,
            };
            base_delay + estimate_expression_delay(inner)
        }
        Expression::TernaryOp(cond, then_expr, else_expr) => {
            BINARY_OP_DELAY + estimate_expression_delay(cond) 
                + estimate_expression_delay(then_expr).max(estimate_expression_delay(else_expr))
        }
        Expression::FunctionCall(_, args) => {
            TRIT_OP_DELAY * 2 + args.iter().map(estimate_expression_delay).max().unwrap_or(0)
        }
        Expression::Ident(_) => WIRE_DELAY,
        _ => 0,
    }
}
