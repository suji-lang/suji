use suji_ast::Stmt;
use suji_lexer::LexError;
use suji_lexer::{Span, SpannedToken, Token};
use thiserror::Error;

/// Controls which postfix operators are allowed in the current expression parsing context.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExpressionContext {
    /// Default context: all postfix operators allowed (calls, indexing, field access, ++/--, ::, :)
    Default,

    /// No postfix operators allowed at all.
    /// Used for map keys in `map:key` syntax to prevent ambiguity with nested colons.
    NoPostfix,

    /// All postfix operators allowed except single-colon map access (`:`).
    /// Used for index expressions inside `[]` to prevent ambiguity with slice syntax.
    NoColonAccess,
}

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Lexical error: {0}")]
    Lex(#[from] LexError),
    #[error("Unexpected token: {token:?} at {span:?}")]
    UnexpectedToken { token: Token, span: Span },
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Parse error: {message}")]
    Generic { message: String },
    #[error("Expected token {expected:?} but found {found:?} at {span:?}")]
    ExpectedToken {
        expected: Token,
        found: Token,
        span: Span,
    },
    #[error("Expected item name after ':'")]
    InvalidImportPath { span: Span },
    #[error("Expected alias name after 'as'")]
    InvalidAlias { span: Span },
    #[error("Multiple export statements found. Only one export statement is allowed per file.")]
    MultipleExports { span: Span },
}

/// Main parser result type
pub type ParseResult<T> = Result<T, ParseError>;

/// Simple recursive descent parser
pub struct Parser {
    pub(super) tokens: Vec<SpannedToken>,
    pub(super) current: usize,
    pub(super) export_count: usize,
    // Current expression parsing context controlling postfix handling
    pub(super) expression_context: ExpressionContext,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        let mut parser = Self {
            tokens,
            current: 0,
            export_count: 0,
            expression_context: ExpressionContext::Default,
        };
        parser.skip_newlines_and_comments();
        parser
    }

    /// Parse a program (sequence of statements)
    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);

            // Handle statement separators (semicolons and newlines)
            self.handle_statement_separator(false)?;
        }

        Ok(statements)
    }
}
