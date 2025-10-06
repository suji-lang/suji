use super::Expr;
use crate::span::Span;

/// Import specification
#[derive(Debug, Clone, PartialEq)]
pub enum ImportSpec {
    Module {
        name: String,
    }, // import module
    Item {
        module: String,
        name: String,
    }, // import module:item
    ItemAs {
        module: String,
        name: String,
        alias: String,
    }, // import module:item as alias
}

/// Export specification - maps names to expressions
#[derive(Debug, Clone, PartialEq)]
pub struct ExportSpec {
    pub items: Vec<(String, Expr)>, // name: expression pairs
    pub span: Span,
}
