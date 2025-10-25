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

    /// Return expression: return expr_list?
    /// Empty list represents `return` with no value
    Return { values: Vec<Expr>, span: Span },

    /// Break expression: break label?
    Break { label: Option<String>, span: Span },

    /// Continue expression: continue label?
    Continue { label: Option<String>, span: Span },
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
            Expr::CompoundAssign { span, .. } => span,
            Expr::MethodCall { span, .. } => span,
            Expr::Match { span, .. } => span,
            Expr::Destructure { span, .. } => span,
            Expr::Return { span, .. } => span,
            Expr::Break { span, .. } => span,
            Expr::Continue { span, .. } => span,
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

    /// Check if this expression contains control flow (return, break, continue)
    pub fn has_control_flow(&self) -> bool {
        match self {
            Expr::Return { .. } | Expr::Break { .. } | Expr::Continue { .. } => true,
            Expr::Unary { expr, .. } => expr.has_control_flow(),
            Expr::Binary { left, right, .. } => left.has_control_flow() || right.has_control_flow(),
            Expr::PostfixIncrement { target, .. } | Expr::PostfixDecrement { target, .. } => {
                target.has_control_flow()
            }
            Expr::Call { callee, args, .. } => {
                callee.has_control_flow() || args.iter().any(|arg| arg.has_control_flow())
            }
            Expr::Grouping { expr, .. } => expr.has_control_flow(),
            // FunctionLiteral: return false - control flow inside functions doesn't escape
            Expr::FunctionLiteral { .. } => false,
            Expr::Index { target, index, .. } => {
                target.has_control_flow() || index.has_control_flow()
            }
            Expr::Slice {
                target, start, end, ..
            } => {
                target.has_control_flow()
                    || start.as_deref().is_some_and(|e| e.has_control_flow())
                    || end.as_deref().is_some_and(|e| e.has_control_flow())
            }
            Expr::MapAccessByName { target, .. } => target.has_control_flow(),
            Expr::Assign { target, value, .. } => {
                target.has_control_flow() || value.has_control_flow()
            }
            Expr::CompoundAssign { target, value, .. } => {
                target.has_control_flow() || value.has_control_flow()
            }
            Expr::MethodCall { target, args, .. } => {
                target.has_control_flow() || args.iter().any(|arg| arg.has_control_flow())
            }
            Expr::Match { arms, .. } => arms.iter().any(|arm| arm.body.has_control_flow()),
            Expr::Destructure { elements, .. } => {
                elements.iter().any(|elem| elem.has_control_flow())
            }
            Expr::ShellCommandTemplate { parts, .. } => parts
                .iter()
                .any(|part| matches!(part, StringPart::Expr(e) if e.has_control_flow())),
            Expr::Literal(lit) => lit.has_control_flow(),
        }
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

            // Return: from values
            Expr::Return { values, span, .. } => {
                if let Some(last_value) = values.last() {
                    let last_span = last_value.covering_span();
                    combine_spans(span, &last_span)
                } else {
                    span.clone()
                }
            }

            // Break: from label
            Expr::Break { label, span, .. } => {
                if label.is_some() {
                    // Label text is typically part of the source span already
                    span.clone()
                } else {
                    span.clone()
                }
            }

            // Continue: from label
            Expr::Continue { label, span, .. } => {
                if label.is_some() {
                    // Label text is typically part of the source span already
                    span.clone()
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
    let start = a.start.min(b.start);
    let end = a.end.max(b.end);
    // Select line/column from the span whose start equals the chosen earliest start
    let (line, column) = if a.start <= b.start {
        (a.line, a.column)
    } else {
        (b.line, b.column)
    };
    Span {
        start,
        end,
        line,
        column,
    }
}

/// Helper to combine three spans
fn combine_three_spans(a: &Span, b: &Span, c: &Span) -> Span {
    // Compute overall start/end
    let start = a.start.min(b.start.min(c.start));
    let end = a.end.max(b.end.max(c.end));
    // Choose the earliest-starting original span for line/column; tie-breaker: a, then b, then c
    let (line, column) = if a.start <= b.start && a.start <= c.start {
        (a.line, a.column)
    } else if b.start <= a.start && b.start <= c.start {
        (b.line, b.column)
    } else {
        (c.line, c.column)
    };
    Span {
        start,
        end,
        line,
        column,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_span(start: usize, end: usize, line: usize, column: usize) -> Span {
        Span {
            start,
            end,
            line,
            column,
        }
    }

    fn num(n: &str, s: Span) -> Expr {
        Expr::Literal(Literal::Number(n.to_string(), s))
    }

    #[test]
    fn covering_span_cases() {
        // Unary
        let op_span = mk_span(10, 11, 2, 5);
        let child_span = mk_span(0, 1, 1, 1);
        let expr = Expr::Unary {
            op: UnaryOp::Negate,
            expr: Box::new(num("1", child_span)),
            span: op_span.clone(),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 0);
        assert_eq!(cov.end, 11);
        assert_eq!((cov.line, cov.column), (1, 1));

        // Binary with earliest-start on right (negative case)
        let left = num("1", mk_span(20, 21, 3, 7));
        let right = num("2", mk_span(5, 6, 10, 20));
        let op_span = mk_span(15, 16, 3, 10);
        let expr = Expr::Binary {
            left: Box::new(left),
            op: BinaryOp::Add,
            right: Box::new(right),
            span: op_span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 5);
        assert_eq!(cov.end, 21);
        assert_eq!((cov.line, cov.column), (10, 20));

        // Call: no args
        let callee = num("1", mk_span(2, 3, 1, 3));
        let call_span = mk_span(4, 5, 1, 5);
        let expr = Expr::Call {
            callee: Box::new(callee),
            args: vec![],
            span: call_span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 2);
        assert_eq!(cov.end, 5);
        assert_eq!((cov.line, cov.column), (1, 3));

        // Call: with one arg
        let callee = num("1", mk_span(10, 11, 2, 1));
        let arg = num("2", mk_span(30, 31, 4, 2));
        let call_span = mk_span(20, 21, 3, 5);
        let expr = Expr::Call {
            callee: Box::new(callee),
            args: vec![arg],
            span: call_span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 10);
        assert_eq!(cov.end, 31);
        assert_eq!((cov.line, cov.column), (2, 1));

        // Method call
        let target = num("1", mk_span(5, 6, 1, 1));
        let arg = num("2", mk_span(25, 26, 2, 10));
        let call_span = mk_span(15, 16, 1, 6);
        let expr = Expr::MethodCall {
            target: Box::new(target),
            method: "m".to_string(),
            args: vec![arg],
            span: call_span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 5);
        assert_eq!(cov.end, 26);
        assert_eq!((cov.line, cov.column), (1, 1));

        // Index
        let target = num("1", mk_span(1, 2, 1, 1));
        let idx = num("0", mk_span(10, 11, 1, 10));
        let span = mk_span(5, 6, 1, 5);
        let expr = Expr::Index {
            target: Box::new(target),
            index: Box::new(idx),
            span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 1);
        assert_eq!(cov.end, 11);
        assert_eq!((cov.line, cov.column), (1, 1));

        // Slice end-only
        let target = num("1", mk_span(1, 2, 1, 1));
        let end = num("3", mk_span(20, 21, 2, 2));
        let span = mk_span(5, 6, 1, 5);
        let expr = Expr::Slice {
            target: Box::new(target),
            start: None,
            end: Some(Box::new(end)),
            span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 1);
        assert_eq!(cov.end, 21);
        assert_eq!((cov.line, cov.column), (1, 1));

        // Assign
        let target = num("x", mk_span(30, 31, 3, 1));
        let value = num("1", mk_span(40, 41, 4, 2));
        let span = mk_span(35, 36, 3, 5);
        let expr = Expr::Assign {
            target: Box::new(target),
            value: Box::new(value),
            span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 30);
        assert_eq!(cov.end, 41);
        assert_eq!((cov.line, cov.column), (3, 1));

        // Compound assign
        let target = num("x", mk_span(30, 31, 3, 1));
        let value = num("1", mk_span(50, 51, 5, 2));
        let span = mk_span(40, 41, 4, 5);
        let expr = Expr::CompoundAssign {
            target: Box::new(target),
            op: CompoundOp::PlusAssign,
            value: Box::new(value),
            span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 30);
        assert_eq!(cov.end, 51);
        assert_eq!((cov.line, cov.column), (3, 1));

        // Destructure empty
        let span = mk_span(10, 11, 2, 3);
        let expr = Expr::Destructure {
            elements: vec![],
            span: span.clone(),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 10);
        assert_eq!(cov.end, 11);
        assert_eq!((cov.line, cov.column), (2, 3));

        // Destructure non-empty
        let first = num("a", mk_span(5, 6, 1, 1));
        let last = num("b", mk_span(25, 26, 2, 2));
        let span = mk_span(15, 16, 1, 6);
        let expr = Expr::Destructure {
            elements: vec![first, last],
            span,
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 5);
        assert_eq!(cov.end, 26);
        assert_eq!((cov.line, cov.column), (1, 1));

        // Return empty
        let span = mk_span(10, 11, 2, 3);
        let expr = Expr::Return {
            values: vec![],
            span: span.clone(),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 10);
        assert_eq!(cov.end, 11);
        assert_eq!((cov.line, cov.column), (2, 3));

        // Return non-empty
        let value = num("1", mk_span(20, 21, 3, 4));
        let span = mk_span(15, 16, 1, 6);
        let expr = Expr::Return {
            values: vec![value],
            span: span.clone(),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 15);
        assert_eq!(cov.end, 21);
        assert_eq!((cov.line, cov.column), (1, 6));

        // Break
        let span = mk_span(10, 11, 2, 3);
        let expr = Expr::Break {
            label: None,
            span: span.clone(),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 10);
        assert_eq!(cov.end, 11);
        assert_eq!((cov.line, cov.column), (2, 3));

        // Break with label - span already encompasses the label
        let label = "loop".to_string();
        let expr = Expr::Break {
            label: Some(label),
            span: mk_span(10, 15, 2, 3),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 10);
        assert_eq!(cov.end, 15);
        assert_eq!((cov.line, cov.column), (2, 3));

        // Continue
        let span = mk_span(10, 11, 2, 3);
        let expr = Expr::Continue {
            label: None,
            span: span.clone(),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 10);
        assert_eq!(cov.end, 11);
        assert_eq!((cov.line, cov.column), (2, 3));

        // Continue with label - span already encompasses the label
        let label = "loop".to_string();
        let expr = Expr::Continue {
            label: Some(label),
            span: mk_span(10, 15, 2, 3),
        };
        let cov = expr.covering_span();
        assert_eq!(cov.start, 10);
        assert_eq!(cov.end, 15);
        assert_eq!((cov.line, cov.column), (2, 3));
    }
}
