//! Statement parsing dispatcher.

use crate::{ParseResult, Parser};
use suji_ast::Stmt;
use suji_lexer::Token;

impl Parser {
    /// Parse a statement - main entry point
    pub fn statement(&mut self) -> ParseResult<Stmt> {
        // Return statement
        if self.match_token(Token::Return) {
            let expr = self.parse_return_expr()?;
            return Ok(Stmt::Expr(expr));
        }

        // Break statement
        if self.match_token(Token::Break) {
            let expr = self.parse_break_expr()?;
            return Ok(Stmt::Expr(expr));
        }

        // Continue statement
        if self.match_token(Token::Continue) {
            let expr = self.parse_continue_expr()?;
            return Ok(Stmt::Expr(expr));
        }

        // Loop statements
        if self.match_token(Token::Loop) {
            return self.parse_loop_statement();
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
        self.parse_expression_statement()
    }
}

pub mod blocks;
pub mod expressions;
pub mod imports;
pub mod loops;
