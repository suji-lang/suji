use crate::span::Span;
use crate::states::context::ParentInterpolation;
use crate::states::interpolation::InterpolationScanner;
use crate::states::normal::NormalScanner;
use crate::states::regex::RegexScanner;
use crate::states::shell::ShellScanner;
use crate::states::string::StringScanner;
use crate::states::{LexState, QuoteType, ScannerContext};
use crate::token::{SpannedToken, Token};
use thiserror::Error;

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

pub struct Lexer<'a> {
    context: ScannerContext<'a>,
    state: LexState,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            context: ScannerContext::new(input),
            state: LexState::Normal,
        }
    }

    pub fn lex(input: &str) -> Result<Vec<SpannedToken>, LexError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();

        loop {
            let token_with_span = lexer.next_token()?;
            let is_eof = matches!(token_with_span.token, Token::Eof);
            tokens.push(token_with_span);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<SpannedToken, LexError> {
        match self.state {
            LexState::Normal => self.scan_normal_token(),
            LexState::InString {
                start_pos,
                quote_type,
                multiline,
            } => self.scan_string_content(start_pos, quote_type, multiline),
            LexState::InStringInterp {
                start_pos,
                quote_type: _,
                multiline: _,
                brace_depth,
            } => self.scan_string_interpolation(start_pos, brace_depth),
            LexState::InShellCommand { start_pos } => self.scan_shell_content(start_pos),
            LexState::InShellInterp {
                start_pos,
                brace_depth,
            } => self.scan_shell_interpolation(start_pos, brace_depth),
            LexState::InRegex { start_pos } => self.scan_regex_content(start_pos),
            LexState::StringContentReturned { start_pos, .. } => {
                self.handle_content_returned_state(Token::StringEnd, start_pos)
            }
            LexState::RegexContentReturned { start_pos } => {
                self.handle_content_returned_state(Token::RegexEnd, start_pos)
            }
            LexState::ShellContentReturned { start_pos } => {
                self.handle_content_returned_state(Token::ShellEnd, start_pos)
            }
        }
    }

    /// Helper function to handle content returned states
    fn handle_content_returned_state(
        &mut self,
        token: Token,
        start_pos: usize,
    ) -> Result<SpannedToken, LexError> {
        // Check if we should return to a parent interpolation context
        self.state = if let Some(parent) = self.context.interpolation_stack.pop() {
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

        self.context.prev_token = Some(token.clone());
        let span = Span::new(
            start_pos,
            self.context.position,
            self.context.line,
            self.context.column,
        );
        Ok(SpannedToken::new(token, span))
    }

    fn scan_normal_token(&mut self) -> Result<SpannedToken, LexError> {
        NormalScanner::scan_token(&mut self.context, &mut self.state)
    }

    fn scan_string_content(
        &mut self,
        start_pos: usize,
        quote_type: QuoteType,
        multiline: bool,
    ) -> Result<SpannedToken, LexError> {
        let quote_char = match quote_type {
            QuoteType::Double => '"',
            QuoteType::Single => '\'',
        };
        StringScanner::scan_content(
            &mut self.context,
            &mut self.state,
            start_pos,
            quote_char,
            multiline,
        )
    }

    fn scan_string_interpolation(
        &mut self,
        start_pos: usize,
        brace_depth: usize,
    ) -> Result<SpannedToken, LexError> {
        InterpolationScanner::scan_string_interpolation(
            &mut self.context,
            &mut self.state,
            start_pos,
            brace_depth,
        )
    }

    fn scan_shell_content(&mut self, start_pos: usize) -> Result<SpannedToken, LexError> {
        ShellScanner::scan_content(&mut self.context, &mut self.state, start_pos)
    }

    fn scan_shell_interpolation(
        &mut self,
        start_pos: usize,
        brace_depth: usize,
    ) -> Result<SpannedToken, LexError> {
        InterpolationScanner::scan_shell_interpolation(
            &mut self.context,
            &mut self.state,
            start_pos,
            brace_depth,
        )
    }

    fn scan_regex_content(&mut self, start_pos: usize) -> Result<SpannedToken, LexError> {
        RegexScanner::scan_content(&mut self.context, &mut self.state, start_pos)
    }
}
