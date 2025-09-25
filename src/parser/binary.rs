use super::{ParseResult, Parser};
use crate::ast::Expr;
use crate::token::Token;

impl Parser {
    /// Parse a unary expression
    pub(super) fn unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::Not) || self.match_token(Token::Minus) {
            let op = match &self.previous().token {
                Token::Not => crate::ast::UnaryOp::Not,
                Token::Minus => crate::ast::UnaryOp::Negate,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span,
            });
        }

        self.power()
    }

    /// Parse power expressions (^) - right associative
    pub(super) fn power(&mut self) -> ParseResult<Expr> {
        let mut expr = self.postfix()?;

        if self.match_token(Token::Power) {
            let op = crate::ast::BinaryOp::Power;
            let span = self.previous().span.clone();
            let right = self.power()?; // Right-associative recursion
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse a multiplication/division expression
    pub(super) fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(Token::Divide)
            || self.match_token(Token::Multiply)
            || self.match_token(Token::Modulo)
        {
            let op = match &self.previous().token {
                Token::Divide => crate::ast::BinaryOp::Divide,
                Token::Multiply => crate::ast::BinaryOp::Multiply,
                Token::Modulo => crate::ast::BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse an addition/subtraction expression
    pub(super) fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(Token::Minus) || self.match_token(Token::Plus) {
            let op = match &self.previous().token {
                Token::Minus => crate::ast::BinaryOp::Subtract,
                Token::Plus => crate::ast::BinaryOp::Add,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse range expressions (..)
    pub(super) fn range(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;

        while self.match_token(Token::Range) {
            let op = crate::ast::BinaryOp::Range;
            let span = self.previous().span.clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse a comparison expression
    pub(super) fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.range()?;

        while self.match_token(Token::Greater)
            || self.match_token(Token::GreaterEqual)
            || self.match_token(Token::Less)
            || self.match_token(Token::LessEqual)
        {
            let op = match &self.previous().token {
                Token::Greater => crate::ast::BinaryOp::Greater,
                Token::GreaterEqual => crate::ast::BinaryOp::GreaterEqual,
                Token::Less => crate::ast::BinaryOp::Less,
                Token::LessEqual => crate::ast::BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.range()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse an equality expression
    pub(super) fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(Token::NotEqual) || self.match_token(Token::Equal) {
            let op = match &self.previous().token {
                Token::NotEqual => crate::ast::BinaryOp::NotEqual,
                Token::Equal => crate::ast::BinaryOp::Equal,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse regex match expressions (~, !~)
    pub(super) fn regex_match(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(Token::RegexMatch) || self.match_token(Token::RegexNotMatch) {
            let op = match &self.previous().token {
                Token::RegexMatch => crate::ast::BinaryOp::RegexMatch,
                Token::RegexNotMatch => crate::ast::BinaryOp::RegexNotMatch,
                _ => unreachable!(),
            };
            let span = self.previous().span.clone();
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse logical AND expressions (&&)
    pub(super) fn logical_and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.regex_match()?;

        while self.match_token(Token::And) {
            let op = crate::ast::BinaryOp::And;
            let span = self.previous().span.clone();
            let right = self.regex_match()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse logical OR expressions (||)
    pub(super) fn logical_or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.logical_and()?;

        while self.match_token(Token::Or) {
            let op = crate::ast::BinaryOp::Or;
            let span = self.previous().span.clone();
            let right = self.logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse forward apply pipelines (|>) - left-associative
    pub(super) fn pipe_apply_forward(&mut self) -> ParseResult<Expr> {
        let mut expr = self.logical_or()?;

        while self.match_token(Token::PipeForward) {
            let op = crate::ast::BinaryOp::PipeApplyFwd;
            let span = self.previous().span.clone();
            let right = self.logical_or()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse backward apply pipelines (<|) - right-associative
    pub(super) fn pipe_apply_backward(&mut self) -> ParseResult<Expr> {
        let mut expr = self.pipe_apply_forward()?;

        if self.match_token(Token::PipeBackward) {
            let op = crate::ast::BinaryOp::PipeApplyBwd;
            let span = self.previous().span.clone();
            let right = self.pipe_apply_backward()?; // right-associative
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse stream pipe (|) built over apply-pipe layers - left-associative
    pub(super) fn pipe_stream(&mut self) -> ParseResult<Expr> {
        let mut expr = self.pipe_apply_backward()?;

        while self.match_token(Token::Pipe) {
            let op = crate::ast::BinaryOp::Pipe;
            let span = self.previous().span.clone();
            let right = self.pipe_apply_backward()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    /// Parse assignment expressions (right-associative)
    pub(super) fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.pipe_stream()?;

        if self.match_token(Token::Assign) {
            let span = self.previous().span.clone();
            let value = self.assignment()?; // Right-associative
            return Ok(Expr::Assign {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            });
        }

        // Compound assignment operators
        if self.match_token(Token::PlusAssign) {
            let span = self.previous().span.clone();
            let value = self.assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: crate::ast::CompoundOp::PlusAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::MinusAssign) {
            let span = self.previous().span.clone();
            let value = self.assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: crate::ast::CompoundOp::MinusAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::MultiplyAssign) {
            let span = self.previous().span.clone();
            let value = self.assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: crate::ast::CompoundOp::MultiplyAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::DivideAssign) {
            let span = self.previous().span.clone();
            let value = self.assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: crate::ast::CompoundOp::DivideAssign,
                value: Box::new(value),
                span,
            });
        }

        if self.match_token(Token::ModuloAssign) {
            let span = self.previous().span.clone();
            let value = self.assignment()?; // Right-associative
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: crate::ast::CompoundOp::ModuloAssign,
                value: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    /// Parse an expression
    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    /// Parse an expression but without considering the pipe operator layer.
    /// Useful in contexts (like function parameter defaults) where '|' is a terminator.
    pub(super) fn expression_without_pipe(&mut self) -> ParseResult<Expr> {
        self.logical_or()
    }
}
