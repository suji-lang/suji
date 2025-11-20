use super::Expr;

/// Parts of a string template (for interpolation)
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Expr(Expr),
}

impl StringPart {
    /// Check if this string part contains control flow
    pub fn has_control_flow(&self) -> bool {
        match self {
            StringPart::Text(_) => false,
            StringPart::Expr(expr) => expr.has_control_flow(),
        }
    }
}
