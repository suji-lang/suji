use super::{ParseResult, Parser};
use suji_ast::ast::Stmt;
use suji_lexer::token::Token;

impl Parser {
    /// Parse block statement: { stmt1; stmt2; ... }
    pub(super) fn parse_block_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();
        let statements = self.parse_block()?;
        Ok(Stmt::Block { statements, span })
    }

    /// Parse a block of statements
    pub fn parse_block(&mut self) -> ParseResult<Vec<suji_ast::ast::Stmt>> {
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
