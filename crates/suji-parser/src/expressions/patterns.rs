use crate::{ParseError, ParseResult, Parser};
use suji_lexer::Token;

impl Parser {
    /// Parse pattern for match statements
    pub(super) fn parse_pattern(&mut self) -> ParseResult<suji_ast::Pattern> {
        if self.match_token(Token::Underscore) {
            let span = self.previous().span.clone();
            return Ok(suji_ast::Pattern::Wildcard { span });
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
            return Ok(suji_ast::Pattern::Tuple { patterns, span });
        }

        if self.match_token(Token::RegexStart) {
            // Regex pattern
            let span = self.previous().span.clone();
            if let Token::RegexContent(pattern) = &self.peek().token {
                let pattern = pattern.clone();
                self.advance();
                self.consume(Token::RegexEnd, "Expected end of regex pattern")?;
                return Ok(suji_ast::Pattern::Regex { pattern, span });
            } else {
                let current = self.peek();
                return Err(ParseError::ExpectedToken {
                    expected: Token::RegexContent(String::new()),
                    found: current.token,
                    span: current.span,
                });
            }
        }

        if self.match_token(Token::Minus) {
            let minus_span = self.previous().span.clone();
            if let Token::Number(n) = &self.peek().token {
                let n = n.clone();
                let number_span = self.advance().span.clone();
                // Prepend minus sign to create negative number
                let negative_n = format!("-{}", n);
                // Combine spans from minus to number
                let combined_span = suji_lexer::Span::new(
                    minus_span.start,
                    number_span.end,
                    minus_span.line,
                    minus_span.column,
                );
                return Ok(suji_ast::Pattern::Literal {
                    value: suji_ast::ValueLike::Number(negative_n),
                    span: combined_span,
                });
            } else {
                // Minus not followed by a number - error
                let current = self.peek();
                return Err(ParseError::ExpectedToken {
                    expected: Token::Number(String::new()),
                    found: current.token,
                    span: current.span,
                });
            }
        }

        if let Token::Number(n) = &self.peek().token {
            let n = n.clone();
            let span = self.advance().span.clone();
            return Ok(suji_ast::Pattern::Literal {
                value: suji_ast::ValueLike::Number(n),
                span,
            });
        }

        if self.match_token(Token::True) {
            let span = self.previous().span.clone();
            return Ok(suji_ast::Pattern::Literal {
                value: suji_ast::ValueLike::Boolean(true),
                span,
            });
        }

        if self.match_token(Token::False) {
            let span = self.previous().span.clone();
            return Ok(suji_ast::Pattern::Literal {
                value: suji_ast::ValueLike::Boolean(false),
                span,
            });
        }

        if self.match_token(Token::Nil) {
            let span = self.previous().span.clone();
            return Ok(suji_ast::Pattern::Literal {
                value: suji_ast::ValueLike::Nil,
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
                    parts.push(suji_ast::StringPart::Text(text));
                } else if self.match_token(Token::InterpStart) {
                    let expr = self.expression()?;
                    parts.push(suji_ast::StringPart::Expr(expr));
                    self.consume(Token::InterpEnd, "Expected '}' after string interpolation")?;
                } else {
                    let current = self.peek();
                    return Err(ParseError::UnexpectedToken {
                        token: current.token,
                        span: current.span,
                    });
                }
            }

            self.consume(Token::StringEnd, "Expected end of string literal")?;

            // For pattern matching, we only support simple string literals (no interpolation)
            if parts.len() == 1
                && let suji_ast::StringPart::Text(text) = &parts[0]
            {
                return Ok(suji_ast::Pattern::Literal {
                    value: suji_ast::ValueLike::String(text.clone()),
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
            return Ok(suji_ast::Pattern::Literal {
                value: suji_ast::ValueLike::String(name),
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
}
