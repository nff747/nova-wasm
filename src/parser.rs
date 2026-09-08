//! Recursive descent parser with Pratt precedence climbing for Nova.

use crate::ast::*;
use crate::token::{Span, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut memory = None;

        while !self.is_at_end() {
            if self.check(&TokenKind::Import) {
                imports.push(self.parse_import_decl()?);
                continue;
            }

            let is_export = self.match_token(&TokenKind::Export);

            if self.check(&TokenKind::Memory) {
                if memory.is_some() {
                    return Err(format!("Duplicate memory declaration at {}", self.peek().span));
                }
                memory = Some(self.parse_memory_decl(is_export)?);
            } else if self.check(&TokenKind::Fn) {
                functions.push(self.parse_function(is_export)?);
            } else {
                let token = self.peek();
                return Err(format!("Unexpected token '{:?}' at top level ({})", token.kind, token.span));
            }
        }

        Ok(Program { imports, functions, memory })
    }

    fn parse_import_decl(&mut self) -> Result<ImportFunc, String> {
        let import_token = self.consume(&TokenKind::Import, "Expected 'import'")?;
        let module = match self.advance().kind {
            TokenKind::Str(s) => s,
            _ => return Err(format!("Expected module name string after 'import' at {}", import_token.span)),
        };

        let mut field = None;
        if let TokenKind::Str(s) = &self.peek().kind {
            field = Some(s.clone());
            self.advance();
        }

        self.consume(&TokenKind::Fn, "Expected 'fn' in import declaration")?;
        let name_token = self.peek().clone();
        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            _ => return Err(format!("Expected function identifier in import at {}", name_token.span)),
        };

        let resolved_field = field.unwrap_or_else(|| name.clone());

        self.consume(&TokenKind::LParen, "Expected '(' after imported function name")?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                let p_span = self.peek().span;
                let p_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    _ => return Err(format!("Expected parameter name at {}", p_span)),
                };

                self.consume(&TokenKind::Colon, "Expected ':' after parameter name")?;
                let p_ty = self.parse_type()?;
                params.push(Param {
                    name: p_name,
                    ty: p_ty,
                    span: p_span,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenKind::RParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };

        self.consume(&TokenKind::Semicolon, "Expected ';' after import declaration")?;

        Ok(ImportFunc {
            module,
            field: resolved_field,
            name,
            params,
            return_type,
            span: import_token.span,
        })
    }

    fn parse_memory_decl(&mut self, is_export: bool) -> Result<MemoryDecl, String> {
        let span_start = self.peek().span;
        self.consume(&TokenKind::Memory, "Expected 'memory'")?;

        let initial_pages = match self.advance().kind {
            TokenKind::Int(val) => val as u32,
            _ => return Err(format!("Expected integer literal for initial memory pages at {}", span_start)),
        };

        let mut max_pages = None;
        if self.match_token(&TokenKind::Comma) {
            max_pages = match self.advance().kind {
                TokenKind::Int(val) => Some(val as u32),
                _ => return Err(format!("Expected integer literal for maximum memory pages at {}", self.peek().span)),
            };
        }

        self.consume(&TokenKind::Semicolon, "Expected ';' after memory declaration")?;
        Ok(MemoryDecl {
            initial_pages,
            max_pages,
            is_export,
            span: span_start,
        })
    }

    fn parse_function(&mut self, is_export: bool) -> Result<Function, String> {
        let fn_token = self.consume(&TokenKind::Fn, "Expected 'fn'")?;
        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            _ => return Err(format!("Expected function name after 'fn' at {}", fn_token.span)),
        };

        self.consume(&TokenKind::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                let p_span = self.peek().span;
                let p_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    _ => return Err(format!("Expected parameter name at {}", p_span)),
                };

                self.consume(&TokenKind::Colon, "Expected ':' after parameter name")?;
                let p_ty = self.parse_type()?;
                params.push(Param {
                    name: p_name,
                    ty: p_ty,
                    span: p_span,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenKind::RParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = self.parse_block()?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
            is_export,
            span: fn_token.span,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let token = self.advance();
        match token.kind {
            TokenKind::TypeI32 => Ok(Type::I32),
            TokenKind::TypeI64 => Ok(Type::I64),
            TokenKind::TypeF32 => Ok(Type::F32),
            TokenKind::TypeF64 => Ok(Type::F64),
            TokenKind::TypeBool => Ok(Type::Bool),
            TokenKind::TypeVoid => Ok(Type::Void),
            _ => Err(format!("Expected type at {}", token.span)),
        }
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        let brace = self.consume(&TokenKind::LBrace, "Expected '{' to start block")?;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(&TokenKind::RBrace, "Expected '}' to close block")?;
        Ok(Block {
            statements,
            span: brace.span,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek();
        match token.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Break => {
                let tok = self.advance();
                self.consume(&TokenKind::Semicolon, "Expected ';' after 'break'")?;
                Ok(Statement::Break(tok.span))
            }
            TokenKind::At => {
                // Potential @store_i32(ptr, val);
                self.parse_at_statement()
            }
            TokenKind::Ident(_) => {
                // Check if assignment (e.g. x = expr;) or expression statement (e.g. foo();)
                if self.check_ahead_assignment() {
                    self.parse_assignment_statement()
                } else {
                    self.parse_expr_statement()
                }
            }
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        let let_tok = self.consume(&TokenKind::Let, "Expected 'let'")?;
        let is_mut = self.match_token(&TokenKind::Mut);

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            _ => return Err(format!("Expected variable name at {}", self.peek().span)),
        };

        let ty = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&TokenKind::Eq, "Expected '=' in variable declaration")?;
        let init = self.parse_expression(0)?;
        self.consume(&TokenKind::Semicolon, "Expected ';' after let statement")?;

        Ok(Statement::Let {
            name,
            is_mut,
            ty,
            init,
            span: let_tok.span,
        })
    }

    fn check_ahead_assignment(&self) -> bool {
        if self.cursor + 1 < self.tokens.len() {
            matches!(
                self.tokens[self.cursor + 1].kind,
                TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq
            )
        } else {
            false
        }
    }

    fn parse_assignment_statement(&mut self) -> Result<Statement, String> {
        let target_tok = self.advance();
        let target = match target_tok.kind {
            TokenKind::Ident(n) => n,
            _ => unreachable!(),
        };

        let op_tok = self.advance();
        let value = match op_tok.kind {
            TokenKind::Eq => self.parse_expression(0)?,
            TokenKind::PlusEq => {
                let right = self.parse_expression(0)?;
                Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Variable(target.clone(), target_tok.span)),
                    right: Box::new(right),
                    span: op_tok.span,
                }
            }
            TokenKind::MinusEq => {
                let right = self.parse_expression(0)?;
                Expression::Binary {
                    op: BinaryOp::Sub,
                    left: Box::new(Expression::Variable(target.clone(), target_tok.span)),
                    right: Box::new(right),
                    span: op_tok.span,
                }
            }
            _ => return Err(format!("Expected assignment operator at {}", op_tok.span)),
        };

        self.consume(&TokenKind::Semicolon, "Expected ';' after assignment")?;
        Ok(Statement::Assignment {
            target,
            value,
            span: target_tok.span,
        })
    }

    fn parse_at_statement(&mut self) -> Result<Statement, String> {
        let at_tok = self.consume(&TokenKind::At, "Expected '@'")?;
        let op_name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            _ => return Err(format!("Expected memory intrinsic name after '@' at {}", at_tok.span)),
        };

        let ty = match op_name.as_str() {
            "store_i32" => Type::I32,
            "store_i64" => Type::I64,
            "store_f32" => Type::F32,
            "store_f64" => Type::F64,
            _ => return Err(format!("Unknown memory store intrinsic '@{}' at {}", op_name, at_tok.span)),
        };

        self.consume(&TokenKind::LParen, "Expected '(' after intrinsic")?;
        let ptr = self.parse_expression(0)?;
        self.consume(&TokenKind::Comma, "Expected ',' between pointer and value")?;
        let value = self.parse_expression(0)?;
        self.consume(&TokenKind::RParen, "Expected ')' after arguments")?;
        self.consume(&TokenKind::Semicolon, "Expected ';' after intrinsic statement")?;

        Ok(Statement::MemoryStore {
            ty,
            ptr,
            offset: 0,
            value,
            span: at_tok.span,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        let if_tok = self.consume(&TokenKind::If, "Expected 'if'")?;
        let has_paren = self.match_token(&TokenKind::LParen);
        let condition = self.parse_expression(0)?;
        if has_paren {
            self.consume(&TokenKind::RParen, "Expected ')' after if condition")?;
        }

        let then_branch = self.parse_block()?;
        let else_branch = if self.match_token(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                // else if -> wrap in single-statement block
                let nested_if = self.parse_if_statement()?;
                Some(Block {
                    span: nested_if_span(&nested_if),
                    statements: vec![nested_if],
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            span: if_tok.span,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        let while_tok = self.consume(&TokenKind::While, "Expected 'while'")?;
        let has_paren = self.match_token(&TokenKind::LParen);
        let condition = self.parse_expression(0)?;
        if has_paren {
            self.consume(&TokenKind::RParen, "Expected ')' after while condition")?;
        }

        let body = self.parse_block()?;
        Ok(Statement::While {
            condition,
            body,
            span: while_tok.span,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        let ret_tok = self.consume(&TokenKind::Return, "Expected 'return'")?;
        let value = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        self.consume(&TokenKind::Semicolon, "Expected ';' after return statement")?;

        Ok(Statement::Return {
            value,
            span: ret_tok.span,
        })
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expression(0)?;
        let span = expr.span();
        self.consume(&TokenKind::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::Expr { expr, span })
    }

    // Pratt parser for binary expressions
    fn parse_expression(&mut self, min_bp: u8) -> Result<Expression, String> {
        let mut left = self.parse_prefix()?;

        while let Some((op, l_bp, r_bp)) = self.peek_infix_op() {
            if l_bp < min_bp {
                break;
            }

            self.advance(); // consume operator
            let right = self.parse_expression(r_bp)?;
            let span = left.span();

            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression, String> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Minus => {
                let span = self.advance().span;
                let expr = self.parse_expression(8)?;
                Ok(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Bang => {
                let span = self.advance().span;
                let expr = self.parse_expression(8)?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span,
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let token = self.advance();
        match token.kind {
            TokenKind::Int(v) => Ok(Expression::LiteralInt(v, token.span)),
            TokenKind::Float(v) => Ok(Expression::LiteralFloat(v, token.span)),
            TokenKind::True => Ok(Expression::LiteralBool(true, token.span)),
            TokenKind::False => Ok(Expression::LiteralBool(false, token.span)),
            TokenKind::Str(s) => Ok(Expression::LiteralString(s, token.span)),
            TokenKind::Ident(name) => {
                if self.check(&TokenKind::LParen) {
                    // Function call
                    self.consume(&TokenKind::LParen, "Expected '('")?;
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression(0)?);
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenKind::RParen, "Expected ')'")?;
                    Ok(Expression::Call {
                        callee: name,
                        args,
                        span: token.span,
                    })
                } else {
                    Ok(Expression::Variable(name, token.span))
                }
            }
            TokenKind::LParen => {
                let expr = self.parse_expression(0)?;
                self.consume(&TokenKind::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            TokenKind::At => {
                // @load_i32(ptr)
                let op_tok = self.advance();
                let op_name = match op_tok.kind {
                    TokenKind::Ident(n) => n,
                    _ => return Err(format!("Expected memory load intrinsic after '@' at {}", token.span)),
                };

                let ty = match op_name.as_str() {
                    "load_i32" => Type::I32,
                    "load_i64" => Type::I64,
                    "load_f32" => Type::F32,
                    "load_f64" => Type::F64,
                    _ => return Err(format!("Unknown memory load intrinsic '@{}' at {}", op_name, token.span)),
                };

                self.consume(&TokenKind::LParen, "Expected '(' after load intrinsic")?;
                let ptr = self.parse_expression(0)?;
                self.consume(&TokenKind::RParen, "Expected ')' after pointer expression")?;

                Ok(Expression::MemoryLoad {
                    ty,
                    ptr: Box::new(ptr),
                    offset: 0,
                    span: token.span,
                })
            }
            _ => Err(format!("Unexpected token '{:?}' in expression at {}", token.kind, token.span)),
        }
    }

    fn peek_infix_op(&self) -> Option<(BinaryOp, u8, u8)> {
        let token = self.peek();
        match token.kind {
            TokenKind::Pipe => Some((BinaryOp::BitOr, 1, 2)),
            TokenKind::Caret => Some((BinaryOp::BitXor, 2, 3)),
            TokenKind::Amp => Some((BinaryOp::BitAnd, 3, 4)),
            TokenKind::EqEq => Some((BinaryOp::Eq, 4, 5)),
            TokenKind::BangEq => Some((BinaryOp::Ne, 4, 5)),
            TokenKind::Lt => Some((BinaryOp::Lt, 5, 6)),
            TokenKind::LtEq => Some((BinaryOp::Le, 5, 6)),
            TokenKind::Gt => Some((BinaryOp::Gt, 5, 6)),
            TokenKind::GtEq => Some((BinaryOp::Ge, 5, 6)),
            TokenKind::Shl => Some((BinaryOp::Shl, 6, 7)),
            TokenKind::Shr => Some((BinaryOp::Shr, 6, 7)),
            TokenKind::Plus => Some((BinaryOp::Add, 7, 8)),
            TokenKind::Minus => Some((BinaryOp::Sub, 7, 8)),
            TokenKind::Star => Some((BinaryOp::Mul, 9, 10)),
            TokenKind::Slash => Some((BinaryOp::Div, 9, 10)),
            TokenKind::Percent => Some((BinaryOp::Rem, 9, 10)),
            _ => None,
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.cursor += 1;
        }
        self.tokens[self.cursor - 1].clone()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: &TokenKind, err_msg: &str) -> Result<Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let tok = self.peek();
            Err(format!("{} (got '{:?}' at {})", err_msg, tok.kind, tok.span))
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

fn nested_if_span(s: &Statement) -> Span {
    match s {
        Statement::If { span, .. } => *span,
        _ => Span::new(0, 0, 0),
    }
}
