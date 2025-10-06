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
    Pipe,         // |
    PipeApplyFwd, // |>
    PipeApplyBwd, // <|

    // Function composition
    ComposeRight, // >>
    ComposeLeft,  // <<

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
