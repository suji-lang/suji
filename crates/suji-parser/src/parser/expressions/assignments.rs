use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::Expr;
use suji_lexer::token::Token;

impl Parser {
    pub(crate) fn looks_like_destructure_pattern(&self) -> bool {
        let mut index = self.current;
        let mut expect_binding = false;
        let mut saw_comma = false;

        while index < self.tokens.len() {
            match &self.tokens[index].token {
                Token::Comma => {
                    saw_comma = true;
                    expect_binding = true;
                    index += 1;
                }
                Token::Identifier(_) | Token::Underscore => {
                    if !expect_binding {
                        return false;
                    }
                    expect_binding = false;
                    index += 1;
                }
                Token::Assign => {
                    return saw_comma && !expect_binding;
                }
                Token::Newline | Token::Comment(_) => {
                    index += 1;
                }
                _ => return false,
            }
        }

        false
    }

    pub(crate) fn parse_destructure_pattern(&mut self, first_expr: Expr) -> ParseResult<Expr> {
        let mut elements = Vec::new();
        let first_span = first_expr.span().clone();
        elements.push(self.ensure_destructure_element(first_expr)?);

        let mut last_span = first_span.clone();

        while self.match_token(Token::Comma) {
            if self.check(Token::Assign) {
                return Err(ParseError::Generic {
                    message: "Expected binding after ',' in destructuring assignment".to_string(),
                });
            }

            let next_expr = self.parse_pipe_apply_backward()?;
            let validated = self.ensure_destructure_element(next_expr)?;
            last_span = validated.span().clone();
            elements.push(validated);
        }

        if !self.check(Token::Assign) {
            return Err(ParseError::Generic {
                message: "Expected '=' after destructuring assignment target".to_string(),
            });
        }

        if elements.len() < 2 {
            return Err(ParseError::Generic {
                message: "Destructuring assignment requires at least two bindings".to_string(),
            });
        }

        let mut span = first_span;
        span.end = last_span.end;

        Ok(Expr::Destructure { elements, span })
    }

    pub(crate) fn convert_tuple_to_destructure(
        &self,
        elements: Vec<Expr>,
        span: suji_ast::Span,
    ) -> ParseResult<Expr> {
        if elements.is_empty() {
            return Err(ParseError::Generic {
                message: "Cannot destructure an empty tuple".to_string(),
            });
        }

        let mut converted = Vec::new();
        for element in elements {
            converted.push(self.ensure_destructure_element(element)?);
        }

        Ok(Expr::Destructure {
            elements: converted,
            span,
        })
    }

    pub(crate) fn ensure_destructure_element(&self, expr: Expr) -> ParseResult<Expr> {
        match &expr {
            Expr::Literal(suji_ast::ast::Literal::Identifier(_, _)) => Ok(expr),
            _ => Err(ParseError::Generic {
                message: "Destructuring assignments only support identifiers or '_' bindings"
                    .to_string(),
            }),
        }
    }
}
