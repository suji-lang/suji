use super::{ParseResult, Parser};
use suji_ast::ast::Stmt;
use suji_lexer::token::{Span, Token};

impl Parser {
    /// Parse loop statement: loop (as label)? { ... } or loop through ...
    pub(super) fn parse_loop_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        // Check for "loop through"
        if self.match_token(Token::Through) {
            return self.parse_loop_through_statement(span);
        }

        // Parse optional label: loop as label { ... }
        let label = if self.match_token(Token::As) {
            let (name, _span) = self.consume_identifier()?;
            Some(name)
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
            let (var1, _span1) = self.consume_identifier()?;
            if self.match_token(Token::Comma) {
                let (var2, _span2) = self.consume_identifier()?;
                suji_ast::ast::LoopBindings::Two(var1, var2)
            } else {
                suji_ast::ast::LoopBindings::One(var1)
            }
        } else {
            suji_ast::ast::LoopBindings::None
        };

        // Parse optional label: as label
        let label = if self.match_token(Token::As) {
            let (name, _span) = self.consume_identifier()?;
            Some(name)
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
