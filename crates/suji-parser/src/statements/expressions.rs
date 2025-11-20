use crate::{ParseResult, Parser};
use suji_ast::Stmt;

impl Parser {
    /// Parse expression statement (default case when no other statement type matches)
    pub(super) fn parse_expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        Ok(Stmt::Expr(expr))
    }
}
