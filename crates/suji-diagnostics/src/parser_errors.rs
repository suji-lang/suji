use super::error_codes::*;
use super::error_template::ErrorTemplate;
use suji_lexer::Token;

// Parser error templates
pub fn unexpected_token() -> ErrorTemplate {
    // Base template with no suggestions; suggestions are composed per token
    ErrorTemplate::new(
        PARSE_UNEXPECTED_TOKEN,
        "Unexpected token",
        "Unexpected token",
    )
}

/// Provide a suggestion message tailored to a specific unexpected token.
/// Returns a single suggestion string matching previous behavior.
pub fn unexpected_token_suggestion(token: &Token) -> String {
    match token {
        Token::Identifier(name) => format!(
            "Did you mean to use '{}' as a variable name? Variables must be assigned before use",
            name
        ),
        Token::Number(_) => {
            "This number is not valid in this context. Check for missing operators or syntax"
                .to_string()
        }
        Token::StringStart => {
            "Did you mean to start a string literal? Use quotes: \"text\"".to_string()
        }
        Token::ShellStart => {
            "Did you mean to start a shell command? Use backticks: `command`".to_string()
        }
        Token::RegexStart => {
            "Did you mean to start a regex literal? Use slashes: /pattern/".to_string()
        }
        Token::LeftBrace => "Did you mean to start a block or map? Use braces: { }".to_string(),
        Token::RightBrace => {
            "Did you mean to close a block or map? Check for matching opening brace".to_string()
        }
        Token::LeftBracket => "Did you mean to start a list? Use brackets: [ ]".to_string(),
        Token::RightBracket => {
            "Did you mean to close a list? Check for matching opening bracket".to_string()
        }
        Token::LeftParen => "Did you mean to start a grouping? Use parentheses: ( )".to_string(),
        Token::RightParen => {
            "Did you mean to close a grouping? Check for matching opening parenthesis".to_string()
        }
        Token::Comma => {
            "Comma is not expected here. Check for missing values or extra commas".to_string()
        }
        Token::Assign => {
            "Assignment operator '=' is not expected here. Check for missing variable name"
                .to_string()
        }
        Token::Equal => {
            "Comparison operator '==' is not expected here. Check for missing left operand"
                .to_string()
        }
        _ => "This token is not expected here. Check for syntax errors or missing elements"
            .to_string(),
    }
}

pub fn unexpected_eof() -> ErrorTemplate {
    ErrorTemplate::new(
        PARSE_UNEXPECTED_EOF,
        "Unexpected end of input",
        "Unexpected end of input",
    )
    .with_suggestion(
        "The input ended unexpectedly. Check for missing closing brackets, quotes, or other syntax",
    )
}

pub fn generic_parse_error(message: &str) -> ErrorTemplate {
    ErrorTemplate::new(PARSE_GENERIC_ERROR, "Parse error", message)
}

pub fn multiple_exports() -> ErrorTemplate {
    ErrorTemplate::new(
        PARSE_MULTIPLE_EXPORTS,
        "Multiple export statements found",
        "Multiple export statements found",
    )
    .with_suggestion("Only one export statement is allowed per file")
}

pub fn expected_token(expected: &Token, found: &Token) -> ErrorTemplate {
    let message = format!("Expected {:?}, found {:?}", expected, found);
    ErrorTemplate::new(PARSE_EXPECTED_TOKEN, "Expected token", &message)
}

pub fn invalid_import_path() -> ErrorTemplate {
    ErrorTemplate::new(
        PARSE_INVALID_IMPORT_PATH,
        "Expected item name after ':'",
        "Expected item name after ':'",
    )
}

pub fn invalid_alias() -> ErrorTemplate {
    ErrorTemplate::new(
        PARSE_INVALID_ALIAS,
        "Expected alias name after 'as'",
        "Expected alias name after 'as'",
    )
}
