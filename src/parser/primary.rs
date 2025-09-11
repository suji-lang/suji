use super::{ParseError, ParseResult, Parser};
use crate::ast::{Expr, Literal};
use crate::token::{Span, Token};

/// Type of content inside braces in match arms
#[derive(Debug, Clone, PartialEq)]
enum BracedContentType {
    MapLiteral,
    Block,
}

impl Parser {
    /// Parse a primary expression
    pub(super) fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::True) {
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Boolean(true, span)));
        }

        if self.match_token(Token::False) {
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Boolean(false, span)));
        }

        if self.match_token(Token::Nil) {
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Nil(span)));
        }

        if let Token::Number(n) = self.peek().token {
            self.advance();
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Number(n, span)));
        }

        if let Token::Identifier(name) = self.peek().token {
            self.advance();
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Identifier(name, span)));
        }

        if self.match_token(Token::LeftParen) {
            return self.parse_tuple_or_grouping();
        }

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

        if self.match_token(Token::Pipe) {
            return self.parse_function_literal();
        }

        // Handle || as two pipes for empty function parameters
        if self.match_token(Token::Or) {
            return self.parse_empty_function_literal();
        }

        if self.match_token(Token::ShellStart) {
            return self.parse_shell_command_template();
        }

        if self.match_token(Token::Match) {
            return self.parse_match_expression();
        }

        let current = self.peek();
        Err(ParseError::UnexpectedToken {
            token: current.token,
            span: current.span,
        })
    }

    /// Parse function body - either { statements } or single expression
    pub(super) fn parse_function_body(
        &mut self,
        start_span: Span,
    ) -> ParseResult<crate::ast::Stmt> {
        if self.match_token(Token::LeftBrace) {
            // Traditional block syntax: { statements }
            let statements = self.parse_block()?;
            Ok(crate::ast::Stmt::Block {
                statements,
                span: start_span,
            })
        } else {
            // Single expression syntax: expression
            let expr = self.expression()?;
            Ok(crate::ast::Stmt::Expr(expr))
        }
    }

    /// Parse empty function literal || { body } or || expression
    pub(super) fn parse_empty_function_literal(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();

        // No parameters for empty function literal
        let params = Vec::new();

        // Parse function body (either block or single expression)
        let body = self.parse_function_body(start_span.clone())?;

        Ok(Expr::FunctionLiteral {
            params,
            body: Box::new(body),
            span: start_span,
        })
    }

    /// Parse function literal |param1, param2| { body } or |param1, param2| expression
    pub(super) fn parse_function_literal(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();
        let mut params = Vec::new();

        // Parse parameters until we hit the closing |
        if !self.check(Token::Pipe) {
            loop {
                if let Token::Identifier(param_name) = &self.peek().token {
                    let param_name = param_name.clone();
                    self.advance();

                    // Check for default value
                    let default = if self.match_token(Token::Assign) {
                        Some(self.expression()?)
                    } else {
                        None
                    };

                    params.push(crate::ast::Param {
                        name: param_name,
                        default,
                        span: self.previous().span.clone(),
                    });

                    if !self.match_token(Token::Comma) {
                        break;
                    }
                    // Allow trailing comma
                    if self.check(Token::Pipe) {
                        break;
                    }
                } else {
                    return Err(ParseError::Generic {
                        message: "Expected parameter name".to_string(),
                    });
                }
            }
        }

        self.consume(Token::Pipe, "Expected '|' after function parameters")?;

        // Parse function body (either block or single expression)
        let body = self.parse_function_body(start_span.clone())?;

        Ok(Expr::FunctionLiteral {
            params,
            body: Box::new(body),
            span: start_span,
        })
    }

    /// Parse match expression: match expr? { pattern: expr, ... } or match { condition: expr, ... }
    pub(super) fn parse_match_expression(&mut self) -> ParseResult<Expr> {
        let span = self.previous().span.clone();

        // Check if next token is LeftBrace for conditional match
        let scrutinee = if self.check(Token::LeftBrace) {
            // Conditional match: match { condition: expr, ... }
            None
        } else {
            // Traditional match: match expr { pattern: expr, ... }
            Some(Box::new(self.expression()?))
        };

        self.consume(Token::LeftBrace, "Expected '{' after match expression")?;

        let mut arms = Vec::new();
        while !self.check(Token::RightBrace) && !self.is_at_end() {
            if scrutinee.is_none() {
                // Conditional match: parse boolean expression as pattern (without postfix to avoid colon conflicts)
                let condition_expr = if self.check(Token::Underscore) {
                    // Special case: wildcard in conditional match - treat as "true"
                    self.advance(); // consume the underscore
                    Expr::Literal(crate::ast::Literal::Boolean(
                        true,
                        self.previous().span.clone(),
                    ))
                } else {
                    self.map_key_expression()?
                };
                self.consume(Token::Colon, "Expected ':' after condition")?;

                // Parse body (either block or single expression)
                let body = self.parse_match_arm_body(span.clone())?;

                // For conditional match, we treat the condition as a "pattern"
                arms.push(crate::ast::MatchArm {
                    pattern: crate::ast::Pattern::Expression(condition_expr),
                    body,
                    span: span.clone(),
                });
            } else {
                // Traditional match: parse pattern
                let pattern = self.parse_pattern()?;
                self.consume(Token::Colon, "Expected ':' after match pattern")?;

                // Parse body (either block or single expression)
                let body = self.parse_match_arm_body(span.clone())?;

                arms.push(crate::ast::MatchArm {
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
    pub(super) fn parse_match_arm_body(&mut self, span: Span) -> ParseResult<crate::ast::Stmt> {
        match self.peek().token {
            Token::LeftBrace => self.parse_braced_arm_body(span),
            _ => self.parse_unbraced_arm_body(),
        }
    }

    /// Parse braced arm body - either { statements } or { map_literal }
    fn parse_braced_arm_body(&mut self, span: Span) -> ParseResult<crate::ast::Stmt> {
        self.advance(); // consume LeftBrace

        match self.detect_braced_content_type() {
            BracedContentType::MapLiteral => {
                let expr = self.parse_map()?;
                Ok(crate::ast::Stmt::Expr(expr))
            }
            BracedContentType::Block => {
                let statements = self.parse_block()?;
                Ok(crate::ast::Stmt::Block { statements, span })
            }
        }
    }

    /// Parse unbraced arm body - single expression or statement
    fn parse_unbraced_arm_body(&mut self) -> ParseResult<crate::ast::Stmt> {
        match self.peek().token {
            Token::Return => self.parse_match_arm_return(),
            Token::Break => self.parse_match_arm_break(),
            Token::Continue => self.parse_match_arm_continue(),
            _ => {
                // Fall back to expression
                let expr = self.expression()?;
                Ok(crate::ast::Stmt::Expr(expr))
            }
        }
    }

    /// Parse return statement in match arm
    fn parse_match_arm_return(&mut self) -> ParseResult<crate::ast::Stmt> {
        let return_span = self.advance().span.clone(); // consume Return

        let value = if self.has_expression_after() {
            Some(self.expression()?)
        } else {
            None
        };

        Ok(crate::ast::Stmt::Return {
            value,
            span: return_span,
        })
    }

    /// Parse break statement in match arm
    fn parse_match_arm_break(&mut self) -> ParseResult<crate::ast::Stmt> {
        let break_span = self.advance().span.clone(); // consume Break

        let label = if self.has_expression_after() {
            self.parse_optional_label()
        } else {
            None
        };

        Ok(crate::ast::Stmt::Break {
            label,
            span: break_span,
        })
    }

    /// Parse continue statement in match arm
    fn parse_match_arm_continue(&mut self) -> ParseResult<crate::ast::Stmt> {
        let continue_span = self.advance().span.clone(); // consume Continue

        let label = if self.has_expression_after() {
            self.parse_optional_label()
        } else {
            None
        };

        Ok(crate::ast::Stmt::Continue {
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
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_parse_simple_expression() {
        let result = parse_expression("42");
        assert!(result.is_ok());
    }
}
