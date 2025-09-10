use crate::token::{Span, Token, TokenWithSpan};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum LexError {
    #[error("Unterminated string literal at line {line}, column {column}")]
    UnterminatedString { line: usize, column: usize },
    #[error("Unterminated shell command at line {line}, column {column}")]
    UnterminatedShellCommand { line: usize, column: usize },
    #[error("Unterminated regex literal at line {line}, column {column}")]
    UnterminatedRegex { line: usize, column: usize },
    #[error("Invalid escape sequence '\\{escape}' at line {line}, column {column}")]
    InvalidEscape {
        escape: char,
        line: usize,
        column: usize,
    },
    #[error("Invalid number literal '{literal}' at line {line}, column {column}")]
    InvalidNumber {
        literal: String,
        line: usize,
        column: usize,
    },
    #[error("Unexpected character '{ch}' at line {line}, column {column}")]
    UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone)]
enum LexState {
    Normal,
    InString {
        start_pos: usize,
    },
    InStringInterp {
        start_pos: usize,
        brace_depth: usize,
    },
    InShellCommand {
        start_pos: usize,
    },
    InShellInterp {
        start_pos: usize,
        brace_depth: usize,
    },
    InRegex {
        start_pos: usize,
    },
    StringContentReturned {
        start_pos: usize,
    },
    RegexContentReturned {
        start_pos: usize,
    },
    ShellContentReturned {
        start_pos: usize,
    },
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
    state: LexState,
    prev_token: Option<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            state: LexState::Normal,
            prev_token: None,
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
            LexState::InStringInterp {
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
                self.state = LexState::Normal;
                let token = Token::StringEnd;
                self.prev_token = Some(token.clone());
                let span = Span::new(start_pos, self.position, self.line, self.column);
                Ok(TokenWithSpan::new(token, span))
            }
            LexState::RegexContentReturned { start_pos } => {
                self.state = LexState::Normal;
                let token = Token::RegexEnd;
                self.prev_token = Some(token.clone());
                let span = Span::new(start_pos, self.position, self.line, self.column);
                Ok(TokenWithSpan::new(token, span))
            }
            LexState::ShellContentReturned { start_pos } => {
                self.state = LexState::Normal;
                let token = Token::ShellEnd;
                self.prev_token = Some(token.clone());
                let span = Span::new(start_pos, self.position, self.line, self.column);
                Ok(TokenWithSpan::new(token, span))
            }
        }
    }

    fn scan_normal_token(&mut self) -> Result<TokenWithSpan, LexError> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(self.make_token(Token::Eof));
        }

        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.advance();

        let token = match ch {
            '"' => {
                self.state = LexState::InString { start_pos };
                Token::StringStart
            }
            '`' => {
                self.state = LexState::InShellCommand { start_pos };
                Token::ShellStart
            }
            '/' => {
                // Check for compound assignment first
                if self.match_char('=') {
                    Token::DivideAssign
                } else if self.should_parse_as_regex() {
                    // Regex vs division disambiguation
                    self.state = LexState::InRegex { start_pos };
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
                if self.match_char('+') {
                    Token::Increment
                } else if self.match_char('=') {
                    Token::PlusAssign
                } else {
                    Token::Plus
                }
            }
            '-' => {
                if self.match_char('-') {
                    Token::Decrement
                } else if self.match_char('=') {
                    Token::MinusAssign
                } else {
                    Token::Minus
                }
            }
            '*' => {
                if self.match_char('=') {
                    Token::MultiplyAssign
                } else {
                    Token::Multiply
                }
            }
            '%' => {
                if self.match_char('=') {
                    Token::ModuloAssign
                } else {
                    Token::Modulo
                }
            }
            '^' => Token::Power,
            '=' => {
                if self.match_char('=') {
                    Token::Equal
                } else {
                    Token::Assign
                }
            }
            '!' => {
                if self.match_char('=') {
                    Token::NotEqual
                } else if self.match_char('~') {
                    Token::RegexNotMatch
                } else {
                    Token::Not
                }
            }
            '<' => {
                if self.match_char('=') {
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '&' => {
                if self.match_char('&') {
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
                if self.match_char('|') {
                    Token::Or
                } else {
                    Token::Pipe
                }
            }
            ':' => {
                if self.match_char(':') {
                    Token::DoubleColon
                } else {
                    Token::Colon
                }
            }
            '.' => {
                if self.match_char('.') {
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
            '#' => self.scan_comment(),
            '\n' => {
                self.line += 1;
                self.column = 1;
                Token::Newline
            }
            _ if ch.is_ascii_digit() => self.scan_number(start_pos)?,
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                self.scan_identifier_or_keyword(start_pos)
            }
            _ => {
                return Err(LexError::UnexpectedCharacter {
                    ch,
                    line: start_line,
                    column: start_column,
                });
            }
        };

        let span = Span::new(start_pos, self.position, start_line, start_column);
        let token_with_span = TokenWithSpan::new(token.clone(), span);
        self.prev_token = Some(token);
        Ok(token_with_span)
    }

    fn scan_string_content(&mut self, start_pos: usize) -> Result<TokenWithSpan, LexError> {
        let mut content = String::new();
        let start_line = self.line;
        let start_column = self.column;

        while !self.is_at_end() {
            let ch = self.peek();

            if ch == '"' {
                // End of string
                self.advance(); // consume closing quote
                self.state = LexState::Normal;

                if !content.is_empty() {
                    // Return accumulated content first, then we'll be called again for StringEnd
                    self.state = LexState::StringContentReturned { start_pos };
                    let span = Span::new(start_pos, self.position - 1, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Empty string content, return StringEnd directly
                    self.state = LexState::Normal;
                    let span = Span::new(start_pos, self.position, start_line, start_column);
                    let token = Token::StringEnd;
                    self.prev_token = Some(token.clone());
                    return Ok(TokenWithSpan::new(token, span));
                }
            } else if ch == '$' && self.peek_next() == Some('{') {
                // Start of interpolation
                if !content.is_empty() {
                    // Return accumulated string text first
                    let span = Span::new(start_pos, self.position, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Start interpolation
                    self.advance(); // consume $
                    self.advance(); // consume {
                    self.state = LexState::InStringInterp {
                        start_pos,
                        brace_depth: 1,
                    };
                    let span = Span::new(start_pos, self.position, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::InterpStart, span));
                }
            } else if ch == '\\' {
                // Handle escape sequences
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return Err(LexError::UnterminatedString {
                        line: self.line,
                        column: self.column,
                    });
                }
                let escaped = self.advance();
                let escaped_char = match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    '$' => '$',
                    _ => {
                        return Err(LexError::InvalidEscape {
                            escape: escaped,
                            line: self.line,
                            column: self.column - 1,
                        });
                    }
                };
                content.push(escaped_char);
            } else {
                content.push(self.advance());
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
            }
        }

        Err(LexError::UnterminatedString {
            line: self.line,
            column: self.column,
        })
    }

    fn scan_string_interpolation(
        &mut self,
        start_pos: usize,
        mut brace_depth: usize,
    ) -> Result<TokenWithSpan, LexError> {
        // Scan normal tokens until we hit the closing brace
        let token_result = self.scan_normal_token();

        match &token_result {
            Ok(token_with_span) => {
                match &token_with_span.token {
                    Token::LeftBrace => {
                        brace_depth += 1;
                        self.state = LexState::InStringInterp {
                            start_pos,
                            brace_depth,
                        };
                    }
                    Token::RightBrace => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // End interpolation, return to string parsing
                            self.state = LexState::InString { start_pos };
                            let span = Span::new(start_pos, self.position, self.line, self.column);
                            self.prev_token = Some(Token::InterpEnd);
                            return Ok(TokenWithSpan::new(Token::InterpEnd, span));
                        } else {
                            self.state = LexState::InStringInterp {
                                start_pos,
                                brace_depth,
                            };
                        }
                    }
                    _ => {
                        self.state = LexState::InStringInterp {
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

    fn scan_shell_content(&mut self, start_pos: usize) -> Result<TokenWithSpan, LexError> {
        let mut content = String::new();
        let start_line = self.line;
        let start_column = self.column;

        while !self.is_at_end() {
            let ch = self.peek();

            if ch == '`' {
                // End of shell command
                self.advance(); // consume closing backtick

                if !content.is_empty() {
                    // Return accumulated content first, then we'll be called again for ShellEnd
                    self.state = LexState::ShellContentReturned { start_pos };
                    let span = Span::new(start_pos, self.position - 1, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Empty shell command content, return ShellEnd directly
                    self.state = LexState::Normal;
                    let span = Span::new(start_pos, self.position, start_line, start_column);
                    let token = Token::ShellEnd;
                    self.prev_token = Some(token.clone());
                    return Ok(TokenWithSpan::new(token, span));
                }
            } else if ch == '$' && self.peek_next() == Some('{') {
                // Start of interpolation
                if !content.is_empty() {
                    // Return accumulated string text first
                    let span = Span::new(start_pos, self.position, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::StringText(content), span));
                } else {
                    // Start interpolation
                    self.advance(); // consume $
                    self.advance(); // consume {
                    self.state = LexState::InShellInterp {
                        start_pos,
                        brace_depth: 1,
                    };
                    let span = Span::new(start_pos, self.position, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::InterpStart, span));
                }
            } else if ch == '\\' {
                // Handle escape sequences (same as strings)
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return Err(LexError::UnterminatedShellCommand {
                        line: self.line,
                        column: self.column,
                    });
                }
                let escaped = self.advance();
                let escaped_char = match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '`' => '`',
                    '\\' => '\\',
                    '$' => '$',
                    _ => {
                        return Err(LexError::InvalidEscape {
                            escape: escaped,
                            line: self.line,
                            column: self.column - 1,
                        });
                    }
                };
                content.push(escaped_char);
            } else {
                content.push(self.advance());
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
            }
        }

        Err(LexError::UnterminatedShellCommand {
            line: self.line,
            column: self.column,
        })
    }

    fn scan_shell_interpolation(
        &mut self,
        start_pos: usize,
        mut brace_depth: usize,
    ) -> Result<TokenWithSpan, LexError> {
        // Same as string interpolation
        let token_result = self.scan_normal_token();

        match &token_result {
            Ok(token_with_span) => {
                match &token_with_span.token {
                    Token::LeftBrace => {
                        brace_depth += 1;
                        self.state = LexState::InShellInterp {
                            start_pos,
                            brace_depth,
                        };
                    }
                    Token::RightBrace => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // End interpolation, return to shell parsing
                            self.state = LexState::InShellCommand { start_pos };
                            let span = Span::new(start_pos, self.position, self.line, self.column);
                            self.prev_token = Some(Token::InterpEnd);
                            return Ok(TokenWithSpan::new(Token::InterpEnd, span));
                        } else {
                            self.state = LexState::InShellInterp {
                                start_pos,
                                brace_depth,
                            };
                        }
                    }
                    _ => {
                        self.state = LexState::InShellInterp {
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

    fn scan_regex_content(&mut self, start_pos: usize) -> Result<TokenWithSpan, LexError> {
        let mut content = String::new();
        let start_line = self.line;
        let start_column = self.column;

        while !self.is_at_end() {
            let ch = self.peek();

            if ch == '/' {
                // End of regex
                self.advance(); // consume closing slash
                self.state = LexState::Normal;

                if !content.is_empty() {
                    // Return regex content first, then we'll be called again for RegexEnd
                    self.state = LexState::RegexContentReturned { start_pos };
                    let span = Span::new(start_pos, self.position - 1, start_line, start_column);
                    return Ok(TokenWithSpan::new(Token::RegexContent(content), span));
                } else {
                    // Empty regex, return end token
                    self.state = LexState::Normal;
                    let span = Span::new(start_pos, self.position, start_line, start_column);
                    let token = Token::RegexEnd;
                    self.prev_token = Some(token.clone());
                    return Ok(TokenWithSpan::new(token, span));
                }
            } else if ch == '\\' {
                // Handle escape sequences in regex
                content.push(self.advance()); // add backslash
                if !self.is_at_end() {
                    content.push(self.advance()); // add escaped character
                }
            } else {
                content.push(self.advance());
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
            }
        }

        Err(LexError::UnterminatedRegex {
            line: self.line,
            column: self.column,
        })
    }

    fn should_parse_as_regex(&self) -> bool {
        // If no previous token or previous token cannot end an expression, treat as regex
        match &self.prev_token {
            None => true,
            Some(token) => !token.can_end_expression(),
        }
    }

    fn scan_comment(&mut self) -> Token {
        let mut content = String::new();
        content.push('#'); // include the # in the comment

        while !self.is_at_end() && self.peek() != '\n' {
            content.push(self.advance());
        }

        Token::Comment(content)
    }

    fn scan_number(&mut self, start_pos: usize) -> Result<Token, LexError> {
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for decimal point
        if !self.is_at_end()
            && self.peek() == '.'
            && self.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            self.advance(); // consume the '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let literal = &self.input[start_pos..self.position];
        match literal.parse::<f64>() {
            Ok(value) => Ok(Token::Number(value)),
            Err(_) => Err(LexError::InvalidNumber {
                literal: literal.to_string(),
                line: self.line,
                column: self.column,
            }),
        }
    }

    fn scan_identifier_or_keyword(&mut self, start_pos: usize) -> Token {
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.input[start_pos..self.position];

        // Check if it's a standalone underscore (wildcard)
        if text == "_" {
            return Token::Underscore;
        }

        // Check if it's a keyword
        match text {
            "return" => Token::Return,
            "loop" => Token::Loop,
            "as" => Token::As,
            "through" => Token::Through,
            "with" => Token::With,
            "continue" => Token::Continue,
            "break" => Token::Break,
            "match" => Token::Match,
            "import" => Token::Import,
            "export" => Token::Export,
            "true" => Token::True,
            "false" => Token::False,
            "nil" => Token::Nil,
            _ => Token::Identifier(text.to_string()),
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.input.chars().nth(self.position).unwrap_or('\0');
        self.position += ch.len_utf8();
        if ch != '\n' {
            self.column += 1;
        }
        ch
    }

    fn peek(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn peek_next(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn make_token(&self, token: Token) -> TokenWithSpan {
        let span = Span::new(self.position, self.position, self.line, self.column);
        TokenWithSpan::new(token, span)
    }
}
