use super::Expr;
use crate::span::Span;

/// Function parameter with optional default value
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
    pub span: Span,
}
