use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::{Expr, Param, Stmt};
use suji_lexer::token::{Span, Token};

impl Parser {
    /// Parse function literals: |params| { body } or |params| expression
    pub fn parse_functions(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::Pipe) {
            return self.parse_function_literal();
        }

        // Handle || as two pipes for empty function parameters
        if self.match_token(Token::Or) {
            return self.parse_empty_function_literal();
        }

        Err(ParseError::Generic {
            message: "Not a function literal".to_string(),
        })
    }

    /// Parse function body - either { statements } or single expression
    pub(super) fn parse_function_body(&mut self, start_span: Span) -> ParseResult<Stmt> {
        if self.match_token(Token::LeftBrace) {
            // Traditional block syntax: { statements }
            let statements = self.parse_block()?;
            Ok(Stmt::Block {
                statements,
                span: start_span,
            })
        } else {
            // Single expression syntax: expression
            let expr = self.expression()?;
            Ok(Stmt::Expr(expr))
        }
    }

    /// Parse empty function literal || { body } or || expression
    fn parse_empty_function_literal(&mut self) -> ParseResult<Expr> {
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
    fn parse_function_literal(&mut self) -> ParseResult<Expr> {
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
                        // Use an expression parser that doesn't consume the '|' param terminator as a pipe op
                        Some(self.parse_expression_without_pipe()?)
                    } else {
                        None
                    };

                    params.push(Param {
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
        // When parsing function literals, a top-level '|' should terminate param list,
        // not act as a binary operator inside defaults/body parsing.
        let body = self.parse_function_body(start_span.clone())?;

        Ok(Expr::FunctionLiteral {
            params,
            body: Box::new(body),
            span: start_span,
        })
    }
}
