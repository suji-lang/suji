use super::{ParseError, ParseResult, Parser};
use crate::ast::{Expr, Literal};
use crate::token::Token;

impl Parser {
    /// Parse tuple or grouping expression (disambiguation by comma)
    pub(super) fn parse_tuple_or_grouping(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();

        // Handle empty tuple ()
        if self.match_token(Token::RightParen) {
            return Ok(Expr::Literal(Literal::Tuple(vec![], start_span)));
        }

        let first_expr = self.expression()?;

        // Check for comma to distinguish tuple from grouping
        if self.match_token(Token::Comma) {
            // It's a tuple
            let mut elements = vec![first_expr];

            // Handle single-element tuple: (expr,)
            if self.match_token(Token::RightParen) {
                return Ok(Expr::Literal(Literal::Tuple(elements, start_span)));
            }

            // Parse remaining elements
            while !self.check(Token::RightParen) && !self.is_at_end() {
                elements.push(self.expression()?);
                if !self.match_token(Token::Comma) {
                    break;
                }
            }

            self.consume(Token::RightParen, "Expected ')' after tuple elements")?;
            Ok(Expr::Literal(Literal::Tuple(elements, start_span)))
        } else {
            // It's a grouping expression
            self.consume(Token::RightParen, "Expected ')' after expression")?;
            Ok(Expr::Grouping {
                expr: Box::new(first_expr),
                span: start_span,
            })
        }
    }

    /// Parse list literal [expr, expr, ...]
    pub(super) fn parse_list(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();
        let mut elements = Vec::new();

        if !self.check(Token::RightBracket) {
            loop {
                elements.push(self.expression()?);
                if !self.match_token(Token::Comma) {
                    break;
                }
                // Allow trailing comma
                if self.check(Token::RightBracket) {
                    break;
                }
            }
        }

        self.consume(Token::RightBracket, "Expected ']' after list elements")?;
        Ok(Expr::Literal(Literal::List(elements, start_span)))
    }

    /// Parse map literal { key: value, ... }
    pub(super) fn parse_map(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();
        let mut pairs = Vec::new();

        if !self.check(Token::RightBrace) {
            loop {
                // Parse key, converting bare identifiers to string literals
                let key = self.parse_map_key()?;
                self.consume(Token::Colon, "Expected ':' after map key")?;
                let value = self.expression()?;
                pairs.push((key, value));

                if !self.match_token(Token::Comma) {
                    break;
                }
                // Allow trailing comma
                if self.check(Token::RightBrace) {
                    break;
                }
            }
        }

        self.consume(Token::RightBrace, "Expected '}' after map entries")?;
        Ok(Expr::Literal(Literal::Map(pairs, start_span)))
    }

    /// Parse a map key, converting bare identifiers to string literals per NN spec
    fn parse_map_key(&mut self) -> ParseResult<Expr> {
        // Handle bare identifier as string literal
        if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            let span = self.advance().span.clone();
            // Convert bare identifier to string literal for map keys
            return Ok(Expr::Literal(Literal::StringTemplate(
                vec![crate::ast::StringPart::Text(name)],
                span,
            )));
        }

        // For other key types (string literals, expressions), parse normally
        self.map_key_expression()
    }

    /// Parse string template with interpolation
    pub(super) fn parse_string_template(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();
        let mut parts = Vec::new();

        while !self.check(Token::StringEnd) && !self.is_at_end() {
            if let Token::StringText(text) = &self.peek().token {
                let text = text.clone();
                self.advance();
                parts.push(crate::ast::StringPart::Text(text));
            } else if self.match_token(Token::InterpStart) {
                // Parse interpolated expression
                let expr = self.expression()?;
                parts.push(crate::ast::StringPart::Expr(expr));
                self.consume(
                    Token::InterpEnd,
                    "Expected '}' after interpolated expression",
                )?;
            } else {
                return Err(ParseError::Generic {
                    message: "Unexpected token in string template".to_string(),
                });
            }
        }

        self.consume(Token::StringEnd, "Expected end of string")?;
        Ok(Expr::Literal(Literal::StringTemplate(parts, start_span)))
    }

    /// Parse regex literal /pattern/
    pub(super) fn parse_regex(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();

        if let Token::RegexContent(pattern) = &self.peek().token {
            let pattern = pattern.clone();
            self.advance();
            self.consume(Token::RegexEnd, "Expected end of regex")?;
            Ok(Expr::Literal(Literal::RegexLiteral(pattern, start_span)))
        } else {
            Err(ParseError::Generic {
                message: "Expected regex pattern".to_string(),
            })
        }
    }

    /// Parse shell command template `command ${var} more`
    pub(super) fn parse_shell_command_template(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();
        let mut parts = Vec::new();

        while !self.check(Token::ShellEnd) && !self.is_at_end() {
            if let Token::StringText(text) = &self.peek().token {
                let text = text.clone();
                self.advance();
                parts.push(crate::ast::StringPart::Text(text));
            } else if self.match_token(Token::InterpStart) {
                // Parse interpolated expression
                let expr = self.expression()?;
                parts.push(crate::ast::StringPart::Expr(expr));
                self.consume(
                    Token::InterpEnd,
                    "Expected '}' after interpolated expression",
                )?;
            } else {
                return Err(ParseError::Generic {
                    message: "Unexpected token in shell command template".to_string(),
                });
            }
        }

        self.consume(Token::ShellEnd, "Expected end of shell command")?;
        Ok(Expr::ShellCommandTemplate {
            parts,
            span: start_span,
        })
    }
}
