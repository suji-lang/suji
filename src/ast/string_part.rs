use super::Expr;

/// Parts of a string template (for interpolation)
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Expr(Expr),
}
