use super::{ExportBody, Expr, ImportSpec, LoopBindings, Pattern};
use suji_lexer::Span;

/// Statement nodes representing all SUJI language statements
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statement: expr;
    Expr(Expr),

    /// Block statement: { stmt1; stmt2; ... }
    Block { statements: Vec<Stmt>, span: Span },

    /// Infinite loop: loop (as label)? { ... }
    Loop {
        label: Option<String>,
        body: Box<Stmt>,
        span: Span,
    },

    /// Loop through iterable: loop through expr (with bindings)? (as label)? { ... }
    LoopThrough {
        label: Option<String>,
        iterable: Expr,
        bindings: LoopBindings,
        body: Box<Stmt>,
        span: Span,
    },

    /// Import statement: import spec
    Import { spec: ImportSpec, span: Span },

    /// Export statement: supports map form or expression form
    Export { body: ExportBody, span: Span },
}

/// A single match arm: pattern: statement
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Stmt,
    pub span: Span,
}

impl Stmt {
    /// Get the span of this statement
    pub fn span(&self) -> &Span {
        match self {
            Stmt::Expr(expr) => expr.span(),
            Stmt::Block { span, .. } => span,
            Stmt::Loop { span, .. } => span,
            Stmt::LoopThrough { span, .. } => span,
            Stmt::Import { span, .. } => span,
            Stmt::Export { span, .. } => span,
        }
    }

    /// Check if this statement is a block statement
    pub fn is_block(&self) -> bool {
        matches!(self, Stmt::Block { .. })
    }

    /// Check if this statement contains control flow (break/continue/return)
    pub fn has_control_flow(&self) -> bool {
        match self {
            Stmt::Expr(expr) => expr.has_control_flow(),
            Stmt::Block { statements, .. } => statements.iter().any(|stmt| stmt.has_control_flow()),
            Stmt::Loop { body, .. } | Stmt::LoopThrough { body, .. } => body.has_control_flow(),
            _ => false,
        }
    }
}
