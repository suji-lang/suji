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
    /// Parse match expression: match expr? { pattern: expr, ... } or match { condition: expr, ... }
    pub fn parse_match_expression(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::Match) {
            return self.parse_match_expression_impl();
        }

        Err(ParseError::Generic {
            message: "Not a match expression".to_string(),
        })
    }

    /// Parse match expression: match expr? { pattern: expr, ... } or match { condition: expr, ... }
    fn parse_match_expression_impl(&mut self) -> ParseResult<Expr> {
        let span = self.previous().span.clone();

        // Check if next token is LeftBrace for conditional match
        let scrutinee = if self.check(Token::LeftBrace) {
            // Conditional match: match { condition: expr, ... }
            None
        } else {
            // Traditional match: match expr { pattern: expr, ... }
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
                    self.conditional_match_expression()?
                };
                self.consume(
                    Token::Colon,
                    "Expected ':' after condition in conditional match",
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
                // Traditional match: parse pattern
                let pattern = self.parse_pattern()?;
                self.consume(Token::Colon, "Expected ':' after match pattern")?;

                // Parse body (either block or single expression)
                let body = self.parse_match_arm_body(span.clone())?;

                arms.push(MatchArm {
                    pattern,
                    body,
                    span: span.clone(),
                });
            }

            // Optional comma
            self.match_token(Token::Comma);
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
    fn parse_match_arm_return(&mut self) -> ParseResult<Stmt> {
        let return_span = self.advance().span.clone(); // consume Return

        if !self.has_expression_after() {
            return Ok(Stmt::Return {
                values: Vec::new(),
                span: return_span,
            });
        }

        let mut values = Vec::new();
        loop {
            values.push(self.expression()?);

            if !self.match_token(Token::Comma) {
                break;
            }

            if !self.has_expression_after() {
                return Err(ParseError::Generic {
                    message: "Trailing comma not allowed in return".to_string(),
                });
            }
        }

        Ok(Stmt::Return {
            values,
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
        !self.check(Token::RightBrace) && !self.check(Token::Comma) && !self.is_at_end()
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

    /// Parse expression for conditional match conditions
    /// Supports method calls but stops at colons to avoid conflicts
    fn conditional_match_expression(&mut self) -> ParseResult<Expr> {
        self.conditional_match_assignment()
    }

    /// Parse assignment for conditional match (without colon conflicts)
    fn conditional_match_assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.conditional_match_logical_or()?;

        if self.match_token(Token::Assign) {
            let span = self.previous().span.clone();
            let value = self.conditional_match_assignment()?; // Right-associative
            return Ok(Expr::Assign {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    /// Parse logical OR for conditional match
    fn conditional_match_logical_or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_logical_and()?;

        while self.match_token(Token::Or) {
            let op = crate::ast::BinaryOp::Or;
            let span = self.previous().span.clone();
            let right = self.conditional_match_logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse logical AND for conditional match
    fn conditional_match_logical_and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_regex_match()?;

        while self.match_token(Token::And) {
            let op = crate::ast::BinaryOp::And;
            let span = self.previous().span.clone();
            let right = self.conditional_match_regex_match()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse regex match for conditional match
    fn conditional_match_regex_match(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_equality()?;

        while self.match_token(Token::RegexMatch) || self.match_token(Token::RegexNotMatch) {
            let op = match &self.previous().token {
                Token::RegexMatch => crate::ast::BinaryOp::RegexMatch,
                Token::RegexNotMatch => crate::ast::BinaryOp::RegexNotMatch,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.conditional_match_equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse equality for conditional match
    fn conditional_match_equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_comparison()?;

        while self.match_token(Token::NotEqual) || self.match_token(Token::Equal) {
            let op = match &self.previous().token {
                Token::NotEqual => crate::ast::BinaryOp::NotEqual,
                Token::Equal => crate::ast::BinaryOp::Equal,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.conditional_match_comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse comparison for conditional match
    fn conditional_match_comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_term()?;

        while self.match_token(Token::Greater)
            || self.match_token(Token::GreaterEqual)
            || self.match_token(Token::Less)
            || self.match_token(Token::LessEqual)
        {
            let op = match &self.previous().token {
                Token::Greater => crate::ast::BinaryOp::Greater,
                Token::GreaterEqual => crate::ast::BinaryOp::GreaterEqual,
                Token::Less => crate::ast::BinaryOp::Less,
                Token::LessEqual => crate::ast::BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.conditional_match_term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse term for conditional match
    fn conditional_match_term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_range()?;

        while self.match_token(Token::Minus) || self.match_token(Token::Plus) {
            let op = match &self.previous().token {
                Token::Minus => crate::ast::BinaryOp::Subtract,
                Token::Plus => crate::ast::BinaryOp::Add,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.conditional_match_range()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse range for conditional match
    fn conditional_match_range(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_factor()?;

        while self.match_token(Token::Range) {
            let op = crate::ast::BinaryOp::Range;
            let span = self.previous().span.clone();
            let right = self.conditional_match_factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse factor for conditional match
    fn conditional_match_factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_power()?;

        while self.match_token(Token::Divide)
            || self.match_token(Token::Multiply)
            || self.match_token(Token::Modulo)
        {
            let op = match &self.previous().token {
                Token::Divide => crate::ast::BinaryOp::Divide,
                Token::Multiply => crate::ast::BinaryOp::Multiply,
                Token::Modulo => crate::ast::BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.conditional_match_power()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse power for conditional match
    fn conditional_match_power(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional_match_unary()?;

        if self.match_token(Token::Power) {
            let op = crate::ast::BinaryOp::Power;
            let span = self.previous().span.clone();
            let right = self.conditional_match_power()?; // Right-associative recursion
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse unary for conditional match
    fn conditional_match_unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::Not) || self.match_token(Token::Minus) {
            let op = match &self.previous().token {
                Token::Not => crate::ast::UnaryOp::Not,
                Token::Minus => crate::ast::UnaryOp::Negate,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let expr = self.conditional_match_unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span,
            });
        }

        self.conditional_match_postfix()
    }

    /// Parse postfix for conditional match (supports method calls but stops at colons)
    fn conditional_match_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(Token::LeftParen) {
                // Function call
                expr = self.finish_call(expr)?;
            } else if self.match_token(Token::LeftBracket) {
                // Index or slice
                expr = self.finish_index_or_slice(expr)?;
            } else if self.match_token(Token::DoubleColon) {
                // Method call
                expr = self.finish_method_call(expr)?;
            } else if self.match_token(Token::Increment) {
                // Postfix increment
                let span = self.previous().span.clone();
                expr = Expr::PostfixIncrement {
                    target: Box::new(expr),
                    span,
                };
            } else if self.match_token(Token::Decrement) {
                // Postfix decrement
                let span = self.previous().span.clone();
                expr = Expr::PostfixDecrement {
                    target: Box::new(expr),
                    span,
                };
            } else {
                // Stop at colons to avoid conflicts with match arm separator
                break;
            }
        }

        Ok(expr)
    }
}
