use crate::lexer::LexError;
use crate::token::SpannedToken;

pub type ScannerResult = Result<SpannedToken, LexError>;

#[derive(Debug, Clone, Copy)]
pub enum QuoteType {
    Double,
    Single,
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
    },
    RegexContentReturned {
        start_pos: usize,
    },
    ShellContentReturned {
        start_pos: usize,
    },
}
