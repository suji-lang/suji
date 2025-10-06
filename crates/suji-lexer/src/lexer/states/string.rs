use super::super::utils::LexerUtils;
// use super::context::ParentInterpolation; // no longer needed after helper consolidation
use super::{LexError, LexState, QuoteType, ScannerContext, ScannerResult};
use crate::token::{Span, Token, TokenWithSpan};

/// Scanner for string literal state
pub struct StringScanner;

impl StringScanner {
    /// Scan string content with a specific quote character and multiline flag
    pub fn scan_content(
        context: &mut ScannerContext,
        state: &mut LexState,
        start_pos: usize,
        quote_char: char,
        multiline: bool,
    ) -> ScannerResult {
        let mut content = String::new();
        let start_line = context.line;
        let start_column = context.column;

        while !context.is_at_end() {
            let ch = context.peek();

            // Check for end of string
            let is_end = if multiline {
                // We need to see three consecutive quote bytes starting at current position
                LexerUtils::detect_triple_quote(context, context.position, quote_char)
            } else {
                // Single-quote end: compare the current char only
                ch == quote_char
            };

            if is_end {
                // End of string
                if multiline {
                    // Consume triple quotes (ASCII quotes: 3 single-byte advances)
                    context.advance();
                    context.advance();
                    context.advance();
                } else {
                    // Consume single quote
                    context.advance();
                }

                if !content.is_empty() {
                    // Return accumulated content first, then we'll be called again for StringEnd
                    // Don't pop from stack yet - let handle_content_returned_state do it
                    let quote_type = if quote_char == '"' {
                        QuoteType::Double
                    } else {
                        QuoteType::Single
                    };
                    *state = LexState::StringContentReturned {
                        start_pos,
                        quote_type,
                        multiline,
                    };
                    let span_offset = if multiline { 3 } else { 1 };
                    let span = Span::new(
                        start_pos,
                        context.position - span_offset,
                        start_line,
                        start_column,
                    );
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Empty string content, return StringEnd directly via helper
                    let token = context.finalize_end_token(
                        state,
                        start_pos,
                        start_line,
                        start_column,
                        Token::StringEnd,
                    );
                    return Ok(token);
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
                    let quote_type = if quote_char == '"' {
                        QuoteType::Double
                    } else {
                        QuoteType::Single
                    };
                    *state = LexState::InStringInterp {
                        start_pos,
                        quote_type,
                        multiline,
                        brace_depth: 1,
                    };
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
            }
        }

        Err(LexError::UnterminatedString {
            line: context.line,
            column: context.column,
        })
    }
}
