pub mod assignments;
pub mod collections;
pub mod functions;
pub mod literals;
pub mod match_expr;
pub mod shell_cmd;

use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::Expr;
use suji_lexer::token::Token;

impl Parser {
    /// Parse a primary expression - main dispatcher
    pub(super) fn primary(&mut self) -> ParseResult<Expr> {
        // Try parsing literals first
        if let Ok(expr) = self.parse_literals() {
            return Ok(expr);
        }

        // Allow tuple/grouping to appear in primary contexts (e.g., destructuring)
        if self.match_token(Token::LeftParen) {
            return self.parse_tuple_or_grouping();
        }

        // Try parsing collections
        if let Ok(expr) = self.parse_collections() {
            return Ok(expr);
        }

        // Try parsing functions
        if let Ok(expr) = self.parse_functions() {
            return Ok(expr);
        }

        // Try parsing match expressions
        if let Ok(expr) = self.parse_match_expression() {
            return Ok(expr);
        }

        // Try parsing shell commands
        if let Ok(expr) = self.parse_shell_command() {
            return Ok(expr);
        }

        // If none match, return error
        let current = self.peek();
        Err(ParseError::UnexpectedToken {
            token: current.token,
            span: current.span,
        })
    }
}
