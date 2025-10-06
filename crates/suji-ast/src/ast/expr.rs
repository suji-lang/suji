use super::{BinaryOp, CompoundOp, Literal, MatchArm, Param, Stmt, StringPart, UnaryOp};
use crate::span::Span;

/// Expression nodes representing all SUJI language expressions
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

    /// Compute a covering span that encompasses this entire expression.
    /// For multi-token expressions (like binary ops, method calls), this
    /// returns a span from the leftmost child's start to the rightmost child's end.
    pub fn covering_span(&self) -> Span {
        match self {
            // Literal uses its own span
            Expr::Literal(lit) => lit.span().clone(),

            // Unary: combine operator span with child expression
            Expr::Unary { expr, span, .. } => {
                let child_span = expr.covering_span();
                combine_spans(span, &child_span)
            }

            // Binary: combine left, operator, and right
            Expr::Binary {
                left, right, span, ..
            } => {
                let left_span = left.covering_span();
                let right_span = right.covering_span();
                combine_three_spans(&left_span, span, &right_span)
            }

            // Call: from callee to last arg (or closing paren)
            Expr::Call {
                callee, args, span, ..
            } => {
                let callee_span = callee.covering_span();
                if let Some(last_arg) = args.last() {
                    let last_span = last_arg.covering_span();
                    combine_three_spans(&callee_span, span, &last_span)
                } else {
                    combine_spans(&callee_span, span)
                }
            }

            // MethodCall: from target to last arg (or closing paren)
            Expr::MethodCall {
                target, args, span, ..
            } => {
                let target_span = target.covering_span();
                if let Some(last_arg) = args.last() {
                    let last_span = last_arg.covering_span();
                    combine_three_spans(&target_span, span, &last_span)
                } else {
                    combine_spans(&target_span, span)
                }
            }

            // Index: from target through index
            Expr::Index {
                target,
                index,
                span,
                ..
            } => {
                let target_span = target.covering_span();
                let index_span = index.covering_span();
                combine_three_spans(&target_span, span, &index_span)
            }

            // Slice: from target through end (if present) or start (if present)
            Expr::Slice {
                target,
                start,
                end,
                span,
                ..
            } => {
                let target_span = target.covering_span();
                let last_span = if let Some(e) = end {
                    e.covering_span()
                } else if let Some(s) = start {
                    s.covering_span()
                } else {
                    span.clone()
                };
                combine_three_spans(&target_span, span, &last_span)
            }

            // MapAccessByName: from target through key
            Expr::MapAccessByName { target, span, .. } => {
                let target_span = target.covering_span();
                combine_spans(&target_span, span)
            }

            // Assign: from target through value
            Expr::Assign {
                target,
                value,
                span,
                ..
            } => {
                let target_span = target.covering_span();
                let value_span = value.covering_span();
                combine_three_spans(&target_span, span, &value_span)
            }

            // CompoundAssign: from target through value
            Expr::CompoundAssign {
                target,
                value,
                span,
                ..
            } => {
                let target_span = target.covering_span();
                let value_span = value.covering_span();
                combine_three_spans(&target_span, span, &value_span)
            }

            // Grouping: use the inner expression's covering span
            Expr::Grouping { expr, span, .. } => {
                let expr_span = expr.covering_span();
                combine_spans(span, &expr_span)
            }

            // Destructure: from first to last element
            Expr::Destructure { elements, span, .. } => {
                if let (Some(first), Some(last)) = (elements.first(), elements.last()) {
                    let first_span = first.covering_span();
                    let last_span = last.covering_span();
                    combine_three_spans(&first_span, span, &last_span)
                } else {
                    span.clone()
                }
            }

            // These variants already have comprehensive spans
            Expr::PostfixIncrement { span, .. }
            | Expr::PostfixDecrement { span, .. }
            | Expr::FunctionLiteral { span, .. }
            | Expr::ShellCommandTemplate { span, .. }
            | Expr::Match { span, .. } => span.clone(),
        }
    }
}

/// Helper to combine two spans into a covering span
fn combine_spans(a: &Span, b: &Span) -> Span {
    Span {
        start: a.start.min(b.start),
        end: a.end.max(b.end),
        line: a.line.min(b.line),
        column: a.column.min(b.column),
    }
}

/// Helper to combine three spans
fn combine_three_spans(a: &Span, b: &Span, c: &Span) -> Span {
    combine_spans(&combine_spans(a, b), c)
}

// Literal span impl now lives with the type in `literal.rs`
