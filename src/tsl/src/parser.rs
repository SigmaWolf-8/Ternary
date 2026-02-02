//! TSL Parser - Builds AST from tokens

use crate::lexer::Token;
use crate::ast::*;
use crate::CompileError;

pub fn parse(tokens: &[Token]) -> Result<Program, CompileError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }
    
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    
    fn advance(&mut self) {
        self.pos += 1;
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), CompileError> {
        if std::mem::discriminant(self.current()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(CompileError::ParserError(format!(
                "Expected {:?}, found {:?}", expected, self.current()
            )))
        }
    }
    
    fn parse_program(&mut self) -> Result<Program, CompileError> {
        let mut functions = Vec::new();
        
        while *self.current() != Token::Eof {
            functions.push(self.parse_function()?);
        }
        
        Ok(Program { functions })
    }
    
    fn parse_function(&mut self) -> Result<Function, CompileError> {
        self.expect(Token::Fn)?;
        
        let name = match self.current() {
            Token::Ident(s) => s.clone(),
            _ => return Err(CompileError::ParserError("Expected function name".into())),
        };
        self.advance();
        
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;
        
        let return_type = if *self.current() == Token::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RBrace)?;
        
        Ok(Function { name, params, return_type, body })
    }
    
    fn parse_params(&mut self) -> Result<Vec<Parameter>, CompileError> {
        let mut params = Vec::new();
        
        while *self.current() != Token::RParen {
            let name = match self.current() {
                Token::Ident(s) => s.clone(),
                _ => return Err(CompileError::ParserError("Expected parameter name".into())),
            };
            self.advance();
            
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            
            params.push(Parameter { name, ty });
            
            if *self.current() == Token::Comma {
                self.advance();
            }
        }
        
        Ok(params)
    }
    
    fn parse_type(&mut self) -> Result<Type, CompileError> {
        let ty = match self.current() {
            Token::Trit => Type::Trit,
            Token::Tryte => Type::Tryte,
            Token::Word => Type::Word,
            _ => return Err(CompileError::ParserError("Expected type".into())),
        };
        self.advance();
        Ok(ty)
    }
    
    fn parse_block(&mut self) -> Result<Vec<Statement>, CompileError> {
        let mut statements = Vec::new();
        
        while *self.current() != Token::RBrace && *self.current() != Token::Eof {
            statements.push(self.parse_statement()?);
        }
        
        Ok(statements)
    }
    
    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        match self.current() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Return => self.parse_return(),
            _ => self.parse_expr_statement(),
        }
    }
    
    fn parse_let(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'let'
        
        let name = match self.current() {
            Token::Ident(s) => s.clone(),
            _ => return Err(CompileError::ParserError("Expected variable name".into())),
        };
        self.advance();
        
        let ty = if *self.current() == Token::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Expect assignment
        self.expect(Token::Eq)?; // This might need adjustment based on how = is tokenized
        let value = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        
        Ok(Statement::Let { name, ty, value })
    }
    
    fn parse_if(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'if'
        
        let condition = self.parse_expression()?;
        self.expect(Token::LBrace)?;
        let then_block = self.parse_block()?;
        self.expect(Token::RBrace)?;
        
        let else_block = if *self.current() == Token::Else {
            self.advance();
            self.expect(Token::LBrace)?;
            let block = self.parse_block()?;
            self.expect(Token::RBrace)?;
            Some(block)
        } else {
            None
        };
        
        Ok(Statement::If { condition, then_block, else_block })
    }
    
    fn parse_while(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'while'
        
        let condition = self.parse_expression()?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RBrace)?;
        
        Ok(Statement::While { condition, body })
    }
    
    fn parse_return(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'return'
        
        let value = if *self.current() != Token::Semicolon {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(Token::Semicolon)?;
        Ok(Statement::Return(value))
    }
    
    fn parse_expr_statement(&mut self) -> Result<Statement, CompileError> {
        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::Expression(expr))
    }
    
    fn parse_expression(&mut self) -> Result<Expression, CompileError> {
        self.parse_additive()
    }
    
    fn parse_additive(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_multiplicative()?;
        
        while matches!(self.current(), Token::Plus | Token::Minus) {
            let op = match self.current() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_multiplicative(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_unary()?;
        
        while matches!(self.current(), Token::Star | Token::Slash) {
            let op = match self.current() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<Expression, CompileError> {
        match self.current() {
            Token::TernaryNot => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Token::Rotate => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Rotate,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }
    
    fn parse_primary(&mut self) -> Result<Expression, CompileError> {
        match self.current() {
            Token::TritLiteral(v) => {
                let val = *v;
                self.advance();
                Ok(Expression::TritLiteral(val))
            }
            Token::IntLiteral(v) => {
                let val = *v;
                self.advance();
                Ok(Expression::IntLiteral(val))
            }
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                
                if *self.current() == Token::LParen {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    Ok(Expression::Call { name, args })
                } else {
                    Ok(Expression::Ident(name))
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(CompileError::ParserError(format!(
                "Unexpected token: {:?}", self.current()
            ))),
        }
    }
    
    fn parse_args(&mut self) -> Result<Vec<Expression>, CompileError> {
        let mut args = Vec::new();
        
        while *self.current() != Token::RParen {
            args.push(self.parse_expression()?);
            
            if *self.current() == Token::Comma {
                self.advance();
            }
        }
        
        Ok(args)
    }
}
