use super::error_codes::*;
use super::error_template::ErrorTemplate;

// Lexer error templates
pub fn unterminated_string() -> ErrorTemplate {
    ErrorTemplate::new(
        LEX_UNTERMINATED_STRING,
        "Unterminated string literal",
        "Unterminated string literal",
    )
    .with_suggestion("Add a closing quote (\") to terminate the string")
}

pub fn unterminated_shell_command() -> ErrorTemplate {
    ErrorTemplate::new(
        LEX_UNTERMINATED_SHELL_COMMAND,
        "Unterminated shell command",
        "Unterminated shell command",
    )
    .with_suggestion("Add a closing backtick (`) to terminate the shell command")
}

pub fn unterminated_regex() -> ErrorTemplate {
    ErrorTemplate::new(
        LEX_UNTERMINATED_REGEX,
        "Unterminated regex literal",
        "Unterminated regex literal",
    )
    .with_suggestion("Add a closing slash (/) to terminate the regex")
}

pub fn invalid_escape(escape: char) -> ErrorTemplate {
    let suggestion = match escape {
        'n' => "Use \\n for newline",
        't' => "Use \\t for tab",
        '"' => "Use \\\" for quote",
        '\\' => "Use \\\\ for backslash",
        _ => "Valid escape sequences are: \\n, \\t, \\\", \\\\",
    };
    ErrorTemplate::new(
        LEX_INVALID_ESCAPE,
        "Invalid escape sequence",
        &format!("Invalid escape sequence '\\{}'", escape),
    )
    .with_suggestion(suggestion)
}

pub fn invalid_number(literal: &str) -> ErrorTemplate {
    ErrorTemplate::new(
        LEX_INVALID_NUMBER,
        "Invalid number literal",
        &format!("Invalid number literal '{}'", literal),
    )
    .with_suggestion("Numbers can be integers (42) or decimals (3.14)")
}

pub fn unexpected_character(ch: char) -> ErrorTemplate {
    let suggestion = match ch {
        '`' => "Did you mean to start a shell command? Use backticks: `command`",
        '/' => "Did you mean to start a regex literal? Use slashes: /pattern/",
        '"' => "Did you mean to start a string literal? Use quotes: \"text\"",
        '{' => "Did you mean to start a block or map? Use braces: { key: value }",
        '}' => "Did you mean to close a block or map? Check for matching opening brace",
        '[' => "Did you mean to start a list? Use brackets: [1, 2, 3]",
        ']' => "Did you mean to close a list? Check for matching opening bracket",
        '(' => "Did you mean to start a grouping? Use parentheses: (expression)",
        ')' => "Did you mean to close a grouping? Check for matching opening parenthesis",
        '@' => "This character is not used in SUJI language. Did you mean something else?",
        '#' => "Comments start with # and continue to the end of the line",
        _ => "This character is not valid in this context. Check for typos or missing quotes",
    };
    ErrorTemplate::new(
        LEX_UNEXPECTED_CHARACTER,
        "Unexpected character",
        &format!("Unexpected character '{}'", ch),
    )
    .with_suggestion(suggestion)
}
