use super::ExpressionContext;
use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::Expr;
use suji_lexer::token::{Span, Token, TokenWithSpan};

impl Parser {
    /// Check if we're at the end of the token stream
    pub(super) fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().token == Token::Eof
    }

    /// Consume an identifier token and return its name and span
    pub(super) fn consume_identifier(&mut self) -> ParseResult<(String, Span)> {
        if let Token::Identifier(name) = &self.peek().token {
            let name = name.clone();
            let span = self.advance().span.clone();
            Ok((name, span))
        } else {
            let current = self.peek();
            Err(ParseError::ExpectedToken {
                expected: Token::Identifier(String::new()),
                found: current.token,
                span: current.span,
            })
        }
    }

    /// Parse a colon-separated path starting from a known first identifier.
    pub(super) fn parse_colon_path_from(
        &mut self,
        first_segment: String,
        first_span: Span,
        require_additional_segment: bool,
    ) -> ParseResult<(Vec<String>, Span)> {
        let mut segments = vec![first_segment];
        let mut end_span = first_span.clone();

        let mut saw_additional = false;
        while self.match_token(Token::Colon) {
            if let Token::Identifier(_) = &self.peek().token {
                let (segment, span) = self.consume_identifier()?;
                end_span = span;
                segments.push(segment);
                saw_additional = true;
            } else {
                // Trailing ':' without an identifier
                if require_additional_segment && !saw_additional {
                    let current = self.peek();
                    return Err(ParseError::InvalidImportPath { span: current.span });
                }
                break;
            }
        }

        let span = Span::new(
            first_span.start,
            end_span.end,
            first_span.line,
            first_span.column,
        );
        Ok((segments, span))
    }

    /// Get the current token without advancing
    pub(super) fn peek(&self) -> TokenWithSpan {
        self.tokens
            .get(self.current)
            .cloned()
            .unwrap_or(TokenWithSpan {
                token: Token::Eof,
                span: Span::new(0, 0, 0, 0),
            })
    }

    /// Get the previous token
    pub(super) fn previous(&self) -> &TokenWithSpan {
        &self.tokens[self.current - 1]
    }

    /// Advance to the next token and return the current one
    pub(super) fn advance(&mut self) -> &TokenWithSpan {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.skip_newlines_and_comments();
        self.previous()
    }

    /// Skip newline tokens
    pub(super) fn skip_newlines(&mut self) {
        while !self.is_at_end() && self.peek().token == Token::Newline {
            self.current += 1;
        }
    }

    /// Skip comment tokens
    pub(super) fn skip_comments(&mut self) {
        while !self.is_at_end() && matches!(self.peek().token, Token::Comment(_)) {
            self.current += 1;
        }
    }

    /// Skip both newlines and comments
    pub(super) fn skip_newlines_and_comments(&mut self) {
        loop {
            let old_position = self.current;
            self.skip_newlines();
            self.skip_comments();
            // If position didn't change, we're done
            if self.current == old_position {
                break;
            }
        }
    }

    /// Check if the current token matches the given type
    pub(super) fn check(&self, token_type: Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token) == std::mem::discriminant(&token_type)
    }

    /// Consume a token if it matches the given type
    pub(super) fn match_token(&mut self, token_type: Token) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume a token, returning an error if it doesn't match
    pub(super) fn consume(
        &mut self,
        token_type: Token,
        _message: &str,
    ) -> ParseResult<&TokenWithSpan> {
        if self.check(token_type.clone()) {
            Ok(self.advance())
        } else {
            let current = self.peek();
            Err(ParseError::ExpectedToken {
                expected: token_type,
                found: current.token,
                span: current.span,
            })
        }
    }

    /// Parse an expression for use as a map key (no postfix operators to avoid confusion with ":")
    pub(super) fn map_key_expression(&mut self) -> ParseResult<Expr> {
        self.expression_in_context(ExpressionContext::NoPostfix)
    }

    /// Parse expressions used as array indices (without postfix operations to avoid conflicts with ':')
    pub(super) fn index_expression(&mut self) -> ParseResult<Expr> {
        self.expression_in_context(ExpressionContext::NoPostfix)
    }

    /// Handle statement separators (semicolons and newlines)
    /// This method handles the common logic for parsing statements with optional semicolon separators
    pub(super) fn handle_statement_separator(&mut self, is_block_context: bool) -> ParseResult<()> {
        // Check for semicolon separator
        if self.match_token(Token::Semicolon) {
            // Consume any additional consecutive semicolons (they're just noise)
            while self.match_token(Token::Semicolon) {
                // Keep consuming semicolons
            }
            return Ok(());
        } else if is_block_context && !self.check(Token::RightBrace) && !self.is_at_end() {
            // In block context, if no semicolon and not at end, expect newline
            self.skip_newlines();
        } else if !is_block_context && !self.is_at_end() {
            // In top-level context, if no semicolon and not at end, expect newline
            self.skip_newlines();
        }
        Ok(())
    }
}
