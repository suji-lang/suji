// Expression parser - simplified implementation for Phase 3
// This module provides placeholder functions that match the expected API
// but uses the main parser in mod.rs for actual implementation

use crate::ast::{Expr, Literal};
use crate::token::Span;

/// Placeholder function for expression parsing
/// The actual implementation is in the main parser (mod.rs)
pub fn expr_parser() -> impl Fn() -> Expr {
    || Expr::Literal(Literal::Number(0.0, dummy_span()))
}

/// Helper to create dummy spans
pub fn dummy_span() -> Span {
    Span::new(0, 0, 0, 0)
}
