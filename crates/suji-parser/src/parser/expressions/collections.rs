use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::{Expr, Literal, StringPart};
use suji_lexer::token::Token;

impl Parser {
    /// Parse collection literals: tuples, lists, maps, strings, regex
    pub fn parse_collections(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::LeftBracket) {
            return self.parse_list();
        }

        if self.match_token(Token::LeftBrace) {
            return self.parse_map();
        }

        if self.match_token(Token::StringStart) {
            return self.parse_string_template();
        }

        if self.match_token(Token::RegexStart) {
            return self.parse_regex();
        }

        let current = self.peek();
        Err(ParseError::UnexpectedToken {
            token: current.token,
            span: current.span,
        })
    }

    /// Parse tuple or grouping expression (disambiguation by comma)
    pub(crate) fn parse_tuple_or_grouping(&mut self) -> ParseResult<Expr> {
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
    fn parse_list(&mut self) -> ParseResult<Expr> {
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

    /// Parse a map key, converting bare identifiers to string literals per SUJI spec
    fn parse_map_key(&mut self) -> ParseResult<Expr> {
        // Handle bare identifier as string literal
        if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            let span = self.advance().span.clone();
            // Convert bare identifier to string literal for map keys
            return Ok(Expr::Literal(Literal::StringTemplate(
                vec![StringPart::Text(name)],
                span,
            )));
        }

        // For other key types (string literals, expressions), parse normally
        self.map_key_expression()
    }

    /// Parse string template with interpolation
    fn parse_string_template(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();
        let mut parts = Vec::new();

        while !self.check(Token::StringEnd) && !self.is_at_end() {
            if let Token::StringText(text) = &self.peek().token {
                let text = text.clone();
                self.advance();
                parts.push(StringPart::Text(text));
            } else if self.match_token(Token::InterpStart) {
                // Parse interpolated expression
                let expr = self.expression()?;
                parts.push(StringPart::Expr(expr));
                self.consume(
                    Token::InterpEnd,
                    "Expected '}' after interpolated expression",
                )?;
            } else {
                let current = self.peek();
                return Err(ParseError::UnexpectedToken {
                    token: current.token,
                    span: current.span,
                });
            }
        }

        self.consume(Token::StringEnd, "Expected end of string")?;
        Ok(Expr::Literal(Literal::StringTemplate(parts, start_span)))
    }

    /// Parse regex literal /pattern/
    fn parse_regex(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();

        if let Token::RegexContent(pattern) = &self.peek().token {
            let pattern = pattern.clone();
            self.advance();
            self.consume(Token::RegexEnd, "Expected end of regex")?;
            Ok(Expr::Literal(Literal::RegexLiteral(pattern, start_span)))
        } else {
            let current = self.peek();
            Err(ParseError::ExpectedToken {
                expected: Token::RegexContent(String::new()),
                found: current.token,
                span: current.span,
            })
        }
    }
}
