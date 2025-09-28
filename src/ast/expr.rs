use super::{BinaryOp, CompoundOp, Literal, MatchArm, Param, Stmt, StringPart, UnaryOp};
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

    /// Compound assignment: target += value, target -= value, etc.
    CompoundAssign {
        target: Box<Expr>,
        op: CompoundOp,
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

    /// Match expression: match expr? { pattern: expr, ... } or match { condition: expr, ... }
    Match {
        scrutinee: Option<Box<Expr>>, // None for conditional match, Some(expr) for traditional match
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Destructuring assignment target: (a, _, b)
    Destructure { elements: Vec<Expr>, span: Span },
}

// Literal moved to `crate::ast::literal` and re-exported via `mod.rs`

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
            Expr::CompoundAssign { span, .. } => span,
            Expr::MethodCall { span, .. } => span,
            Expr::Match { span, .. } => span,
            Expr::Destructure { span, .. } => span,
        }
    }

    /// Check if this expression is a valid assignment target
    pub fn is_assignable(&self) -> bool {
        matches!(
            self,
            Expr::Literal(Literal::Identifier(..))
                | Expr::Index { .. }
                | Expr::MapAccessByName { .. }
                | Expr::Destructure { .. }
        )
    }
}

// Literal span impl now lives with the type in `literal.rs`
