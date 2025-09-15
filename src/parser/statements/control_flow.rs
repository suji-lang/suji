use super::{ParseError, ParseResult, Parser};
use crate::ast::Stmt;
use crate::token::{Span, Token};

impl Parser {
    /// Parse return statement: return expr?
    pub(super) fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();
        let value = if self.check(Token::Newline) || self.is_at_end() {
            None
        } else {
            Some(self.expression()?)
        };
        Ok(Stmt::Return { value, span })
    }

    /// Parse break statement: break label?
    pub(super) fn parse_break_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();
        let label = if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        };
        Ok(Stmt::Break { label, span })
    }

    /// Parse continue statement: continue label?
    pub(super) fn parse_continue_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();
        let label = if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        };
        Ok(Stmt::Continue { label, span })
    }

    /// Parse loop statement: loop (as label)? { ... } or loop through ...
    pub(super) fn parse_loop_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        // Check for "loop through"
        if self.match_token(Token::Through) {
            return self.parse_loop_through_statement(span);
        }

        // Parse optional label: loop as label { ... }
        let label = if self.match_token(Token::As) {
            if let Token::Identifier(name) = &self.peek().token {
                let name = name.clone();
                self.advance();
                Some(name)
            } else {
                return Err(ParseError::Generic {
                    message: "Expected label after 'as'".to_string(),
                });
            }
        } else {
            None
        };

        // Parse body
        self.consume(Token::LeftBrace, "Expected '{' after loop")?;
        let body_statements = self.parse_block()?;
        let body = Box::new(Stmt::Block {
            statements: body_statements,
            span: span.clone(),
        });

        Ok(Stmt::Loop { label, body, span })
    }

    /// Parse loop through statement: loop through expr (with bindings)? (as label)? { ... }
    pub(super) fn parse_loop_through_statement(&mut self, span: Span) -> ParseResult<Stmt> {
        let iterable = self.expression()?;

        // Parse optional bindings: with var1, var2
        let bindings = if self.match_token(Token::With) {
            if let Token::Identifier(var1) = &self.peek().token {
                let var1 = var1.clone();
                self.advance();

                if self.match_token(Token::Comma) {
                    if let Token::Identifier(var2) = &self.peek().token {
                        let var2 = var2.clone();
                        self.advance();
                        crate::ast::LoopBindings::Two(var1, var2)
                    } else {
                        return Err(ParseError::Generic {
                            message: "Expected second variable after comma".to_string(),
                        });
                    }
                } else {
                    crate::ast::LoopBindings::One(var1)
                }
            } else {
                return Err(ParseError::Generic {
                    message: "Expected variable name after 'with'".to_string(),
                });
            }
        } else {
            crate::ast::LoopBindings::None
        };

        // Parse optional label: as label
        let label = if self.match_token(Token::As) {
            if let Token::Identifier(name) = &self.peek().token {
                let name = name.clone();
                self.advance();
                Some(name)
            } else {
                return Err(ParseError::Generic {
                    message: "Expected label after 'as'".to_string(),
                });
            }
        } else {
            None
        };

        // Parse body
        self.consume(Token::LeftBrace, "Expected '{' after loop through")?;
        let body_statements = self.parse_block()?;
        let body = Box::new(Stmt::Block {
            statements: body_statements,
            span: span.clone(),
        });

        Ok(Stmt::LoopThrough {
            label,
            iterable,
            bindings,
            body,
            span,
        })
    }
}
