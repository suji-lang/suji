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

    // Pipe
    Pipe, // |

    // Range
    Range, // ..

    // Regex matching
    RegexMatch,    // ~
    RegexNotMatch, // !~
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
