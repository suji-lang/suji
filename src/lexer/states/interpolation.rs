use super::normal::NormalScanner;
use super::{LexState, ScannerContext, ScannerResult};
use crate::token::{Span, Token, TokenWithSpan};

/// Scanner for interpolation states (both string and shell)
pub struct InterpolationScanner;

impl InterpolationScanner {
    /// Scan string interpolation content
    pub fn scan_string_interpolation(
        context: &mut ScannerContext,
        state: &mut LexState,
        start_pos: usize,
        mut brace_depth: usize,
    ) -> ScannerResult {
        // Extract state information
        let (quote_type, multiline) = match state {
            LexState::InStringInterp {
                quote_type,
                multiline,
                ..
            } => (*quote_type, *multiline),
            _ => {
                return Err(super::LexError::UnexpectedCharacter {
                    ch: ' ',
                    line: context.line,
                    column: context.column,
                });
            }
        };
        // Scan normal tokens until we hit the closing brace
        let token_result = NormalScanner::scan_token(context, state);

        match &token_result {
            Ok(token_with_span) => {
                match &token_with_span.token {
                    Token::LeftBrace => {
                        brace_depth += 1;
                        *state = LexState::InStringInterp {
                            start_pos,
                            quote_type,
                            multiline,
                            brace_depth,
                        };
                    }
                    Token::RightBrace => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // End interpolation, return to string parsing
                            *state = LexState::InString {
                                start_pos,
                                quote_type,
                                multiline,
                            };
                            let span = Span::new(
                                start_pos,
                                context.position,
                                context.line,
                                context.column,
                            );
                            context.prev_token = Some(Token::InterpEnd);
                            return Ok(TokenWithSpan::new(Token::InterpEnd, span));
                        } else {
                            // Keep interpolation state
                            *state = LexState::InStringInterp {
                                start_pos,
                                quote_type,
                                multiline,
                                brace_depth,
                            };
                        }
                    }
                    _ => {
                        // Keep interpolation state
                        *state = LexState::InStringInterp {
                            start_pos,
                            quote_type,
                            multiline,
                            brace_depth,
                        };
                    }
                }
            }
            Err(_) => {
                return token_result;
            }
        }

        token_result
    }

    /// Scan shell interpolation content
    pub fn scan_shell_interpolation(
        context: &mut ScannerContext,
        state: &mut LexState,
        start_pos: usize,
        mut brace_depth: usize,
    ) -> ScannerResult {
        // Same as string interpolation
        let token_result = NormalScanner::scan_token(context, state);

        match &token_result {
            Ok(token_with_span) => {
                match &token_with_span.token {
                    Token::LeftBrace => {
                        brace_depth += 1;
                        *state = LexState::InShellInterp {
                            start_pos,
                            brace_depth,
                        };
                    }
                    Token::RightBrace => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // End interpolation, return to shell parsing
                            *state = LexState::InShellCommand { start_pos };
                            let span = Span::new(
                                start_pos,
                                context.position,
                                context.line,
                                context.column,
                            );
                            context.prev_token = Some(Token::InterpEnd);
                            return Ok(TokenWithSpan::new(Token::InterpEnd, span));
                        } else {
                            *state = LexState::InShellInterp {
                                start_pos,
                                brace_depth,
                            };
                        }
                    }
                    _ => {
                        *state = LexState::InShellInterp {
                            start_pos,
                            brace_depth,
                        };
                    }
                }
            }
            Err(_) => {
                return token_result;
            }
        }

        token_result
    }
}
