use super::{BinaryOp, Param, Stmt, StringPart, UnaryOp};
use crate::token::Span;

/// Expression nodes representing all NN language expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal values
    Literal(Literal),

    /// Unary operations: -x, !x
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    /// Binary operations: x + y, x == y, etc.
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Postfix increment: x++
    PostfixIncrement { target: Box<Expr>, span: Span },

    /// Postfix decrement: x--
    PostfixDecrement { target: Box<Expr>, span: Span },

    /// Function call: f(a, b, c)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    /// Parenthesized expression: (expr)
    Grouping { expr: Box<Expr>, span: Span },

    /// Function literal: |x, y| { return x + y }
    FunctionLiteral {
        params: Vec<Param>,
        body: Box<Stmt>,
        span: Span,
    },

    /// Shell command template: `echo ${name}`
    ShellCommandTemplate { parts: Vec<StringPart>, span: Span },

    /// Array/map indexing: list[i], map[key]
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    /// List slicing: list[start:end]
    Slice {
        target: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        span: Span,
    },

    /// Map access by name: map:key (where key is converted to string)
    MapAccessByName {
        target: Box<Expr>,
        key: String,
        span: Span,
    },

    /// Assignment: target = value
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    /// Method call: receiver::method(args)
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
}

/// Literal expression values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Numeric literal: 42, 3.14
    Number(f64, Span),

    /// Boolean literal: true, false
    Boolean(bool, Span),

    /// Identifier: variable_name
    Identifier(String, Span),

    /// String template with interpolation: "Hello ${name}!"
    StringTemplate(Vec<StringPart>, Span),

    /// List literal: [1, 2, 3]
    List(Vec<Expr>, Span),

    /// Map literal: { key: value, ... }
    Map(Vec<(Expr, Expr)>, Span),

    /// Tuple literal: (a, b, c)
    Tuple(Vec<Expr>, Span),

    /// Regex literal: /pattern/
    RegexLiteral(String, Span),
}

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> &Span {
        match self {
            Expr::Literal(lit) => lit.span(),
            Expr::Unary { span, .. } => span,
            Expr::Binary { span, .. } => span,
            Expr::PostfixIncrement { span, .. } => span,
            Expr::PostfixDecrement { span, .. } => span,
            Expr::Call { span, .. } => span,
            Expr::Grouping { span, .. } => span,
            Expr::FunctionLiteral { span, .. } => span,
            Expr::ShellCommandTemplate { span, .. } => span,
            Expr::Index { span, .. } => span,
            Expr::Slice { span, .. } => span,
            Expr::MapAccessByName { span, .. } => span,
            Expr::Assign { span, .. } => span,
            Expr::MethodCall { span, .. } => span,
        }
    }

    /// Check if this expression is a valid assignment target
    pub fn is_assignable(&self) -> bool {
        match self {
            Expr::Literal(Literal::Identifier(..)) => true,
            Expr::Index { .. } => true,
            Expr::MapAccessByName { .. } => true,
            _ => false,
        }
    }
}

impl Literal {
    /// Get the span of this literal
    pub fn span(&self) -> &Span {
        match self {
            Literal::Number(_, span) => span,
            Literal::Boolean(_, span) => span,
            Literal::Identifier(_, span) => span,
            Literal::StringTemplate(_, span) => span,
            Literal::List(_, span) => span,
            Literal::Map(_, span) => span,
            Literal::Tuple(_, span) => span,
            Literal::RegexLiteral(_, span) => span,
        }
    }
}
