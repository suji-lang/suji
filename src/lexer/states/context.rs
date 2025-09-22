use crate::lexer::token::{Span, Token, TokenWithSpan};

/// Context information passed to state scanners
#[derive(Debug, Clone)]
pub struct ScannerContext<'a> {
    pub input: &'a str,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub prev_token: Option<Token>,
}
impl<'a> ScannerContext<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            prev_token: None,
        }
    }

    pub fn advance_position(&mut self, ch: char) {
        self.position += ch.len_utf8();
        if ch != '\n' {
            self.column += 1;
        } else {
            self.line += 1;
            self.column = 1;
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    pub fn peek(&self) -> char {
        self.input[self.position..].chars().next().unwrap_or('\0')
    }

    pub fn peek_next(&self) -> Option<char> {
        self.input[self.position..].chars().nth(1)
    }

    pub fn peek_next_next(&self) -> Option<char> {
        self.input[self.position..].chars().nth(2)
    }

    pub fn advance(&mut self) -> char {
        let ch = self.input[self.position..].chars().next().unwrap_or('\0');
        self.advance_position(ch);
        ch
    }

    pub fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    pub fn make_token(&self, token: Token) -> TokenWithSpan {
        let span = Span::new(self.position, self.position, self.line, self.column);
        TokenWithSpan::new(token, span)
    }

    pub fn make_token_with_span(
        &self,
        token: Token,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> TokenWithSpan {
        let span = Span::new(start_pos, self.position, start_line, start_column);
        TokenWithSpan::new(token, span)
    }

    pub fn should_parse_as_regex(&self) -> bool {
        // If no previous token or previous token cannot end an expression, treat as regex
        match &self.prev_token {
            None => true,
            Some(token) => !token.can_end_expression(),
        }
    }
}
