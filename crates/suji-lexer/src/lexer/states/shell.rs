use super::super::utils::LexerUtils;
use super::{LexError, LexState, ScannerContext, ScannerResult};
use crate::token::{Span, Token, TokenWithSpan};

/// Scanner for shell command state
pub struct ShellScanner;

impl ShellScanner {
    /// Scan shell command content
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

            if ch == '`' {
                // End of shell command
                context.advance(); // consume closing backtick

                if !content.is_empty() {
                    // Return accumulated content first, then we'll be called again for ShellEnd
                    *state = LexState::ShellContentReturned { start_pos };
                    let span = Span::new(start_pos, context.position - 1, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Empty shell command content, return ShellEnd directly via helper
                    let token = context.finalize_end_token(
                        state,
                        start_pos,
                        start_line,
                        start_column,
                        Token::ShellEnd,
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
                    *state = LexState::InShellInterp {
                        start_pos,
                        brace_depth: 1,
                    };
                    let span = Span::new(start_pos, context.position, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::InterpStart, span));
                }
            } else if ch == '\\' {
                // Handle escape sequences (same as strings)
                let escaped_char =
                    LexerUtils::handle_escape_sequence(context, &['`', '\\', '$', 'n', 't', 'r'])?;
                content.push(escaped_char);
            } else {
                content.push(context.advance());
            }
        }

        Err(LexError::UnterminatedShellCommand {
            line: context.line,
            column: context.column,
        })
    }
}
