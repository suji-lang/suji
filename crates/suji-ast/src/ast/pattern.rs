use super::{Expr, ValueLike};
use crate::span::Span;

/// Pattern nodes for match expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Literal value pattern: 42, true, "hello"
    Literal { value: ValueLike, span: Span },

    /// Tuple pattern with sub-patterns: (x, _, "hello")
    Tuple { patterns: Vec<Pattern>, span: Span },

    /// Regex pattern: /pattern/
    Regex { pattern: String, span: Span },

    /// Wildcard pattern: _
    Wildcard { span: Span },

    /// Expression pattern for conditional match: condition: body
    Expression(Expr),
}

impl Pattern {
    /// Get the span of this pattern
    pub fn span(&self) -> &Span {
        match self {
            Pattern::Literal { span, .. } => span,
            Pattern::Tuple { span, .. } => span,
            Pattern::Regex { span, .. } => span,
            Pattern::Wildcard { span, .. } => span,
            Pattern::Expression(expr) => expr.span(),
        }
    }

    /// Check if this pattern is exhaustive (matches everything)
    pub fn is_exhaustive(&self) -> bool {
        match self {
            Pattern::Wildcard { .. } => true,
            Pattern::Tuple { patterns, .. } => patterns.iter().all(|p| p.is_exhaustive()),
            Pattern::Expression(_) => false, // Expression patterns are never exhaustive
            _ => false,
        }
    }

    /// Check if this pattern can match the given value type
    pub fn can_match_value(&self, value: &ValueLike) -> bool {
        match (self, value) {
            (Pattern::Wildcard { .. }, _) => true,
            (
                Pattern::Literal {
                    value: pattern_val, ..
                },
                val,
            ) => std::mem::discriminant(pattern_val) == std::mem::discriminant(val),
            (Pattern::Tuple { patterns, .. }, ValueLike::Tuple(values)) => {
                patterns.len() == values.len()
                    && patterns
                        .iter()
                        .zip(values.iter())
                        .all(|(p, v)| p.can_match_value(v))
            }
            (Pattern::Regex { .. }, ValueLike::String(_)) => true,
            (Pattern::Expression(_), _) => true, // Expression patterns can match any value (they evaluate to boolean)
            _ => false,
        }
    }
}
