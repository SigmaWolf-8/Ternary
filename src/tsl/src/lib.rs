//! PlenumNET Ternary System Language (TSL)
//!
//! A high-level programming language for ternary computing systems.
//! TSL compiles to THDL (Ternary Hardware Description Language) or
//! directly to ternary machine code.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod compiler;

/// TSL version
pub const TSL_VERSION: &str = "0.1.0";

/// Compile TSL source code to THDL
pub fn compile_to_thdl(source: &str) -> Result<String, CompileError> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(&tokens)?;
    compiler::generate_thdl(&ast)
}

/// Compilation error
#[derive(Debug, Clone)]
pub enum CompileError {
    LexerError(String),
    ParserError(String),
    SemanticError(String),
    CodeGenError(String),
}
