// Pattern parser - simplified implementation for Phase 3
// This module provides placeholder functions that match the expected API

use crate::ast::Pattern;

/// Placeholder function for pattern parsing
/// This will be implemented properly in later phases
pub fn pattern_parser() -> impl Fn() -> Pattern {
    use crate::token::Span;
    || Pattern::Wildcard {
        span: Span::new(0, 0, 0, 0),
    }
}
