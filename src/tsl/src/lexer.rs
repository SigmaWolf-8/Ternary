//! TSL Lexer - Tokenizes TSL source code

use crate::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Trit,
    Tryte,
    Word,
    Fn,
    Let,
    If,
    Else,
    While,
    Return,
    Phase,
    Timing,
    
    // Literals
    TritLiteral(i8),      // -1, 0, +1
    IntLiteral(i64),
    StringLiteral(String),
    
    // Identifiers
    Ident(String),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Rotate,     // >>>
    RotateInv,  // <<<
    TernaryNot, // ~
    TernaryXor, // ^
    
    // Comparisons
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    
    // Special
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, CompileError> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            '{' => { tokens.push(Token::LBrace); chars.next(); }
            '}' => { tokens.push(Token::RBrace); chars.next(); }
            '[' => { tokens.push(Token::LBracket); chars.next(); }
            ']' => { tokens.push(Token::RBracket); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }
            ':' => { tokens.push(Token::Colon); chars.next(); }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '*' => { tokens.push(Token::Star); chars.next(); }
            '/' => { tokens.push(Token::Slash); chars.next(); }
            '~' => { tokens.push(Token::TernaryNot); chars.next(); }
            '^' => { tokens.push(Token::TernaryXor); chars.next(); }
            
            '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else if chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    let mut num = String::from("-");
                    while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        num.push(chars.next().unwrap());
                    }
                    let value: i64 = num.parse().map_err(|_| CompileError::LexerError(format!("Invalid number: {}", num)))?;
                    if value >= -1 && value <= 1 {
                        tokens.push(Token::TritLiteral(value as i8));
                    } else {
                        tokens.push(Token::IntLiteral(value));
                    }
                } else {
                    tokens.push(Token::Minus);
                }
            }
            
            '>' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push(Token::Rotate);
                    } else {
                        return Err(CompileError::LexerError("Expected >>> for rotate".into()));
                    }
                } else if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            
            '<' => {
                chars.next();
                if chars.peek() == Some(&'<') {
                    chars.next();
                    if chars.peek() == Some(&'<') {
                        chars.next();
                        tokens.push(Token::RotateInv);
                    } else {
                        return Err(CompileError::LexerError("Expected <<< for inverse rotate".into()));
                    }
                } else if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Eq);
                } else {
                    return Err(CompileError::LexerError("Use == for equality".into()));
                }
            }
            
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ne);
                } else {
                    return Err(CompileError::LexerError("Use ~ for ternary NOT".into()));
                }
            }
            
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' { break; }
                    s.push(chars.next().unwrap());
                }
                chars.next(); // consume closing quote
                tokens.push(Token::StringLiteral(s));
            }
            
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    num.push(chars.next().unwrap());
                }
                let value: i64 = num.parse().map_err(|_| CompileError::LexerError(format!("Invalid number: {}", num)))?;
                if value >= 0 && value <= 1 {
                    tokens.push(Token::TritLiteral(value as i8));
                } else {
                    tokens.push(Token::IntLiteral(value));
                }
            }
            
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut ident = String::new();
                while chars.peek().map(|c| c.is_ascii_alphanumeric() || *c == '_').unwrap_or(false) {
                    ident.push(chars.next().unwrap());
                }
                let token = match ident.as_str() {
                    "trit" => Token::Trit,
                    "tryte" => Token::Tryte,
                    "word" => Token::Word,
                    "fn" => Token::Fn,
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "return" => Token::Return,
                    "phase" => Token::Phase,
                    "timing" => Token::Timing,
                    _ => Token::Ident(ident),
                };
                tokens.push(token);
            }
            
            _ => return Err(CompileError::LexerError(format!("Unexpected character: {}", c))),
        }
    }
    
    tokens.push(Token::Eof);
    Ok(tokens)
}
