use super::{ParseError, ParseResult, Parser};
use crate::ast::Expr;
use crate::token::Token;

impl Parser {
    /// Parse postfix expressions (calls, indexing, method calls, etc.)
    pub(super) fn postfix(&mut self) -> ParseResult<Expr> {
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
            } else if self.match_token(Token::Colon) {
                // Map access by name
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
        let mut args = Vec::new();

        if !self.check(Token::RightParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_token(Token::Comma) {
                    break;
                }
                // Allow trailing comma
                if self.check(Token::RightParen) {
                    break;
                }
            }
        }

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
        if let Token::Identifier(method_name) = &self.peek().token {
            let method_name = method_name.clone();
            self.advance();

            self.consume(Token::LeftParen, "Expected '(' after method name")?;

            let mut args = Vec::new();
            if !self.check(Token::RightParen) {
                loop {
                    args.push(self.expression()?);
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                    // Allow trailing comma
                    if self.check(Token::RightParen) {
                        break;
                    }
                }
            }

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
            Err(ParseError::Generic {
                message: "Expected method name after '::'".to_string(),
            })
        }
    }

    /// Finish parsing map access by name
    pub(super) fn finish_map_access(&mut self, target: Expr) -> ParseResult<Expr> {
        if let Token::Identifier(key) = &self.peek().token {
            let key = key.clone();
            let span = self.advance().span.clone();
            Ok(Expr::MapAccessByName {
                target: Box::new(target),
                key,
                span,
            })
        } else {
            Err(ParseError::Generic {
                message: "Expected identifier after ':'".to_string(),
            })
        }
    }
}
