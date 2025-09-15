use super::super::utils::LexerUtils;
use super::{LexError, LexState, ScannerContext, ScannerResult};
use crate::token::{Span, Token, TokenWithSpan};

/// Scanner for string literal state
pub struct StringScanner;

impl StringScanner {
    /// Scan string content with a specific quote character
    pub fn scan_content_with_quote(
        context: &mut ScannerContext,
        state: &mut LexState,
        start_pos: usize,
        quote_char: char,
    ) -> ScannerResult {
        let mut content = String::new();
        let start_line = context.line;
        let start_column = context.column;

        while !context.is_at_end() {
            let ch = context.peek();

            if ch == quote_char {
                // End of string
                context.advance(); // consume closing quote
                *state = LexState::Normal;

                if !content.is_empty() {
                    // Return accumulated content first, then we'll be called again for StringEnd
                    if quote_char == '"' {
                        *state = LexState::StringContentReturned { start_pos };
                    } else {
                        *state = LexState::SingleStringContentReturned { start_pos };
                    }
                    let span = Span::new(start_pos, context.position - 1, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Empty string content, return StringEnd directly
                    *state = LexState::Normal;
                    let span = Span::new(start_pos, context.position, start_line, start_column);
                    let token = Token::StringEnd;
                    context.prev_token = Some(token.clone());
                    return Ok(TokenWithSpan::new(token, span));
                }
            } else if ch == '$' && context.peek_next() == Some('{') {
                // Start of interpolation
                if !content.is_empty() {
                    // Return accumulated string text first
                    let span = Span::new(start_pos, context.position, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Start interpolation
                    context.advance(); // consume $
                    context.advance(); // consume {
                    if quote_char == '"' {
                        *state = LexState::InStringInterp {
                            start_pos,
                            brace_depth: 1,
                        };
                    } else {
                        *state = LexState::InSingleStringInterp {
                            start_pos,
                            brace_depth: 1,
                        };
                    }
                    let span = Span::new(start_pos, context.position, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::InterpStart, span));
                }
            } else if ch == '\\' {
                // Handle escape sequences
                let escaped_char = LexerUtils::handle_escape_sequence(
                    context,
                    &[quote_char, '\\', '$', 'n', 't', 'r'],
                )?;
                content.push(escaped_char);
            } else {
                content.push(context.advance());
                if ch == '\n' {
                    context.line += 1;
                    context.column = 1;
                }
            }
        }

        Err(LexError::UnterminatedString {
            line: context.line,
            column: context.column,
        })
    }

    /// Scan string content (backward compatibility for double quotes)
    pub fn scan_content(
        context: &mut ScannerContext,
        state: &mut LexState,
        start_pos: usize,
    ) -> ScannerResult {
        Self::scan_content_with_quote(context, state, start_pos, '"')
    }
}
