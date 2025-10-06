use super::super::utils::LexerUtils;
use super::{LexError, LexState, ScannerContext, ScannerResult};
use crate::token::{Span, Token, TokenWithSpan};

/// Scanner for regex literal state
pub struct RegexScanner;

impl RegexScanner {
    /// Scan regex content
    pub fn scan_content(
        context: &mut ScannerContext,
        state: &mut LexState,
        start_pos: usize,
    ) -> ScannerResult {
        let mut content = String::new();
        let start_line = context.line;
        let start_column = context.column;

        while !context.is_at_end() {
            let ch = context.peek();

            if ch == '/' {
                // End of regex
                context.advance(); // consume closing slash
                if !content.is_empty() {
                    // Return regex content first, then we'll be called again for RegexEnd
                    *state = LexState::RegexContentReturned { start_pos };
                    let span = Span::new(start_pos, context.position - 1, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::RegexContent(content), span));
                } else {
                    // Empty regex, return end token via helper
                    let token = context.finalize_end_token(
                        state,
                        start_pos,
                        start_line,
                        start_column,
                        Token::RegexEnd,
                    );
                    return Ok(token);
                }
            } else if ch == '\\' {
                // Handle escape sequences in regex
                content.push_str(&LexerUtils::handle_regex_escape_sequence(context));
            } else {
                content.push(context.advance());
            }
        }

        Err(LexError::UnterminatedRegex {
            line: context.line,
            column: context.column,
        })
    }
}
