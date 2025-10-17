use crate::lexer::token::{Span, TokenWithSpan};
use thiserror::Error;

pub type ScannerResult = Result<TokenWithSpan, LexError>;

#[derive(Debug, Clone, Copy)]
pub enum QuoteType {
    Double,
    Single,
}

#[derive(Error, Debug, Clone)]
pub enum LexError {
    #[error("Unterminated string literal")]
    UnterminatedString { span: Span },
    #[error("Unterminated shell command")]
    UnterminatedShellCommand { span: Span },
    #[error("Unterminated regex literal")]
    UnterminatedRegex { span: Span },
    #[error("Invalid escape sequence '\\{escape}'")]
    InvalidEscape { escape: char, span: Span },
    #[error("Invalid number literal '{literal}'")]
    InvalidNumber { literal: String, span: Span },
    #[error("Unexpected character '{ch}'")]
    UnexpectedCharacter { ch: char, span: Span },
}

#[derive(Debug, Clone, Copy)]
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
