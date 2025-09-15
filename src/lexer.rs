use crate::lexer::states::interpolation::InterpolationScanner;
use crate::lexer::states::normal::NormalScanner;
use crate::lexer::states::regex::RegexScanner;
use crate::lexer::states::shell::ShellScanner;
use crate::lexer::states::string::StringScanner;
use crate::lexer::states::{LexState, ScannerContext};
use crate::token::{Span, Token, TokenWithSpan};

pub mod states;
pub mod utils;

// Re-export commonly used types
pub use states::LexError;

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

    pub fn lex(input: &str) -> Result<Vec<TokenWithSpan>, LexError> {
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

    fn next_token(&mut self) -> Result<TokenWithSpan, LexError> {
        match self.state.clone() {
            LexState::Normal => self.scan_normal_token(),
            LexState::InString { start_pos } => self.scan_string_content(start_pos),
            LexState::InSingleString { start_pos } => self.scan_single_string_content(start_pos),
            LexState::InStringInterp {
                start_pos,
                brace_depth,
            } => self.scan_string_interpolation(start_pos, brace_depth),
            LexState::InSingleStringInterp {
                start_pos,
                brace_depth,
            } => self.scan_string_interpolation(start_pos, brace_depth),
            LexState::InShellCommand { start_pos } => self.scan_shell_content(start_pos),
            LexState::InShellInterp {
                start_pos,
                brace_depth,
            } => self.scan_shell_interpolation(start_pos, brace_depth),
            LexState::InRegex { start_pos } => self.scan_regex_content(start_pos),
            LexState::StringContentReturned { start_pos } => {
                self.handle_content_returned_state(Token::StringEnd, start_pos)
            }
            LexState::SingleStringContentReturned { start_pos } => {
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
    ) -> Result<TokenWithSpan, LexError> {
        self.state = LexState::Normal;
        self.context.prev_token = Some(token.clone());
        let span = Span::new(
            start_pos,
            self.context.position,
            self.context.line,
            self.context.column,
        );
        Ok(TokenWithSpan::new(token, span))
    }

    fn scan_normal_token(&mut self) -> Result<TokenWithSpan, LexError> {
        NormalScanner::scan_token(&mut self.context, &mut self.state)
    }

    fn scan_string_content(&mut self, start_pos: usize) -> Result<TokenWithSpan, LexError> {
        StringScanner::scan_content(&mut self.context, &mut self.state, start_pos)
    }

    fn scan_single_string_content(&mut self, start_pos: usize) -> Result<TokenWithSpan, LexError> {
        StringScanner::scan_content_with_quote(&mut self.context, &mut self.state, start_pos, '\'')
    }

    fn scan_string_interpolation(
        &mut self,
        start_pos: usize,
        brace_depth: usize,
    ) -> Result<TokenWithSpan, LexError> {
        InterpolationScanner::scan_string_interpolation(
            &mut self.context,
            &mut self.state,
            start_pos,
            brace_depth,
        )
    }

    fn scan_shell_content(&mut self, start_pos: usize) -> Result<TokenWithSpan, LexError> {
        ShellScanner::scan_content(&mut self.context, &mut self.state, start_pos)
    }

    fn scan_shell_interpolation(
        &mut self,
        start_pos: usize,
        brace_depth: usize,
    ) -> Result<TokenWithSpan, LexError> {
        InterpolationScanner::scan_shell_interpolation(
            &mut self.context,
            &mut self.state,
            start_pos,
            brace_depth,
        )
    }

    fn scan_regex_content(&mut self, start_pos: usize) -> Result<TokenWithSpan, LexError> {
        RegexScanner::scan_content(&mut self.context, &mut self.state, start_pos)
    }
}
