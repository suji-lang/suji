use crate::token::{Span, Token, TokenWithSpan};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum QuoteType {
    Double,
    Single,
}

#[derive(Error, Debug, Clone)]
pub enum LexError {
    #[error("Unterminated string literal at line {line}, column {column}")]
    UnterminatedString { line: usize, column: usize },
    #[error("Unterminated shell command at line {line}, column {column}")]
    UnterminatedShellCommand { line: usize, column: usize },
    #[error("Unterminated regex literal at line {line}, column {column}")]
    UnterminatedRegex { line: usize, column: usize },
    #[error("Invalid escape sequence '\\{escape}' at line {line}, column {column}")]
    InvalidEscape {
        escape: char,
        line: usize,
        column: usize,
    },
    #[error("Invalid number literal '{literal}' at line {line}, column {column}")]
    InvalidNumber {
        literal: String,
        line: usize,
        column: usize,
    },
    #[error("Unexpected character '{ch}' at line {line}, column {column}")]
    UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone)]
pub enum LexState {
    Normal,
    InString {
        start_pos: usize,
        quote_type: QuoteType,
        multiline: bool,
    },
    InStringInterp {
        start_pos: usize,
        quote_type: QuoteType,
        multiline: bool,
        brace_depth: usize,
    },
    InShellCommand {
        start_pos: usize,
    },
    InShellInterp {
        start_pos: usize,
        brace_depth: usize,
    },
    InRegex {
        start_pos: usize,
    },
    StringContentReturned {
        start_pos: usize,
        quote_type: QuoteType,
        multiline: bool,
    },
    RegexContentReturned {
        start_pos: usize,
    },
    ShellContentReturned {
        start_pos: usize,
    },
}

impl LexState {
    /// Returns true if the current state is in an interpolation context
    pub fn is_in_interpolation(&self) -> bool {
        matches!(
            self,
            LexState::InStringInterp { .. } | LexState::InShellInterp { .. }
        )
    }

    /// Returns true if the current state is in a string-like context
    pub fn is_in_string_like(&self) -> bool {
        matches!(
            self,
            LexState::InString { .. }
                | LexState::InStringInterp { .. }
                | LexState::InShellCommand { .. }
                | LexState::InShellInterp { .. }
        )
    }

    /// Returns the start position for the current context
    pub fn start_pos(&self) -> Option<usize> {
        match self {
            LexState::InString { start_pos, .. }
            | LexState::InStringInterp { start_pos, .. }
            | LexState::InShellCommand { start_pos }
            | LexState::InShellInterp { start_pos, .. }
            | LexState::InRegex { start_pos }
            | LexState::StringContentReturned { start_pos, .. }
            | LexState::RegexContentReturned { start_pos }
            | LexState::ShellContentReturned { start_pos } => Some(*start_pos),
            LexState::Normal => None,
        }
    }
}

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
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    pub fn peek_next(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    pub fn peek_next_next(&self) -> Option<char> {
        self.input.chars().nth(self.position + 2)
    }

    pub fn advance(&mut self) -> char {
        let ch = self.input.chars().nth(self.position).unwrap_or('\0');
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

/// Result type for state scanner operations
pub type ScannerResult = Result<TokenWithSpan, LexError>;

/// Trait for state scanners
pub trait StateScanner {
    fn scan(&mut self, context: &mut ScannerContext, state: &LexState) -> ScannerResult;
}

pub mod interpolation;
pub mod normal;
pub mod regex;
pub mod shell;
pub mod string;
