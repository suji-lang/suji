use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::Expr;
use suji_lexer::token::{Span, Token, TokenWithSpan};

impl Parser {
    /// Check if we're at the end of the token stream
    pub(super) fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().token == Token::Eof
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
        message: &str,
    ) -> ParseResult<&TokenWithSpan> {
        if self.check(token_type.clone()) {
            Ok(self.advance())
        } else {
            let current = self.peek();
            Err(ParseError::Generic {
                message: format!(
                    "{}: expected {:?}, found {:?}",
                    message, token_type, current.token
                ),
            })
        }
    }

    /// Parse an expression for use as a map key (no postfix operators to avoid confusion with ":")
    pub(super) fn map_key_expression(&mut self) -> ParseResult<Expr> {
        self.assignment_without_postfix()
    }

    /// Parse expressions used as array indices (without postfix operations to avoid conflicts with ':')
    pub(super) fn index_expression(&mut self) -> ParseResult<Expr> {
        self.assignment_without_postfix()
    }

    /// Parse assignment without postfix operators
    fn assignment_without_postfix(&mut self) -> ParseResult<Expr> {
        let expr = self.logical_or_without_postfix()?;

        if self.match_token(Token::Assign) {
            let span = self.previous().span.clone();
            let value = self.assignment_without_postfix()?; // Right-associative
            return Ok(Expr::Assign {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    /// Parse logical OR without postfix operators
    fn logical_or_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.logical_and_without_postfix()?;

        while self.match_token(Token::Or) {
            let op = suji_ast::ast::BinaryOp::Or;
            let span = self.previous().span.clone();
            let right = self.logical_and_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse logical AND without postfix operators
    fn logical_and_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.regex_match_without_postfix()?;

        while self.match_token(Token::And) {
            let op = suji_ast::ast::BinaryOp::And;
            let span = self.previous().span.clone();
            let right = self.regex_match_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse regex match without postfix operators
    fn regex_match_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality_without_postfix()?;

        while self.match_token(Token::RegexMatch) || self.match_token(Token::RegexNotMatch) {
            let op = match &self.previous().token {
                Token::RegexMatch => suji_ast::ast::BinaryOp::RegexMatch,
                Token::RegexNotMatch => suji_ast::ast::BinaryOp::RegexNotMatch,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.equality_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse equality without postfix operators
    fn equality_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison_without_postfix()?;

        while self.match_token(Token::NotEqual) || self.match_token(Token::Equal) {
            let op = match &self.previous().token {
                Token::NotEqual => suji_ast::ast::BinaryOp::NotEqual,
                Token::Equal => suji_ast::ast::BinaryOp::Equal,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.comparison_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse comparison without postfix operators
    fn comparison_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term_without_postfix()?;

        while self.match_token(Token::Greater)
            || self.match_token(Token::GreaterEqual)
            || self.match_token(Token::Less)
            || self.match_token(Token::LessEqual)
        {
            let op = match &self.previous().token {
                Token::Greater => suji_ast::ast::BinaryOp::Greater,
                Token::GreaterEqual => suji_ast::ast::BinaryOp::GreaterEqual,
                Token::Less => suji_ast::ast::BinaryOp::Less,
                Token::LessEqual => suji_ast::ast::BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.term_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse term without postfix operators
    fn term_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.range_without_postfix()?;

        while self.match_token(Token::Minus) || self.match_token(Token::Plus) {
            let op = match &self.previous().token {
                Token::Minus => suji_ast::ast::BinaryOp::Subtract,
                Token::Plus => suji_ast::ast::BinaryOp::Add,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.range_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse range without postfix operators  
    fn range_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor_without_postfix()?;

        while self.match_token(Token::Range) {
            let op = suji_ast::ast::BinaryOp::Range;
            let span = self.previous().span.clone();
            let right = self.factor_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse factor without postfix operators
    fn factor_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.power_without_postfix()?;

        while self.match_token(Token::Divide)
            || self.match_token(Token::Multiply)
            || self.match_token(Token::Modulo)
        {
            let op = match &self.previous().token {
                Token::Divide => suji_ast::ast::BinaryOp::Divide,
                Token::Multiply => suji_ast::ast::BinaryOp::Multiply,
                Token::Modulo => suji_ast::ast::BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.power_without_postfix()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse power without postfix operators
    fn power_without_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary_without_postfix()?;

        if self.match_token(Token::Power) {
            let op = suji_ast::ast::BinaryOp::Power;
            let span = self.previous().span.clone();
            let right = self.power_without_postfix()?; // Right-associative recursion
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse unary without postfix operators
    fn unary_without_postfix(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::Not) || self.match_token(Token::Minus) {
            let op = match &self.previous().token {
                Token::Not => suji_ast::ast::UnaryOp::Not,
                Token::Minus => suji_ast::ast::UnaryOp::Negate,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let expr = self.unary_without_postfix()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span,
            });
        }

        self.primary()
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
