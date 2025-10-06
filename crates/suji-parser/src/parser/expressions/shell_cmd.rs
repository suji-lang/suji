use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::{Expr, StringPart};
use suji_lexer::token::Token;

impl Parser {
    /// Parse shell command template `command ${var} more`
    pub fn parse_shell_command(&mut self) -> ParseResult<Expr> {
        if self.match_token(Token::ShellStart) {
            return self.parse_shell_command_template();
        }

        Err(ParseError::Generic {
            message: "Not a shell command".to_string(),
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
                return Err(ParseError::Generic {
                    message: "Unexpected token in shell command template".to_string(),
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
