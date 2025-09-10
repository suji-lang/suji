pub mod binary;
pub mod expr;
pub mod literals;
pub mod pattern;
pub mod postfix;
pub mod precedence;
pub mod primary;
pub mod statements;
pub mod stmt;
pub mod utils;

use crate::ast::{Expr, Stmt};
use crate::lexer::{LexError, Lexer};
use crate::token::{Span, Token, TokenWithSpan};
use thiserror::Error;

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
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithSpan>) -> Self {
        let mut parser = Self {
            tokens,
            current: 0,
            export_count: 0,
        };
        parser.skip_newlines_and_comments();
        parser
    }

    /// Parse pattern for match statements
    pub(super) fn parse_pattern(&mut self) -> ParseResult<crate::ast::Pattern> {
        // For now, implement basic patterns - can be expanded later
        if self.match_token(Token::Underscore) {
            let span = self.previous().span.clone();
            return Ok(crate::ast::Pattern::Wildcard { span });
        }

        if self.match_token(Token::LeftParen) {
            // Tuple pattern
            let span = self.previous().span.clone();
            let mut patterns = Vec::new();

            if !self.check(Token::RightParen) {
                loop {
                    patterns.push(self.parse_pattern()?);
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                    if self.check(Token::RightParen) {
                        break;
                    }
                }
            }

            self.consume(Token::RightParen, "Expected ')' after tuple pattern")?;
            return Ok(crate::ast::Pattern::Tuple { patterns, span });
        }

        if self.match_token(Token::RegexStart) {
            // Regex pattern
            let span = self.previous().span.clone();
            if let Token::RegexContent(pattern) = &self.peek().token {
                let pattern = pattern.clone();
                self.advance();
                self.consume(Token::RegexEnd, "Expected end of regex pattern")?;
                return Ok(crate::ast::Pattern::Regex { pattern, span });
            } else {
                return Err(ParseError::Generic {
                    message: "Expected regex content".to_string(),
                });
            }
        }

        // Literal pattern - parse simple literals directly to avoid `:` conflicts
        if let Token::Number(n) = &self.peek().token {
            let n = *n;
            let span = self.advance().span.clone();
            return Ok(crate::ast::Pattern::Literal {
                value: crate::ast::ValueLike::Number(n),
                span,
            });
        }

        if self.match_token(Token::True) {
            let span = self.previous().span.clone();
            return Ok(crate::ast::Pattern::Literal {
                value: crate::ast::ValueLike::Boolean(true),
                span,
            });
        }

        if self.match_token(Token::False) {
            let span = self.previous().span.clone();
            return Ok(crate::ast::Pattern::Literal {
                value: crate::ast::ValueLike::Boolean(false),
                span,
            });
        }

        if self.match_token(Token::Nil) {
            let span = self.previous().span.clone();
            return Ok(crate::ast::Pattern::Literal {
                value: crate::ast::ValueLike::Nil,
                span,
            });
        }

        if self.match_token(Token::StringStart) {
            // String literal pattern
            let span = self.previous().span.clone();
            let mut parts = Vec::new();

            while !self.check(Token::StringEnd) && !self.is_at_end() {
                if let Token::StringText(text) = &self.peek().token {
                    let text = text.clone();
                    self.advance();
                    parts.push(crate::ast::StringPart::Text(text));
                } else if self.match_token(Token::InterpStart) {
                    let expr = self.expression()?;
                    parts.push(crate::ast::StringPart::Expr(expr));
                    self.consume(Token::InterpEnd, "Expected '}' after string interpolation")?;
                } else {
                    return Err(ParseError::Generic {
                        message: "Expected string content or interpolation".to_string(),
                    });
                }
            }

            self.consume(Token::StringEnd, "Expected end of string literal")?;

            // For pattern matching, we only support simple string literals (no interpolation)
            if parts.len() == 1
                && let crate::ast::StringPart::Text(text) = &parts[0]
            {
                return Ok(crate::ast::Pattern::Literal {
                    value: crate::ast::ValueLike::String(text.clone()),
                    span,
                });
            }

            return Err(ParseError::Generic {
                message: "String patterns cannot contain interpolation".to_string(),
            });
        }

        if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            let span = self.advance().span.clone();
            return Ok(crate::ast::Pattern::Literal {
                value: crate::ast::ValueLike::String(name),
                span,
            });
        }

        // If we can't parse a simple pattern, fall back to error
        let current = self.peek();
        Err(ParseError::UnexpectedToken {
            token: current.token,
            span: current.span,
        })
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

/// Parse a single expression from source code
pub fn parse_expression(input: &str) -> ParseResult<Expr> {
    let tokens = Lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.expression()
}

/// Parse a single statement
pub fn parse_statement(input: &str) -> ParseResult<Stmt> {
    let tokens = Lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.statement()
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
