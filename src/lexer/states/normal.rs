use super::super::utils::LexerUtils;
use super::{LexError, LexState, QuoteType, ScannerContext, ScannerResult};
use crate::token::{Span, Token, TokenWithSpan};

/// Scanner for normal token state
pub struct NormalScanner;

impl NormalScanner {
    /// Scan a token in normal state
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
            '"' => {
                // Check for triple quotes using byte slicing (quotes are ASCII)
                let bytes = context.input.as_bytes();
                let is_triple_quote = start_pos + 2 < bytes.len()
                    && bytes[start_pos] == b'"'
                    && bytes[start_pos + 1] == b'"'
                    && bytes[start_pos + 2] == b'"';
                if is_triple_quote {
                    // Triple double quotes - multiline string
                    context.advance(); // consume second "
                    context.advance(); // consume third "
                    // First " was already consumed by context.advance() on line 20
                    *state = LexState::InString {
                        start_pos,
                        quote_type: QuoteType::Double,
                        multiline: true,
                    };
                    Token::StringStart // Reuse existing token
                } else {
                    // Single double quote - regular string
                    *state = LexState::InString {
                        start_pos,
                        quote_type: QuoteType::Double,
                        multiline: false,
                    };
                    Token::StringStart
                }
            }
            '\'' => {
                // Check for triple quotes using byte slicing (quotes are ASCII)
                let bytes = context.input.as_bytes();
                let is_triple_quote = start_pos + 2 < bytes.len()
                    && bytes[start_pos] == b'\''
                    && bytes[start_pos + 1] == b'\''
                    && bytes[start_pos + 2] == b'\'';
                if is_triple_quote {
                    // Triple single quotes - multiline string
                    context.advance(); // consume second '
                    context.advance(); // consume third '
                    // First ' was already consumed by context.advance() on line 20
                    *state = LexState::InString {
                        start_pos,
                        quote_type: QuoteType::Single,
                        multiline: true,
                    };
                    Token::StringStart // Reuse existing token
                } else {
                    // Single single quote - regular string
                    *state = LexState::InString {
                        start_pos,
                        quote_type: QuoteType::Single,
                        multiline: false,
                    };
                    Token::StringStart
                }
            }
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
                } else {
                    Token::Less
                }
            }
            '>' => {
                if context.match_char('=') {
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
                        line: start_line,
                        column: start_column,
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
                    Token::Range
                } else {
                    return Err(LexError::UnexpectedCharacter {
                        ch,
                        line: start_line,
                        column: start_column,
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
            _ if ch.is_ascii_digit() => LexerUtils::scan_number(context, start_pos)?,
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                LexerUtils::scan_identifier_or_keyword(context, start_pos)
            }
            _ => {
                return Err(LexError::UnexpectedCharacter {
                    ch,
                    line: start_line,
                    column: start_column,
                });
            }
        };

        let span = Span::new(start_pos, context.position, start_line, start_column);
        let token_with_span = TokenWithSpan::new(token.clone(), span);
        context.prev_token = Some(token);
        Ok(token_with_span)
    }
}
