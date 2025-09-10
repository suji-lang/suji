use crate::token::Token;

/// Operator precedence levels (higher number = higher precedence)
/// Based on the specification in IMPLEMENTATION_PLAN.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 1,      // = (right-associative)
    LogicalOr = 2,       // ||
    LogicalAnd = 3,      // &&
    RegexMatch = 4,      // ~, !~
    Equality = 5,        // ==, !=
    Relational = 6,      // <, <=, >, >=
    Range = 7,           // ..
    Additive = 8,        // +, -
    Multiplicative = 9,  // *, /, %
    Unary = 10,          // -x, !x
    Exponentiation = 11, // ^ (right-associative)
    Postfix = 12,        // x++, x--, f(), x[i], x:key, x::method()
}

/// Associativity of operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

impl Precedence {
    /// Get the precedence of a token when used as a binary operator
    pub fn of_binary_op(token: &Token) -> Self {
        match token {
            Token::Assign
            | Token::PlusAssign
            | Token::MinusAssign
            | Token::MultiplyAssign
            | Token::DivideAssign
            | Token::ModuloAssign => Self::Assignment,
            Token::Or => Self::LogicalOr,
            Token::And => Self::LogicalAnd,
            Token::RegexMatch | Token::RegexNotMatch => Self::RegexMatch,
            Token::Equal | Token::NotEqual => Self::Equality,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                Self::Relational
            }
            Token::Range => Self::Range,
            Token::Plus | Token::Minus => Self::Additive,
            Token::Multiply | Token::Divide | Token::Modulo => Self::Multiplicative,
            Token::Power => Self::Exponentiation,
            _ => Self::None,
        }
    }

    /// Get the precedence of a token when used as a prefix unary operator
    pub fn of_unary_op(token: &Token) -> Self {
        match token {
            Token::Minus | Token::Not => Self::Unary,
            _ => Self::None,
        }
    }

    /// Get the precedence of postfix operators
    pub fn of_postfix_op(token: &Token) -> Self {
        match token {
            Token::Increment
            | Token::Decrement
            | Token::LeftParen
            | Token::LeftBracket
            | Token::Colon
            | Token::DoubleColon => Self::Postfix,
            _ => Self::None,
        }
    }

    /// Get the associativity of an operator at this precedence level
    pub fn associativity(self) -> Associativity {
        match self {
            Self::Assignment | Self::Exponentiation => Associativity::Right,
            _ => Associativity::Left,
        }
    }

    /// Get the next higher precedence level
    pub fn next_higher(self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::LogicalOr,
            Self::LogicalOr => Self::LogicalAnd,
            Self::LogicalAnd => Self::RegexMatch,
            Self::RegexMatch => Self::Equality,
            Self::Equality => Self::Relational,
            Self::Relational => Self::Range,
            Self::Range => Self::Additive,
            Self::Additive => Self::Multiplicative,
            Self::Multiplicative => Self::Unary,
            Self::Unary => Self::Exponentiation,
            Self::Exponentiation => Self::Postfix,
            Self::Postfix => Self::Postfix, // Highest level
        }
    }

    /// Check if this precedence level can bind to the right
    /// Used for precedence climbing with right-associative operators
    pub fn binds_right(self, other: Self) -> bool {
        match self.associativity() {
            Associativity::Left => self > other,
            Associativity::Right => self >= other,
        }
    }
}
