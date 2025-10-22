mod binary;
mod expressions;
mod patterns;
mod postfix;
mod statements;
mod utils;

use suji_ast::ast::Stmt;
use suji_lexer::token::{Span, Token, TokenWithSpan};
use suji_lexer::{LexError, Lexer};
use thiserror::Error;

/// Controls whether postfix operators (calls, indexing, field access, ++/--)
/// are allowed in the current expression parsing context.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ExpressionContext {
    Default,
    NoPostfix,
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
    tokens: Vec<TokenWithSpan>,
    current: usize,
    export_count: usize,
    // Current expression parsing context controlling postfix handling
    expression_context: ExpressionContext,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithSpan>) -> Self {
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

/// Parse a complete program from source code
pub fn parse_program(input: &str) -> ParseResult<Vec<Stmt>> {
    let tokens = Lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {
        let result = parse_program("42");
        assert!(result.is_ok());
    }
}
