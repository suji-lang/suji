use super::ExpressionContext;
use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::Expr;
use suji_lexer::token::Token;

impl Parser {
    /// Parse a comma-separated argument list until the specified end token
    /// Supports trailing commas and empty argument lists
    fn parse_argument_list(&mut self, end_token: Token) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();

        // Handle empty argument list
        if !self.check(end_token.clone()) {
            loop {
                // Parse each argument expression
                args.push(self.expression()?);

                // Check for comma separator
                if !self.match_token(Token::Comma) {
                    break;
                }

                // Allow trailing comma - break if we see the end token
                if self.check(end_token.clone()) {
                    break;
                }
            }
        }

        Ok(args)
    }

    /// Parse postfix expressions (calls, indexing, method calls, etc.)
    pub(super) fn postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            // If postfix is disabled in the current context, stop immediately
            if self.expression_context == ExpressionContext::NoPostfix {
                break;
            }
            if self.match_token(Token::LeftParen) {
                // Function call
                expr = self.finish_call(expr)?;
            } else if self.match_token(Token::LeftBracket) {
                // Index or slice
                expr = self.finish_index_or_slice(expr)?;
            } else if self.match_token(Token::DoubleColon) {
                // Method call
                expr = self.finish_method_call(expr)?;
            } else if self.check(Token::Colon)
                && self.expression_context != ExpressionContext::NoColonAccess
            {
                // Map access by name (skip if in NoColonAccess context for slice parsing)
                self.advance(); // consume the colon
                expr = self.finish_map_access(expr)?;
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
                break;
            }
        }

        Ok(expr)
    }

    /// Finish parsing a function call
    pub(super) fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        // Use shared argument parser
        let args = self.parse_argument_list(Token::RightParen)?;

        let span = self
            .consume(Token::RightParen, "Expected ')' after function arguments")?
            .span
            .clone();
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
            span,
        })
    }

    /// Finish parsing indexing or slicing
    pub(super) fn finish_index_or_slice(&mut self, target: Expr) -> ParseResult<Expr> {
        if self.match_token(Token::Colon) {
            // It's a slice starting with colon: target[:end] or target[:]
            let end_expr = if self.check(Token::RightBracket) {
                None
            } else {
                Some(Box::new(self.index_expression()?))
            };
            let span = self
                .consume(Token::RightBracket, "Expected ']' after slice")?
                .span
                .clone();
            Ok(Expr::Slice {
                target: Box::new(target),
                start: None,
                end: end_expr,
                span,
            })
        } else {
            // Parse the first expression
            let first_expr = self.index_expression()?;

            if self.match_token(Token::Colon) {
                // It's a slice: target[start:end] or target[start:]
                let end_expr = if self.check(Token::RightBracket) {
                    None
                } else {
                    Some(Box::new(self.index_expression()?))
                };
                let span = self
                    .consume(Token::RightBracket, "Expected ']' after slice")?
                    .span
                    .clone();
                Ok(Expr::Slice {
                    target: Box::new(target),
                    start: Some(Box::new(first_expr)),
                    end: end_expr,
                    span,
                })
            } else {
                // It's an index: target[index]
                let span = self
                    .consume(Token::RightBracket, "Expected ']' after index")?
                    .span
                    .clone();
                Ok(Expr::Index {
                    target: Box::new(target),
                    index: Box::new(first_expr),
                    span,
                })
            }
        }
    }

    /// Finish parsing a method call
    pub(super) fn finish_method_call(&mut self, receiver: Expr) -> ParseResult<Expr> {
        if let Token::Identifier(_) = &self.peek().token {
            let (method_name, _span) = self.consume_identifier()?;

            self.consume(Token::LeftParen, "Expected '(' after method name")?;

            // Use shared argument parser
            let args = self.parse_argument_list(Token::RightParen)?;

            let span = self
                .consume(Token::RightParen, "Expected ')' after method arguments")?
                .span
                .clone();
            Ok(Expr::MethodCall {
                target: Box::new(receiver),
                method: method_name,
                args,
                span,
            })
        } else {
            let current = self.peek();
            Err(ParseError::ExpectedToken {
                expected: Token::Identifier(String::new()),
                found: current.token,
                span: current.span,
            })
        }
    }

    /// Finish parsing map access by name
    pub(super) fn finish_map_access(&mut self, target: Expr) -> ParseResult<Expr> {
        if let Token::Identifier(_) = &self.peek().token {
            let (key, span) = self.consume_identifier()?;
            Ok(Expr::MapAccessByName {
                target: Box::new(target),
                key,
                span,
            })
        } else {
            let current = self.peek();
            Err(ParseError::ExpectedToken {
                expected: Token::Identifier(String::new()),
                found: current.token,
                span: current.span,
            })
        }
    }
}
