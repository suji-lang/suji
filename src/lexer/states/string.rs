use super::super::utils::LexerUtils;
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
                // For multiline strings, check for triple quotes
                ch == quote_char
                    && context.peek_next() == Some(quote_char)
                    && context.peek_next_next() == Some(quote_char)
            } else {
                // For regular strings, check for single quote
                ch == quote_char
            };

            if is_end {
                // End of string
                if multiline {
                    // Consume triple quotes
                    context.advance(); // consume first quote
                    context.advance(); // consume second quote
                    context.advance(); // consume third quote
                } else {
                    // Consume single quote
                    context.advance();
                }
                *state = LexState::Normal;

                if !content.is_empty() {
                    // Return accumulated content first, then we'll be called again for StringEnd
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
}
