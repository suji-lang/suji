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
