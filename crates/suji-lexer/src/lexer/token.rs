pub use suji_ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

impl TokenWithSpan {
    pub fn new(token: Token, span: Span) -> Self {
        Self { token, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Return,
    Loop,
    As,
    Through,
    With,
    Continue,
    Break,
    Match,
    Import,
    Export,
    True,
    False,
    Nil,

    // Literals
    Number(String),

    // Wildcard pattern - must come before Identifier
    Underscore,

    Identifier(String),

    // String template tokens
    StringStart,
    StringText(String),
    InterpStart, // ${
    InterpEnd,   // } (implicit when closing interpolation)
    StringEnd,

    // Shell command template tokens (reuse string interpolation tokens)
    ShellStart, // `
    ShellEnd,   // `

    // Regex tokens
    RegexStart, // /
    RegexContent(String),
    RegexEnd, // /

    // Operators
    FatArrow,
    Assign,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Power,
    Increment,
    Decrement,

    // Compound assignment operators
    PlusAssign,
    MinusAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,

    // Comparison operators
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical operators
    Not,
    And,
    Or,

    // Function composition operators
    ComposeRight,
    ComposeLeft,

    // Range operator
    Range,

    // Regex match operators
    RegexMatch,
    RegexNotMatch,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    DoubleColon,
    Pipe,
    PipeForward,
    PipeBackward,
    Semicolon,

    // Special tokens
    Comment(String),
    Newline,

    // End of file
    Eof,

    // Error token
    Error,
}

impl Token {
    /// Returns true if this token can end an expression (used for regex/division disambiguation)
    pub fn can_end_expression(&self) -> bool {
        matches!(
            self,
            Token::Identifier(_)
                | Token::Number(_)
                | Token::StringEnd
                | Token::ShellEnd
                | Token::RegexEnd
                | Token::RightParen
                | Token::RightBracket
                | Token::RightBrace
                | Token::Increment
                | Token::Decrement
                | Token::True
                | Token::False
        )
    }

    /// Returns true if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Return
                | Token::Loop
                | Token::As
                | Token::Through
                | Token::With
                | Token::Continue
                | Token::Break
                | Token::Match
                | Token::Import
                | Token::Export
                | Token::True
                | Token::False
                | Token::Nil
        )
    }

    /// Returns true if this token is an operator
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            Token::Assign
                | Token::PlusAssign
                | Token::MinusAssign
                | Token::MultiplyAssign
                | Token::DivideAssign
                | Token::ModuloAssign
                | Token::Plus
                | Token::Minus
                | Token::Multiply
                | Token::Divide
                | Token::Modulo
                | Token::Power
                | Token::Increment
                | Token::Decrement
                | Token::Equal
                | Token::NotEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual
                | Token::Not
                | Token::And
                | Token::Or
                | Token::Range
                | Token::RegexMatch
                | Token::RegexNotMatch
        )
    }
}
