//! Abstract Syntax Tree (AST) definitions for Nova.

use crate::token::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    Bool,
    Void,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Void => write!(f, "void"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    LiteralInt(i64, Span),
    LiteralFloat(f64, Span),
    LiteralBool(bool, Span),
    LiteralString(String, Span),
    Variable(String, Span),
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
        span: Span,
    },
    Call {
        callee: String,
        args: Vec<Expression>,
        span: Span,
    },
    MemoryLoad {
        ty: Type,
        ptr: Box<Expression>,
        offset: u32,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::LiteralInt(_, s)
            | Expression::LiteralFloat(_, s)
            | Expression::LiteralBool(_, s)
            | Expression::LiteralString(_, s)
            | Expression::Variable(_, s)
            | Expression::Binary { span: s, .. }
            | Expression::Unary { span: s, .. }
            | Expression::Call { span: s, .. }
            | Expression::MemoryLoad { span: s, .. } => *s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        is_mut: bool,
        ty: Option<Type>,
        init: Expression,
        span: Span,
    },
    Assignment {
        target: String,
        value: Expression,
        span: Span,
    },
    MemoryStore {
        ty: Type,
        ptr: Expression,
        offset: u32,
        value: Expression,
        span: Span,
    },
    If {
        condition: Expression,
        then_branch: Block,
        else_branch: Option<Block>,
        span: Span,
    },
    While {
        condition: Expression,
        body: Block,
        span: Span,
    },
    Return {
        value: Option<Expression>,
        span: Span,
    },
    Break(Span),
    Expr {
        expr: Expression,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Block,
    pub is_export: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDecl {
    pub initial_pages: u32,
    pub max_pages: Option<u32>,
    pub is_export: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportFunc {
    pub module: String,
    pub field: String,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub imports: Vec<ImportFunc>,
    pub functions: Vec<Function>,
    pub memory: Option<MemoryDecl>,
}
