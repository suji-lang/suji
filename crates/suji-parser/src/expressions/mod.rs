pub mod assignments;
pub mod binary;
pub mod collections;
pub mod functions;
pub mod literals;
pub mod match_expr;
pub mod patterns;
pub mod postfix;
pub mod shell_cmd;

use crate::{ParseError, ParseResult, Parser};
use suji_ast::Expr;
use suji_lexer::Token;

impl Parser {
    /// Parse a primary expression - main dispatcher
    pub(super) fn primary(&mut self) -> ParseResult<Expr> {
        // Control flow expressions
        if self.match_token(Token::Return) {
            return self.parse_return_expr();
        }
        if self.match_token(Token::Break) {
            return self.parse_break_expr();
        }
        if self.match_token(Token::Continue) {
            return self.parse_continue_expr();
        }

        // Try parsing literals first
        if let Ok(expr) = self.parse_literals() {
            return Ok(expr);
        }

        // Allow tuple/grouping to appear in primary contexts (e.g., destructuring)
        if self.match_token(Token::LeftParen) {
            return self.parse_tuple_or_grouping();
        }

        // Try parsing collections
        if let Ok(expr) = self.parse_collections() {
            return Ok(expr);
        }

        // Try parsing functions
        if let Ok(expr) = self.parse_functions() {
            return Ok(expr);
        }

        // Try parsing match expressions
        if let Ok(expr) = self.parse_match_expression() {
            return Ok(expr);
        }

        // Try parsing shell commands
        if let Ok(expr) = self.parse_shell_command() {
            return Ok(expr);
        }

        // If none match, return error
        let current = self.peek();
        Err(ParseError::UnexpectedToken {
            token: current.token,
            span: current.span,
        })
    }

    /// Parse return expression: return expr*
    pub(super) fn parse_return_expr(&mut self) -> ParseResult<Expr> {
        let span = self.previous().span.clone();

        // If the next token ends the statement, this is a bare `return`
        if self.check(Token::Newline) || self.check(Token::Semicolon) || self.is_at_end() {
            return Ok(Expr::Return {
                values: Vec::new(),
                span,
            });
        }

        // Parse comma-separated expressions until newline/semicolon/end
        let mut values = Vec::new();
        loop {
            values.push(self.expression()?);

            if !self.match_token(Token::Comma) {
                break;
            }

            if self.check(Token::Newline) || self.check(Token::Semicolon) || self.is_at_end() {
                return Err(ParseError::Generic {
                    message: "Trailing comma not allowed in return".to_string(),
                });
            }
        }

        Ok(Expr::Return { values, span })
    }

    /// Parse break expression: break label?
    pub(super) fn parse_break_expr(&mut self) -> ParseResult<Expr> {
        let span = self.previous().span.clone();
        // Only consume label if it's on the same line as the break keyword
        let label = if let Token::Identifier(_) = &self.peek().token {
            let next_span = &self.peek().span;
            // Check if identifier is on the same line as break
            if span.line == next_span.line {
                let (name, _span) = self.consume_identifier()?;
                Some(name)
            } else {
                None
            }
        } else {
            None
        };
        Ok(Expr::Break { label, span })
    }

    /// Parse continue expression: continue label?
    pub(super) fn parse_continue_expr(&mut self) -> ParseResult<Expr> {
        let span = self.previous().span.clone();
        // Only consume label if it's on the same line as the continue keyword
        let label = if let Token::Identifier(_) = &self.peek().token {
            let next_span = &self.peek().span;
            // Check if identifier is on the same line as continue
            if span.line == next_span.line {
                let (name, _span) = self.consume_identifier()?;
                Some(name)
            } else {
                None
            }
        } else {
            None
        };
        Ok(Expr::Continue { label, span })
    }
}
