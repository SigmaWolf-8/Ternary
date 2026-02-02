//! THDL Optimization Passes
//!
//! Implements various optimization strategies for ternary hardware:
//! - Constant folding
//! - Dead code elimination
//! - Common subexpression elimination
//! - Ternary-specific optimizations
//! - Timing-driven optimization
//!
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use crate::ir::*;
use crate::{SynthesisError, SynthesisOptions};
use std::collections::{HashMap, HashSet};

/// Run all optimization passes
pub fn optimize(module: &Module, options: &SynthesisOptions) -> Result<Module, SynthesisError> {
    let mut result = module.clone();
    
    // Run optimization passes in order
    result = constant_folding(&result)?;
    result = dead_code_elimination(&result)?;
    result = common_subexpression_elimination(&result)?;
    result = ternary_specific_optimizations(&result)?;
    
    if options.optimize_speed {
        result = timing_optimization(&result, options)?;
    }
    
    if options.optimize_area {
        result = area_optimization(&result)?;
    }
    
    if options.optimize_power {
        result = power_optimization(&result)?;
    }
    
    Ok(result)
}

/// Constant Folding Pass
/// 
/// Evaluates constant expressions at compile time.
/// For ternary operations, uses GF(3) arithmetic.
pub fn constant_folding(module: &Module) -> Result<Module, SynthesisError> {
    let mut result = module.clone();
    
    for assignment in &mut result.assignments {
        assignment.expression = fold_constants(&assignment.expression);
    }
    
    for block in &mut result.always_blocks {
        for stmt in &mut block.statements {
            fold_statement_constants(stmt);
        }
    }
    
    Ok(result)
}

fn fold_constants(expr: &Expression) -> Expression {
    match expr {
        Expression::BinaryOp(op, left, right) => {
            let left_folded = fold_constants(left);
            let right_folded = fold_constants(right);
            
            // Try to evaluate if both operands are literals
            match (&left_folded, &right_folded) {
                (Expression::TritLiteral(a), Expression::TritLiteral(b)) => {
                    let result = match op {
                        BinaryOp::TritAdd => (*a + *b).rem_euclid(3) as i8 - 1,
                        BinaryOp::TritMul => (*a * *b).rem_euclid(3) as i8 - 1,
                        BinaryOp::TritXor => std::cmp::min(*a, *b),
                        _ => return Expression::BinaryOp(*op, Box::new(left_folded), Box::new(right_folded)),
                    };
                    Expression::TritLiteral(result)
                }
                (Expression::Literal(a), Expression::Literal(b)) => {
                    let result = match op {
                        BinaryOp::Add => a + b,
                        BinaryOp::Sub => a - b,
                        BinaryOp::Mul => a * b,
                        BinaryOp::And => a & b,
                        BinaryOp::Or => a | b,
                        BinaryOp::Xor => a ^ b,
                        _ => return Expression::BinaryOp(*op, Box::new(left_folded), Box::new(right_folded)),
                    };
                    Expression::Literal(result)
                }
                _ => Expression::BinaryOp(*op, Box::new(left_folded), Box::new(right_folded)),
            }
        }
        Expression::UnaryOp(op, inner) => {
            let inner_folded = fold_constants(inner);
            
            match (&inner_folded, op) {
                (Expression::TritLiteral(v), UnaryOp::TritNot) => {
                    Expression::TritLiteral(-*v)
                }
                (Expression::TritLiteral(v), UnaryOp::TritRotate) => {
                    let rotated = match *v {
                        -1 => 0,
                        0 => 1,
                        1 => -1,
                        _ => *v,
                    };
                    Expression::TritLiteral(rotated)
                }
                _ => Expression::UnaryOp(*op, Box::new(inner_folded)),
            }
        }
        _ => expr.clone(),
    }
}

fn fold_statement_constants(stmt: &mut Statement) {
    match stmt {
        Statement::Assign(_, expr) => {
            *expr = fold_constants(expr);
        }
        Statement::If(cond, then_stmts, else_stmts) => {
            *cond = fold_constants(cond);
            for s in then_stmts {
                fold_statement_constants(s);
            }
            if let Some(else_block) = else_stmts {
                for s in else_block {
                    fold_statement_constants(s);
                }
            }
        }
        Statement::Block(stmts) => {
            for s in stmts {
                fold_statement_constants(s);
            }
        }
        Statement::Case(expr, cases, default) => {
            *expr = fold_constants(expr);
            for (case_expr, case_stmts) in cases {
                *case_expr = fold_constants(case_expr);
                for s in case_stmts {
                    fold_statement_constants(s);
                }
            }
            if let Some(default_stmts) = default {
                for s in default_stmts {
                    fold_statement_constants(s);
                }
            }
        }
    }
}

/// Dead Code Elimination Pass
///
/// Removes signals and assignments that are never used.
pub fn dead_code_elimination(module: &Module) -> Result<Module, SynthesisError> {
    let mut result = module.clone();
    
    // Find all used signals
    let mut used_signals = HashSet::new();
    
    // Outputs are always used
    for port in &result.ports {
        if port.direction == PortDirection::Output {
            used_signals.insert(port.name.clone());
        }
    }
    
    // Collect signals used in assignments
    for assignment in &result.assignments {
        collect_used_signals(&assignment.expression, &mut used_signals);
    }
    
    // Collect signals used in always blocks
    for block in &result.always_blocks {
        for stmt in &block.statements {
            collect_used_in_statement(stmt, &mut used_signals);
        }
    }
    
    // Remove unused signals
    result.signals.retain(|s| used_signals.contains(&s.name));
    
    // Remove assignments to unused signals
    result.assignments.retain(|a| used_signals.contains(&a.target));
    
    Ok(result)
}

fn collect_used_signals(expr: &Expression, used: &mut HashSet<String>) {
    match expr {
        Expression::Ident(name) => { used.insert(name.clone()); }
        Expression::BinaryOp(_, left, right) => {
            collect_used_signals(left, used);
            collect_used_signals(right, used);
        }
        Expression::UnaryOp(_, inner) => {
            collect_used_signals(inner, used);
        }
        Expression::TernaryOp(cond, then_expr, else_expr) => {
            collect_used_signals(cond, used);
            collect_used_signals(then_expr, used);
            collect_used_signals(else_expr, used);
        }
        Expression::FunctionCall(_, args) => {
            for arg in args {
                collect_used_signals(arg, used);
            }
        }
        Expression::Concat(exprs) => {
            for e in exprs {
                collect_used_signals(e, used);
            }
        }
        Expression::BitSelect(inner, _) | Expression::RangeSelect(inner, _, _) => {
            collect_used_signals(inner, used);
        }
        _ => {}
    }
}

fn collect_used_in_statement(stmt: &Statement, used: &mut HashSet<String>) {
    match stmt {
        Statement::Assign(_, expr) => collect_used_signals(expr, used),
        Statement::If(cond, then_stmts, else_stmts) => {
            collect_used_signals(cond, used);
            for s in then_stmts {
                collect_used_in_statement(s, used);
            }
            if let Some(else_block) = else_stmts {
                for s in else_block {
                    collect_used_in_statement(s, used);
                }
            }
        }
        Statement::Block(stmts) => {
            for s in stmts {
                collect_used_in_statement(s, used);
            }
        }
        Statement::Case(expr, cases, default) => {
            collect_used_signals(expr, used);
            for (case_expr, case_stmts) in cases {
                collect_used_signals(case_expr, used);
                for s in case_stmts {
                    collect_used_in_statement(s, used);
                }
            }
            if let Some(default_stmts) = default {
                for s in default_stmts {
                    collect_used_in_statement(s, used);
                }
            }
        }
    }
}

/// Common Subexpression Elimination Pass
///
/// Identifies and reuses duplicate expressions.
pub fn common_subexpression_elimination(module: &Module) -> Result<Module, SynthesisError> {
    let mut result = module.clone();
    let mut expr_cache: HashMap<String, String> = HashMap::new();
    let mut new_signals = Vec::new();
    let mut cse_counter = 0;
    
    for assignment in &mut result.assignments {
        let expr_key = format!("{:?}", assignment.expression);
        
        if let Some(existing) = expr_cache.get(&expr_key) {
            // Replace with reference to existing computation
            assignment.expression = Expression::Ident(existing.clone());
        } else {
            // Check if this expression is complex enough to cache
            if is_complex_expression(&assignment.expression) {
                let signal_name = format!("_cse_{}", cse_counter);
                cse_counter += 1;
                
                new_signals.push(Signal {
                    name: signal_name.clone(),
                    width: 2, // Default trit width
                    is_reg: false,
                    trit_type: true,
                });
                
                expr_cache.insert(expr_key, signal_name);
            }
        }
    }
    
    result.signals.extend(new_signals);
    Ok(result)
}

fn is_complex_expression(expr: &Expression) -> bool {
    match expr {
        Expression::BinaryOp(_, _, _) => true,
        Expression::UnaryOp(_, inner) => is_complex_expression(inner),
        Expression::FunctionCall(_, _) => true,
        _ => false,
    }
}

/// Ternary-Specific Optimizations
///
/// Optimizations unique to ternary logic:
/// - Rotation chain reduction
/// - Identity element elimination
/// - GF(3) algebraic simplifications
pub fn ternary_specific_optimizations(module: &Module) -> Result<Module, SynthesisError> {
    let mut result = module.clone();
    
    for assignment in &mut result.assignments {
        assignment.expression = optimize_ternary_expr(&assignment.expression);
    }
    
    Ok(result)
}

fn optimize_ternary_expr(expr: &Expression) -> Expression {
    match expr {
        // Triple rotation is identity: rotate(rotate(rotate(x))) = x
        Expression::UnaryOp(UnaryOp::TritRotate, inner) => {
            if let Expression::UnaryOp(UnaryOp::TritRotate, inner2) = inner.as_ref() {
                if let Expression::UnaryOp(UnaryOp::TritRotate, inner3) = inner2.as_ref() {
                    return optimize_ternary_expr(inner3);
                }
            }
            Expression::UnaryOp(UnaryOp::TritRotate, Box::new(optimize_ternary_expr(inner)))
        }
        
        // Double NOT is identity: not(not(x)) = x
        Expression::UnaryOp(UnaryOp::TritNot, inner) => {
            if let Expression::UnaryOp(UnaryOp::TritNot, inner2) = inner.as_ref() {
                return optimize_ternary_expr(inner2);
            }
            Expression::UnaryOp(UnaryOp::TritNot, Box::new(optimize_ternary_expr(inner)))
        }
        
        // Addition with 0 is identity: x + 0 = x
        Expression::BinaryOp(BinaryOp::TritAdd, left, right) => {
            let left_opt = optimize_ternary_expr(left);
            let right_opt = optimize_ternary_expr(right);
            
            match (&left_opt, &right_opt) {
                (_, Expression::TritLiteral(0)) => left_opt,
                (Expression::TritLiteral(0), _) => right_opt,
                _ => Expression::BinaryOp(BinaryOp::TritAdd, Box::new(left_opt), Box::new(right_opt)),
            }
        }
        
        // Multiplication with 1 is identity: x * 1 = x
        // Multiplication with 0 is 0: x * 0 = 0
        Expression::BinaryOp(BinaryOp::TritMul, left, right) => {
            let left_opt = optimize_ternary_expr(left);
            let right_opt = optimize_ternary_expr(right);
            
            match (&left_opt, &right_opt) {
                (_, Expression::TritLiteral(1)) => left_opt,
                (Expression::TritLiteral(1), _) => right_opt,
                (_, Expression::TritLiteral(0)) | (Expression::TritLiteral(0), _) => {
                    Expression::TritLiteral(0)
                }
                _ => Expression::BinaryOp(BinaryOp::TritMul, Box::new(left_opt), Box::new(right_opt)),
            }
        }
        
        // Recurse into other binary operations
        Expression::BinaryOp(op, left, right) => {
            Expression::BinaryOp(*op, 
                Box::new(optimize_ternary_expr(left)), 
                Box::new(optimize_ternary_expr(right)))
        }
        
        _ => expr.clone(),
    }
}

/// Timing-Driven Optimization
///
/// Restructures logic to meet timing constraints.
pub fn timing_optimization(module: &Module, options: &SynthesisOptions) -> Result<Module, SynthesisError> {
    let mut result = module.clone();
    
    // Calculate critical path and add pipeline registers if needed
    let _critical_path = estimate_critical_path(&result);
    
    // If critical path exceeds constraint, add pipeline stages
    if options.timing_constraints.max_clock_period_ps > 0 {
        // Pipeline insertion logic would go here
        // For now, just return the module as-is
    }
    
    Ok(result)
}

fn estimate_critical_path(module: &Module) -> u64 {
    // Simple delay model: 10ps per logic level
    let mut max_depth = 0u64;
    
    for assignment in &module.assignments {
        let depth = expression_depth(&assignment.expression);
        max_depth = max_depth.max(depth as u64);
    }
    
    max_depth * 10 // 10ps per level
}

fn expression_depth(expr: &Expression) -> usize {
    match expr {
        Expression::BinaryOp(_, left, right) => {
            1 + expression_depth(left).max(expression_depth(right))
        }
        Expression::UnaryOp(_, inner) => 1 + expression_depth(inner),
        Expression::TernaryOp(cond, then_expr, else_expr) => {
            1 + expression_depth(cond)
                .max(expression_depth(then_expr))
                .max(expression_depth(else_expr))
        }
        _ => 0,
    }
}

/// Area Optimization
///
/// Reduces resource usage through logic sharing and minimization.
pub fn area_optimization(module: &Module) -> Result<Module, SynthesisError> {
    // For now, return unchanged
    // Real implementation would do:
    // - Logic minimization
    // - Resource sharing
    // - LUT optimization for FPGAs
    Ok(module.clone())
}

/// Power Optimization
///
/// Reduces dynamic and static power consumption.
pub fn power_optimization(module: &Module) -> Result<Module, SynthesisError> {
    // For now, return unchanged
    // Real implementation would do:
    // - Clock gating insertion
    // - Operand isolation
    // - State encoding optimization
    Ok(module.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_trit_add() {
        let expr = Expression::BinaryOp(
            BinaryOp::TritAdd,
            Box::new(Expression::TritLiteral(1)),
            Box::new(Expression::TritLiteral(1)),
        );
        let folded = fold_constants(&expr);
        assert!(matches!(folded, Expression::TritLiteral(-1)));
    }

    #[test]
    fn test_double_not_elimination() {
        let expr = Expression::UnaryOp(
            UnaryOp::TritNot,
            Box::new(Expression::UnaryOp(
                UnaryOp::TritNot,
                Box::new(Expression::Ident("x".to_string())),
            )),
        );
        let optimized = optimize_ternary_expr(&expr);
        assert!(matches!(optimized, Expression::Ident(_)));
    }

    #[test]
    fn test_multiply_by_zero() {
        let expr = Expression::BinaryOp(
            BinaryOp::TritMul,
            Box::new(Expression::Ident("x".to_string())),
            Box::new(Expression::TritLiteral(0)),
        );
        let optimized = optimize_ternary_expr(&expr);
        assert!(matches!(optimized, Expression::TritLiteral(0)));
    }
}
