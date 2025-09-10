use super::{ParseError, ParseResult, Parser};
use crate::ast::{Expr, Literal};
use crate::token::{Span, Token};

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

    /// Parse match expression: match expr { pattern: expr, ... }
    pub(super) fn parse_match_expression(&mut self) -> ParseResult<Expr> {
        let span = self.previous().span.clone();
        let scrutinee = self.expression()?;

        self.consume(Token::LeftBrace, "Expected '{' after match expression")?;

        let mut arms = Vec::new();
        while !self.check(Token::RightBrace) && !self.is_at_end() {
            // Parse pattern
            let pattern = self.parse_pattern()?;
            self.consume(Token::Colon, "Expected ':' after match pattern")?;

            // Parse body (either block or single expression)
            let body = self.parse_match_arm_body(span.clone())?;

            arms.push(crate::ast::MatchArm {
                pattern,
                body,
                span: span.clone(),
            });

            // Optional comma
            self.match_token(Token::Comma);
            self.skip_newlines();
        }

        self.consume(Token::RightBrace, "Expected '}' after match arms")?;

        Ok(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span,
        })
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
