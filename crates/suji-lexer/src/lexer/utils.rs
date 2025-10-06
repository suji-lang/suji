use super::states::{LexError, ScannerContext};
use crate::token::Token;

/// Common scanning utilities for the lexer
pub struct LexerUtils;

impl LexerUtils {
    /// Detect if there is a triple-quote starting at `start_pos` for the given ASCII `quote_char`.
    /// This uses byte-based checks under the assumption that quote characters are ASCII.
    pub fn detect_triple_quote(
        context: &ScannerContext,
        start_pos: usize,
        quote_char: char,
    ) -> bool {
        let bytes = context.input.as_bytes();
        if start_pos + 2 >= bytes.len() {
            return false;
        }
        let q = quote_char as u8; // safe: ASCII quotes only
        bytes[start_pos] == q && bytes[start_pos + 1] == q && bytes[start_pos + 2] == q
    }
    /// Skip whitespace characters (but not newlines)
    pub fn skip_whitespace(context: &mut ScannerContext) {
        while !context.is_at_end() {
            let ch = context.peek();
            if ch.is_ascii_whitespace() && ch != '\n' {
                context.advance();
            } else {
                break;
            }
        }
    }

    /// Scan a comment starting with '#'
    pub fn scan_comment(context: &mut ScannerContext) -> Token {
        let mut content = String::new();
        content.push('#'); // include the # in the comment

        while !context.is_at_end() && context.peek() != '\n' {
            content.push(context.advance());
        }

        Token::Comment(content)
    }

    /// Scan a number literal
    pub fn scan_number(context: &mut ScannerContext, start_pos: usize) -> Result<Token, LexError> {
        while !context.is_at_end() && context.peek().is_ascii_digit() {
            context.advance();
        }

        // Look for decimal point
        if !context.is_at_end()
            && context.peek() == '.'
            && context.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            context.advance(); // consume the '.'
            while !context.is_at_end() && context.peek().is_ascii_digit() {
                context.advance();
            }
        }

        let literal = &context.input[start_pos..context.position];
        // Validate that it's a valid number format, but return as string
        match literal.parse::<f64>() {
            Ok(_) => Ok(Token::Number(literal.to_string())),
            Err(_) => Err(LexError::InvalidNumber {
                literal: literal.to_string(),
                line: context.line,
                column: context.column,
            }),
        }
    }

    /// Scan an identifier or keyword
    pub fn scan_identifier_or_keyword(context: &mut ScannerContext, start_pos: usize) -> Token {
        while !context.is_at_end() {
            let ch = context.peek();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                context.advance();
            } else {
                break;
            }
        }

        let text = &context.input[start_pos..context.position];

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

    /// Handle escape sequences in strings and shell commands
    pub fn handle_escape_sequence(
        context: &mut ScannerContext,
        allowed_escapes: &[char],
    ) -> Result<char, LexError> {
        context.advance(); // consume backslash
        if context.is_at_end() {
            return Err(LexError::UnterminatedString {
                line: context.line,
                column: context.column,
            });
        }
        let escaped = context.advance();
        let escaped_char = match escaped {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '"' => '"',
            '\'' => '\'',
            '`' => '`',
            '\\' => '\\',
            '$' => '$',
            _ => {
                if allowed_escapes.contains(&escaped) {
                    escaped
                } else {
                    return Err(LexError::InvalidEscape {
                        escape: escaped,
                        line: context.line,
                        column: context.column - 1,
                    });
                }
            }
        };
        Ok(escaped_char)
    }

    /// Handle escape sequences in regex (different rules)
    pub fn handle_regex_escape_sequence(context: &mut ScannerContext) -> String {
        let mut result = String::new();
        result.push(context.advance()); // add backslash
        if !context.is_at_end() {
            result.push(context.advance()); // add escaped character
        }
        result
    }
}
