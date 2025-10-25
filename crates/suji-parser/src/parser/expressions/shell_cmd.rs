use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::{Expr, StringPart};
use suji_lexer::token::Token;

impl Parser {
    /// Parse shell command template `command ${var} more`
    pub fn parse_shell_command(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::ShellStart) {
            return self.parse_shell_command_template();
        }

        let current = self.peek();
        Err(ParseError::UnexpectedToken {
            token: current.token,
            span: current.span,
        })
    }

    /// Parse shell command template `command ${var} more`
    fn parse_shell_command_template(&mut self) -> ParseResult<Expr> {
        let start_span = self.previous().span.clone();
        let mut parts = Vec::new();

        while !self.check(Token::ShellEnd) && !self.is_at_end() {
            if let Token::StringText(text) = &self.peek().token {
                let text = text.clone();
                self.advance();
                parts.push(StringPart::Text(text));
            } else if self.match_token(Token::InterpStart) {
                // Parse interpolated expression
                let expr = self.expression()?;
                parts.push(StringPart::Expr(expr));
                self.consume(
                    Token::InterpEnd,
                    "Expected '}' after interpolated expression",
                )?;
            } else {
                let current = self.peek();
                return Err(ParseError::UnexpectedToken {
                    token: current.token,
                    span: current.span,
                });
            }
        }

        // Validate no control flow in shell command parts
        for part in &parts {
            if part.has_control_flow() {
                return Err(ParseError::Generic {
                    message: "Control flow expressions (return, break, continue) are not allowed in shell command templates".to_string(),
                });
            }
        }

        self.consume(Token::ShellEnd, "Expected end of shell command")?;
        Ok(Expr::ShellCommandTemplate {
            parts,
            span: start_span,
        })
    }
}
