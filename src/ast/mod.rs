use crate::token::Span;

pub mod expr;
pub mod pattern;
pub mod stmt;

// Re-export main types for easy access
pub use expr::*;
pub use pattern::*;
pub use stmt::*;

/// Common operators used in expressions
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate, // -
    Not,    // !
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // ^

    // Comparison
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // Logical
    And, // &&
    Or,  // ||

    // Range
    Range, // ..

    // Regex matching
    RegexMatch,    // ~
    RegexNotMatch, // !~
}

/// Parts of a string template (for interpolation)
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Expr(Expr),
}

/// Function parameter with optional default value
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
    pub span: Span,
}

/// Loop variable bindings for `loop through` statements
#[derive(Debug, Clone, PartialEq)]
pub enum LoopBindings {
    None,                // loop through iterable { ... }
    One(String),         // loop through iterable with x { ... }
    Two(String, String), // loop through iterable with k, v { ... }
}

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

/// Literal values that can be used in patterns
#[derive(Debug, Clone, PartialEq)]
pub enum ValueLike {
    Number(f64),
    Boolean(bool),
    String(String),
    Tuple(Vec<ValueLike>),
    Nil,
}

/// Compound assignment operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompoundOp {
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
    ModuloAssign,   // %=
}

impl CompoundOp {
    pub fn from_token(token: &crate::token::Token) -> Option<Self> {
        match token {
            crate::token::Token::PlusAssign => Some(CompoundOp::PlusAssign),
            crate::token::Token::MinusAssign => Some(CompoundOp::MinusAssign),
            crate::token::Token::MultiplyAssign => Some(CompoundOp::MultiplyAssign),
            crate::token::Token::DivideAssign => Some(CompoundOp::DivideAssign),
            crate::token::Token::ModuloAssign => Some(CompoundOp::ModuloAssign),
            _ => None,
        }
    }
}
