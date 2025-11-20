use super::{LexState, QuoteType, ScannerContext, ScannerResult};
use crate::lexer::LexError;
use crate::span::Span;
use crate::token::{SpannedToken, Token};
use crate::utils::LexerUtils;

/// Scanner for normal token state
pub struct NormalScanner;

impl NormalScanner {
    /// Handle string quote (single or double) detection and state setup.
    fn handle_string_quote(
        context: &mut ScannerContext,
        state: &mut LexState,
        start_pos: usize,
        quote_char: char,
    ) -> Token {
        let is_triple_quote = LexerUtils::detect_triple_quote(context, start_pos, quote_char);
        let quote_type = if quote_char == '"' {
            QuoteType::Double
        } else {
            QuoteType::Single
        };

        if is_triple_quote {
            // Triple quotes - multiline string
            // First quote was already consumed by caller, consume remaining two
            context.advance(); // consume second quote
            context.advance(); // consume third quote
            *state = LexState::InString {
                start_pos,
                quote_type,
                multiline: true,
            };
        } else {
            // Single quote - regular string
            *state = LexState::InString {
                start_pos,
                quote_type,
                multiline: false,
            };
        }
        Token::StringStart
    }

    pub fn scan_token(context: &mut ScannerContext, state: &mut LexState) -> ScannerResult {
        LexerUtils::skip_whitespace(context);

        if context.is_at_end() {
            return Ok(context.make_token(Token::Eof));
        }

        let start_pos = context.position;
        let start_line = context.line;
        let start_column = context.column;
        let ch = context.advance();

        let token = match ch {
            '"' => Self::handle_string_quote(context, state, start_pos, '"'),
            '\'' => Self::handle_string_quote(context, state, start_pos, '\''),
            '`' => {
                *state = LexState::InShellCommand { start_pos };
                Token::ShellStart
            }
            '/' => {
                // Check for compound assignment first
                if context.match_char('=') {
                    Token::DivideAssign
                } else if context.should_parse_as_regex() {
                    // Regex vs division disambiguation
                    *state = LexState::InRegex { start_pos };
                    Token::RegexStart
                } else {
                    Token::Divide
                }
            }
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            ',' => Token::Comma,
            '+' => {
                if context.match_char('+') {
                    Token::Increment
                } else if context.match_char('=') {
                    Token::PlusAssign
                } else {
                    Token::Plus
                }
            }
            '-' => {
                if context.match_char('-') {
                    Token::Decrement
                } else if context.match_char('=') {
                    Token::MinusAssign
                } else {
                    Token::Minus
                }
            }
            '*' => {
                if context.match_char('=') {
                    Token::MultiplyAssign
                } else {
                    Token::Multiply
                }
            }
            '%' => {
                if context.match_char('=') {
                    Token::ModuloAssign
                } else {
                    Token::Modulo
                }
            }
            '^' => Token::Power,
            '=' => {
                if context.match_char('=') {
                    Token::Equal
                } else if context.match_char('>') {
                    Token::FatArrow
                } else {
                    Token::Assign
                }
            }
            '!' => {
                if context.match_char('=') {
                    Token::NotEqual
                } else if context.match_char('~') {
                    Token::RegexNotMatch
                } else {
                    Token::Not
                }
            }
            '<' => {
                if context.match_char('=') {
                    Token::LessEqual
                } else if context.match_char('|') {
                    Token::PipeBackward
                } else if context.match_char('<') {
                    Token::ComposeLeft
                } else {
                    Token::Less
                }
            }
            '>' => {
                if context.match_char('>') {
                    Token::ComposeRight
                } else if context.match_char('=') {
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '&' => {
                if context.match_char('&') {
                    Token::And
                } else {
                    return Err(LexError::UnexpectedCharacter {
                        ch,
                        span: Span::new(start_pos, context.position, start_line, start_column),
                    });
                }
            }
            '|' => {
                if context.match_char('|') {
                    Token::Or
                } else if context.match_char('>') {
                    Token::PipeForward
                } else {
                    Token::Pipe
                }
            }
            ':' => {
                if context.match_char(':') {
                    Token::DoubleColon
                } else {
                    Token::Colon
                }
            }
            '.' => {
                if context.match_char('.') {
                    if context.match_char('=') {
                        Token::RangeInclusive
                    } else {
                        Token::Range
                    }
                } else {
                    return Err(LexError::UnexpectedCharacter {
                        ch,
                        span: Span::new(start_pos, context.position, start_line, start_column),
                    });
                }
            }
            '~' => Token::RegexMatch,
            ';' => Token::Semicolon,
            '#' => LexerUtils::scan_comment(context),
            '\n' => {
                context.line += 1;
                context.column = 1;
                Token::Newline
            }
            _ if ch.is_ascii_digit() => {
                LexerUtils::scan_number(context, start_pos, start_line, start_column)?
            }
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                LexerUtils::scan_identifier_or_keyword(context, start_pos)
            }
            _ => {
                return Err(LexError::UnexpectedCharacter {
                    ch,
                    span: Span::new(start_pos, context.position, start_line, start_column),
                });
            }
        };

        let span = Span::new(start_pos, context.position, start_line, start_column);
        let token_with_span = SpannedToken::new(token.clone(), span);
        context.prev_token = Some(token);
        Ok(token_with_span)
    }
}
