// Main dispatcher for statement parsing
// This module coordinates between different statement types

use super::{ParseError, ParseResult, Parser};
use crate::ast::Stmt;
use crate::token::Token;

impl Parser {
    /// Parse a statement - main entry point
    pub fn statement(&mut self) -> ParseResult<Stmt> {
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

// Re-export all statement parsing modules
pub mod blocks;
pub mod control_flow;
pub mod expressions;
pub mod imports;

#[cfg(test)]
mod tests {
    use crate::parser::parse_statement;

    #[test]
    fn test_parse_simple_statement() {
        let result = parse_statement("return 42");
        assert!(result.is_ok());
    }
}
