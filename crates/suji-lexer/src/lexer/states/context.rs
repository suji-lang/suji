use super::QuoteType;
use super::types::LexState;
use crate::lexer::token::{Span, Token, TokenWithSpan};

/// Represents a saved interpolation state to return to
#[derive(Debug, Clone)]
pub enum ParentInterpolation {
    String {
        start_pos: usize,
        quote_type: QuoteType,
        multiline: bool,
        brace_depth: usize,
    },
    Shell {
        start_pos: usize,
        brace_depth: usize,
    },
}

/// Context information passed to state scanners
#[derive(Debug, Clone)]
pub struct ScannerContext<'a> {
    pub input: &'a str,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub prev_token: Option<Token>,
    pub interpolation_stack: Vec<ParentInterpolation>,
}
impl<'a> ScannerContext<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            prev_token: None,
            interpolation_stack: Vec::new(),
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

    /// Helper to finalize a delimited construct (string/shell/regex):
    /// - Pops any parent interpolation context and sets the next `LexState`
    /// - Sets `prev_token`
    /// - Builds and returns the provided end token with a span from `start_pos`
    pub fn finalize_end_token(
        &mut self,
        state: &mut LexState,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
        end_token: Token,
    ) -> TokenWithSpan {
        // Decide next state based on interpolation stack
        *state = if let Some(parent) = self.interpolation_stack.pop() {
            match parent {
                ParentInterpolation::String {
                    start_pos,
                    quote_type,
                    multiline,
                    brace_depth,
                } => LexState::InStringInterp {
                    start_pos,
                    quote_type,
                    multiline,
                    brace_depth,
                },
                ParentInterpolation::Shell {
                    start_pos,
                    brace_depth,
                } => LexState::InShellInterp {
                    start_pos,
                    brace_depth,
                },
            }
        } else {
            LexState::Normal
        };

        self.prev_token = Some(end_token.clone());
        let span = Span::new(start_pos, self.position, start_line, start_column);
        TokenWithSpan::new(end_token, span)
    }
}
