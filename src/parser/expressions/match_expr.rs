use super::{ParseError, ParseResult, Parser};
use crate::ast::{Expr, MatchArm, Pattern, Stmt};
use crate::token::{Span, Token};

/// Type of content inside braces in match arms
#[derive(Debug, Clone, PartialEq)]
enum BracedContentType {
    MapLiteral,
    Block,
}

impl Parser {
    /// Parse match expression: match expr? { pattern => expr, ... } or match { condition => expr, ... }
    pub fn parse_match_expression(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::Match) {
            return self.parse_match_expression_impl();
        }

        Err(ParseError::Generic {
            message: "Not a match expression".to_string(),
        })
    }

    /// Parse match expression: match expr? { pattern => expr, ... } or match { condition => expr, ... }
    fn parse_match_expression_impl(&mut self) -> ParseResult<Expr> {
        let span = self.previous().span.clone();

        // Check if next token is LeftBrace for conditional match
        let scrutinee = if self.check(Token::LeftBrace) {
            // Conditional match: match { condition => expr, ... }
            None
        } else {
            // Traditional match: match expr { pattern => expr, ... }
            Some(Box::new(self.expression()?))
        };

        self.consume(
            Token::LeftBrace,
            "Expected '{' after match expression or match { for conditional match",
        )?;

        let mut arms = Vec::new();
        while !self.check(Token::RightBrace) && !self.is_at_end() {
            if scrutinee.is_none() {
                // Conditional match: parse boolean expression as pattern
                let condition_expr = if self.check(Token::Underscore) {
                    // Special case: wildcard in conditional match - treat as "true"
                    self.advance(); // consume the underscore
                    Expr::Literal(crate::ast::Literal::Boolean(
                        true,
                        self.previous().span.clone(),
                    ))
                } else {
                    self.expression()?
                };
                self.consume(
                    Token::FatArrow,
                    "Expected '=>' after condition in conditional match",
                )?;

                // Parse body (either block or single expression)
                let body = self.parse_match_arm_body(span.clone())?;

                // For conditional match, we treat the condition as a "pattern"
                arms.push(MatchArm {
                    pattern: Pattern::Expression(condition_expr),
                    body,
                    span: span.clone(),
                });
            } else {
                // Traditional match: parse pattern(s) with alternation using '|'
                let first_pattern = self.parse_pattern()?;
                let mut patterns = vec![first_pattern];
                while self.match_token(Token::Pipe) {
                    // Expect another pattern after '|'
                    // If parse_pattern fails, propagate with a clearer message
                    let next_pattern = match self.parse_pattern() {
                        Ok(p) => p,
                        Err(_) => {
                            return Err(ParseError::Generic {
                                message: "Expected pattern after '|' in match arm".to_string(),
                            });
                        }
                    };
                    patterns.push(next_pattern);
                }

                self.consume(Token::FatArrow, "Expected '=>' after match pattern")?;

                // Parse body (either block or single expression)
                let body = self.parse_match_arm_body(span.clone())?;

                // Desugar alternation into multiple arms with identical bodies
                for pattern in patterns {
                    arms.push(MatchArm {
                        pattern,
                        body: body.clone(),
                        span: span.clone(),
                    });
                }
            }

            // Required comma
            self.consume(Token::Comma, "Expected ',' after match arm")?;
            self.skip_newlines();
        }

        self.consume(Token::RightBrace, "Expected '}' after match arms")?;

        Ok(Expr::Match {
            scrutinee,
            arms,
            span,
        })
    }

    /// Parse match arm body - either { statements } or single expression/statement
    pub(super) fn parse_match_arm_body(&mut self, span: Span) -> ParseResult<Stmt> {
        match self.peek().token {
            Token::LeftBrace => self.parse_braced_arm_body(span),
            _ => self.parse_unbraced_arm_body(),
        }
    }

    /// Parse braced arm body - either { statements } or { map_literal }
    fn parse_braced_arm_body(&mut self, span: Span) -> ParseResult<Stmt> {
        self.advance(); // consume LeftBrace

        match self.detect_braced_content_type() {
            BracedContentType::MapLiteral => {
                let expr = self.parse_map()?;
                Ok(Stmt::Expr(expr))
            }
            BracedContentType::Block => {
                let statements = self.parse_block()?;
                Ok(Stmt::Block { statements, span })
            }
        }
    }

    /// Parse unbraced arm body - single expression or statement
    fn parse_unbraced_arm_body(&mut self) -> ParseResult<Stmt> {
        match self.peek().token {
            Token::Return => self.parse_match_arm_return(),
            Token::Break => self.parse_match_arm_break(),
            Token::Continue => self.parse_match_arm_continue(),
            _ => {
                // Fall back to expression
                let expr = self.expression()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    /// Parse return statement in match arm
    /// In unbraced match arms, only single-value returns are allowed to avoid ambiguity
    /// with the match arm separator comma
    fn parse_match_arm_return(&mut self) -> ParseResult<Stmt> {
        let return_span = self.advance().span.clone(); // consume Return

        if !self.has_expression_after() {
            return Ok(Stmt::Return {
                values: Vec::new(),
                span: return_span,
            });
        }

        // Parse single return value (no comma-separated multi-value returns in unbraced arms)
        let value = self.expression()?;

        Ok(Stmt::Return {
            values: vec![value],
            span: return_span,
        })
    }

    /// Parse break statement in match arm
    fn parse_match_arm_break(&mut self) -> ParseResult<Stmt> {
        let break_span = self.advance().span.clone(); // consume Break

        let label = if self.has_expression_after() {
            self.parse_optional_label()
        } else {
            None
        };

        Ok(Stmt::Break {
            label,
            span: break_span,
        })
    }

    /// Parse continue statement in match arm
    fn parse_match_arm_continue(&mut self) -> ParseResult<Stmt> {
        let continue_span = self.advance().span.clone(); // consume Continue

        let label = if self.has_expression_after() {
            self.parse_optional_label()
        } else {
            None
        };

        Ok(Stmt::Continue {
            label,
            span: continue_span,
        })
    }

    /// Check if there's an expression after the current token (not end of arm)
    fn has_expression_after(&self) -> bool {
        !self.check(Token::RightBrace)
            && !self.check(Token::Comma)
            && !self.check(Token::FatArrow)
            && !self.is_at_end()
    }

    /// Parse optional label for break/continue statements
    fn parse_optional_label(&mut self) -> Option<String> {
        if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    /// Detect whether braced content is a map literal or block
    fn detect_braced_content_type(&self) -> BracedContentType {
        match &self.peek().token {
            Token::StringStart => {
                // Look ahead to see if there's a colon after the string
                if self.has_colon_after_string() {
                    BracedContentType::MapLiteral
                } else {
                    BracedContentType::Block
                }
            }
            Token::Number(_) => {
                // Check if there's a colon after the number
                if self.has_colon_after_number() {
                    BracedContentType::MapLiteral
                } else {
                    BracedContentType::Block
                }
            }
            _ => BracedContentType::Block,
        }
    }

    /// Check if there's a colon after a string literal
    fn has_colon_after_string(&self) -> bool {
        let start_pos = self.current;
        let mut pos = start_pos;

        // Skip the string content
        while pos < self.tokens.len() && !matches!(self.tokens[pos].token, Token::StringEnd) {
            pos += 1;
        }

        // Check if there's a colon after the string
        pos < self.tokens.len()
            && matches!(self.tokens[pos].token, Token::StringEnd)
            && pos + 1 < self.tokens.len()
            && matches!(self.tokens[pos + 1].token, Token::Colon)
    }

    /// Check if there's a colon after a number literal
    fn has_colon_after_number(&self) -> bool {
        self.current + 1 < self.tokens.len()
            && matches!(self.tokens[self.current + 1].token, Token::Colon)
    }
}
