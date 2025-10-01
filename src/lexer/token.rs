use logos::Logos;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

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

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    #[token("return")]
    Return,
    #[token("loop")]
    Loop,
    #[token("as")]
    As,
    #[token("through")]
    Through,
    #[token("with")]
    With,
    #[token("continue")]
    Continue,
    #[token("break")]
    Break,
    #[token("match")]
    Match,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,

    // Literals
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().to_owned())]
    Number(String),

    // Wildcard pattern - must come before Identifier
    #[token("_")]
    Underscore,

    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
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
    #[token("=>")]
    FatArrow,
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("%")]
    Modulo,
    #[token("^")]
    Power,
    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,

    // Compound assignment operators
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    MultiplyAssign,
    #[token("/=")]
    DivideAssign,
    #[token("%=")]
    ModuloAssign,

    // Comparison operators
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    // Logical operators
    #[token("!")]
    Not,
    #[token("&&")]
    And,
    #[token("||")]
    Or,

    // Range operator
    #[token("..")]
    Range,

    // Regex match operators
    #[token("~")]
    RegexMatch,
    #[token("!~")]
    RegexNotMatch,

    // Punctuation
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("|")]
    Pipe,
    #[token("|>")]
    PipeForward,
    #[token("<|")]
    PipeBackward,
    #[token(";")]
    Semicolon,

    // Special tokens
    #[regex(r"#[^\r\n]*", |lex| lex.slice().to_owned())]
    Comment(String),

    #[regex(r"\r?\n", logos::skip)]
    Newline,

    #[regex(r"[ \t]+", logos::skip)]
    Whitespace,

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
