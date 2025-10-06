use super::{ParseResult, Parser};
use suji_ast::ast::{Expr, Literal};
use suji_lexer::token::Token;

impl Parser {
    // Generic helpers for infix parsing layers
    fn parse_infix_left_layer<F>(
        &mut self,
        next: fn(&mut Parser) -> ParseResult<Expr>,
        mut mapper: F,
    ) -> ParseResult<Expr>
    where
        F: FnMut(&Token) -> Option<suji_ast::ast::BinaryOp>,
    {
        let mut expr = next(self)?;
        while let Some(op) = mapper(&self.peek().token) {
            let span = self.advance().span.clone();
            let right = next(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_infix_right_layer<F>(
        &mut self,
        next: fn(&mut Parser) -> ParseResult<Expr>,
        mut mapper: F,
    ) -> ParseResult<Expr>
    where
        F: FnMut(&Token) -> Option<suji_ast::ast::BinaryOp>,
    {
        let mut expr = next(self)?;
        if let Some(op) = mapper(&self.peek().token) {
            let span = self.advance().span.clone();
            let right = self.parse_infix_right_layer(next, mapper)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    /// Parse a unary expression
    pub(super) fn parse_unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::Not) || self.match_token(Token::Minus) {
            let op = match &self.previous().token {
                Token::Not => suji_ast::ast::UnaryOp::Not,
                Token::Minus => suji_ast::ast::UnaryOp::Negate,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_power()
    }

    /// Parse power expressions (^) - right associative
    pub(super) fn parse_power(&mut self) -> ParseResult<Expr> {
        self.parse_infix_right_layer(Parser::postfix, |t| match t {
            Token::Power => Some(suji_ast::ast::BinaryOp::Power),
            _ => None,
        })
    }

    /// Parse a multiplication/division expression
    pub(super) fn parse_factor(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_unary, |t| match t {
            Token::Divide => Some(suji_ast::ast::BinaryOp::Divide),
            Token::Multiply => Some(suji_ast::ast::BinaryOp::Multiply),
            Token::Modulo => Some(suji_ast::ast::BinaryOp::Modulo),
            _ => None,
        })
    }

    /// Parse an addition/subtraction expression
    pub(super) fn parse_term(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_factor, |t| match t {
            Token::Minus => Some(suji_ast::ast::BinaryOp::Subtract),
            Token::Plus => Some(suji_ast::ast::BinaryOp::Add),
            _ => None,
        })
    }

    /// Parse range expressions (..)
    pub(super) fn parse_range(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_term, |t| match t {
            Token::Range => Some(suji_ast::ast::BinaryOp::Range),
            _ => None,
        })
    }

    /// Parse a comparison expression
    pub(super) fn parse_comparison(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_range, |t| match t {
            Token::Greater => Some(suji_ast::ast::BinaryOp::Greater),
            Token::GreaterEqual => Some(suji_ast::ast::BinaryOp::GreaterEqual),
            Token::Less => Some(suji_ast::ast::BinaryOp::Less),
            Token::LessEqual => Some(suji_ast::ast::BinaryOp::LessEqual),
            _ => None,
        })
    }

    /// Parse an equality expression
    pub(super) fn parse_equality(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_comparison, |t| match t {
            Token::NotEqual => Some(suji_ast::ast::BinaryOp::NotEqual),
            Token::Equal => Some(suji_ast::ast::BinaryOp::Equal),
            _ => None,
        })
    }

    /// Parse regex match expressions (~, !~)
    pub(super) fn parse_regex_match(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_equality, |t| match t {
            Token::RegexMatch => Some(suji_ast::ast::BinaryOp::RegexMatch),
            Token::RegexNotMatch => Some(suji_ast::ast::BinaryOp::RegexNotMatch),
            _ => None,
        })
    }

    /// Parse logical AND expressions (&&)
    pub(super) fn parse_logical_and(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_regex_match, |t| match t {
            Token::And => Some(suji_ast::ast::BinaryOp::And),
            _ => None,
        })
    }

    /// Parse logical OR expressions (||)
    pub(super) fn parse_logical_or(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_logical_and, |t| match t {
            Token::Or => Some(suji_ast::ast::BinaryOp::Or),
            _ => None,
        })
    }

    /// Parse function composition operators (>> and <<) - left-associative
    pub(super) fn parse_composition_layer(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_logical_or, |t| match t {
            Token::ComposeRight => Some(suji_ast::ast::BinaryOp::ComposeRight),
            Token::ComposeLeft => Some(suji_ast::ast::BinaryOp::ComposeLeft),
            _ => None,
        })
    }

    /// Parse forward apply pipelines (|>) - left-associative
    pub(super) fn parse_pipe_apply_forward(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_composition_layer, |t| match t {
            Token::PipeForward => Some(suji_ast::ast::BinaryOp::PipeApplyFwd),
            _ => None,
        })
    }

    /// Parse backward apply pipelines (<|) - right-associative
    pub(super) fn parse_pipe_apply_backward(&mut self) -> ParseResult<Expr> {
        self.parse_infix_right_layer(Parser::parse_pipe_apply_forward, |t| match t {
            Token::PipeBackward => Some(suji_ast::ast::BinaryOp::PipeApplyBwd),
            _ => None,
        })
    }

    /// Parse stream pipe (|) built over apply-pipe layers - left-associative
    pub(super) fn parse_pipe_stream(&mut self) -> ParseResult<Expr> {
        self.parse_infix_left_layer(Parser::parse_pipe_apply_backward, |t| match t {
            Token::Pipe => Some(suji_ast::ast::BinaryOp::Pipe),
            _ => None,
        })
    }

    /// Parse assignment expressions (right-associative)
    pub(super) fn parse_assignment(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_pipe_stream()?;

        if self.check(Token::Comma) && self.looks_like_destructure_pattern() {
            expr = self.parse_destructure_pattern(expr)?;
        }

        if self.match_token(Token::Assign) {
            let span = self.previous().span.clone();
            let expr = match expr {
                Expr::Literal(Literal::Tuple(elements, tuple_span)) => {
                    self.convert_tuple_to_destructure(elements, tuple_span)?
                }
                other => other,
            };

            let value = self.parse_assignment()?; // Right-associative
            return Ok(Expr::Assign {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            });
        }

        // Compound assignment operators
        if self.match_token(Token::PlusAssign) {
            let span = self.previous().span.clone();
            let value = self.parse_assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: suji_ast::ast::CompoundOp::PlusAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::MinusAssign) {
            let span = self.previous().span.clone();
            let value = self.parse_assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: suji_ast::ast::CompoundOp::MinusAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::MultiplyAssign) {
            let span = self.previous().span.clone();
            let value = self.parse_assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: suji_ast::ast::CompoundOp::MultiplyAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::DivideAssign) {
            let span = self.previous().span.clone();
            let value = self.parse_assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: suji_ast::ast::CompoundOp::DivideAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::ModuloAssign) {
            let span = self.previous().span.clone();
            let value = self.parse_assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: suji_ast::ast::CompoundOp::ModuloAssign,
                value: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    /// Parse an expression
    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.parse_assignment()
    }

    /// Parse an expression but without considering the pipe operator layer.
    /// Useful in contexts (like function parameter defaults) where '|' is a terminator.
    pub(super) fn parse_expression_without_pipe(&mut self) -> ParseResult<Expr> {
        self.parse_logical_or()
    }
}
