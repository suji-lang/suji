use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::{Expr, Literal};
use suji_lexer::token::Token;

impl Parser {
    /// Parse basic literals: true, false, nil, numbers, identifiers
    pub fn parse_literals(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::True) {
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Boolean(true, span)));
        }

        if self.match_token(Token::False) {
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Boolean(false, span)));
        }

        if self.match_token(Token::Nil) {
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Nil(span)));
        }

        if self.match_token(Token::Underscore) {
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Identifier("_".to_string(), span)));
        }

        if let Token::Number(n) = self.peek().token {
            self.advance();
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Number(n, span)));
        }

        if let Token::Identifier(name) = self.peek().token {
            self.advance();
            let span = self.previous().span.clone();
            return Ok(Expr::Literal(Literal::Identifier(name, span)));
        }

        let current = self.peek();
        Err(ParseError::UnexpectedToken {
            token: current.token,
            span: current.span,
        })
    }
}
