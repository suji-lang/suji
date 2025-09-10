use super::{ParseError, ParseResult, Parser};
use crate::ast::Stmt;
use crate::token::{Span, Token};

/// Type of content inside braces in match arms
#[derive(Debug, Clone, PartialEq)]
enum BracedContentType {
    MapLiteral,
    Block,
}

impl Parser {
    /// Parse a statement
    pub(super) fn statement(&mut self) -> ParseResult<Stmt> {
        // Return statement
        if self.match_token(Token::Return) {
            return self.parse_return_statement();
        }

        // Break statement
        if self.match_token(Token::Break) {
            return self.parse_break_statement();
        }

        // Continue statement
        if self.match_token(Token::Continue) {
            return self.parse_continue_statement();
        }

        // Loop statements
        if self.match_token(Token::Loop) {
            return self.parse_loop_statement();
        }

        // Match statement
        if self.match_token(Token::Match) {
            return self.parse_match_statement();
        }

        // Import statement
        if self.match_token(Token::Import) {
            return self.parse_import_statement();
        }

        // Export statement
        if self.match_token(Token::Export) {
            return self.parse_export_statement();
        }

        // Block statement
        if self.match_token(Token::LeftBrace) {
            return self.parse_block_statement();
        }

        // Expression statement (default)
        let expr = self.expression()?;
        Ok(Stmt::Expr(expr))
    }

    /// Parse return statement: return expr?
    fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();
        let value = if self.check(Token::Newline) || self.is_at_end() {
            None
        } else {
            Some(self.expression()?)
        };
        Ok(Stmt::Return { value, span })
    }

    /// Parse break statement: break label?
    fn parse_break_statement(&mut self) -> ParseResult<Stmt> {
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
    fn parse_continue_statement(&mut self) -> ParseResult<Stmt> {
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
    fn parse_loop_statement(&mut self) -> ParseResult<Stmt> {
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
    fn parse_loop_through_statement(&mut self, span: Span) -> ParseResult<Stmt> {
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

        let value = if self.has_expression_after() {
            Some(self.expression()?)
        } else {
            None
        };

        Ok(Stmt::Return {
            value,
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

    /// Parse match statement: match expr { pattern: stmt, ... }
    fn parse_match_statement(&mut self) -> ParseResult<Stmt> {
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

        Ok(Stmt::Match {
            scrutinee,
            arms,
            span,
        })
    }

    /// Parse block statement: { stmt1; stmt2; ... }
    fn parse_block_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();
        let statements = self.parse_block()?;
        Ok(Stmt::Block { statements, span })
    }

    /// Parse import statement: import spec
    fn parse_import_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        if let Token::Identifier(module_name) = &self.peek().token {
            let module_name = module_name.clone();
            self.advance();

            // Check for colon (import module:item)
            if self.match_token(Token::Colon) {
                // import module:item [as alias]
                if let Token::Identifier(item_name) = &self.peek().token {
                    let item_name = item_name.clone();
                    self.advance();

                    // Check for 'as' alias
                    if self.match_token(Token::As) {
                        if let Token::Identifier(alias) = &self.peek().token {
                            let alias = alias.clone();
                            self.advance();

                            Ok(Stmt::Import {
                                spec: crate::ast::ImportSpec::ItemAs {
                                    module: module_name,
                                    name: item_name,
                                    alias,
                                },
                                span,
                            })
                        } else {
                            Err(ParseError::Generic {
                                message: "Expected alias name after 'as'".to_string(),
                            })
                        }
                    } else {
                        // import module:item
                        Ok(Stmt::Import {
                            spec: crate::ast::ImportSpec::Item {
                                module: module_name,
                                name: item_name,
                            },
                            span,
                        })
                    }
                } else {
                    Err(ParseError::Generic {
                        message: "Expected item name after ':'".to_string(),
                    })
                }
            } else {
                // import module
                Ok(Stmt::Import {
                    spec: crate::ast::ImportSpec::Module { name: module_name },
                    span,
                })
            }
        } else {
            Err(ParseError::Generic {
                message: "Expected module name after import".to_string(),
            })
        }
    }

    /// Parse export statement: export { name: expr, ... }
    fn parse_export_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        // Enforce single export per file
        if self.export_count > 0 {
            return Err(ParseError::MultipleExports { span });
        }
        self.export_count += 1;

        self.consume(Token::LeftBrace, "Expected '{' after export")?;
        let mut exports = Vec::new();

        while !self.check(Token::RightBrace) && !self.is_at_end() {
            if let Token::Identifier(name) = &self.peek().token {
                let name = name.clone();
                self.advance();
                self.consume(Token::Colon, "Expected ':' after export name")?;
                let value = self.expression()?;
                exports.push((name, value));

                if !self.match_token(Token::Comma) {
                    break;
                }
                if self.check(Token::RightBrace) {
                    break;
                }
            } else {
                return Err(ParseError::Generic {
                    message: "Expected export name".to_string(),
                });
            }
        }

        self.consume(Token::RightBrace, "Expected '}' after exports")?;

        Ok(Stmt::Export {
            spec: crate::ast::ExportSpec {
                items: exports,
                span: span.clone(),
            },
            span,
        })
    }

    /// Parse a block of statements
    pub(super) fn parse_block(&mut self) -> ParseResult<Vec<crate::ast::Stmt>> {
        let mut statements = Vec::new();

        while !self.check(Token::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);

            // Handle statement separators (semicolons and newlines)
            self.handle_statement_separator(true)?;
        }

        self.consume(Token::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_statement;

    #[test]
    fn test_parse_simple_statement() {
        let result = parse_statement("return 42");
        assert!(result.is_ok());
    }
}
