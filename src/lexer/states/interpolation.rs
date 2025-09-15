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
        // Determine if we're in single or double quote string interpolation
        let is_single_quote = matches!(state, LexState::InSingleStringInterp { .. });
        // Scan normal tokens until we hit the closing brace
        let token_result = NormalScanner::scan_token(context, state);

        match &token_result {
            Ok(token_with_span) => {
                match &token_with_span.token {
                    Token::LeftBrace => {
                        brace_depth += 1;
                        if is_single_quote {
                            *state = LexState::InSingleStringInterp {
                                start_pos,
                                brace_depth,
                            };
                        } else {
                            *state = LexState::InStringInterp {
                                start_pos,
                                brace_depth,
                            };
                        }
                    }
                    Token::RightBrace => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // End interpolation, return to string parsing
                            if is_single_quote {
                                *state = LexState::InSingleString { start_pos };
                            } else {
                                *state = LexState::InString { start_pos };
                            }
                            let span = Span::new(
                                start_pos,
                                context.position,
                                context.line,
                                context.column,
                            );
                            context.prev_token = Some(Token::InterpEnd);
                            return Ok(TokenWithSpan::new(Token::InterpEnd, span));
                        } else if is_single_quote {
                            *state = LexState::InSingleStringInterp {
                                start_pos,
                                brace_depth,
                            };
                        } else {
                            *state = LexState::InStringInterp {
                                start_pos,
                                brace_depth,
                            };
                        }
                    }
                    _ => {
                        if is_single_quote {
                            *state = LexState::InSingleStringInterp {
                                start_pos,
                                brace_depth,
                            };
                        } else {
                            *state = LexState::InStringInterp {
                                start_pos,
                                brace_depth,
                            };
                        }
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
