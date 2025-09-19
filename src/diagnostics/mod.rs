use std::path::Path;

use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::runtime::value::RuntimeError;
pub mod error_builder;
pub mod templates;

use error_builder::{ErrorBuilder, line_column_to_range};
use templates::ErrorTemplate;
use templates::predefined as template_functions;

/// Context for diagnostic reporting, containing source code and file information
#[derive(Debug, Clone)]
pub struct DiagnosticContext {
    /// The source code content
    pub source: String,
    /// Optional filename for the source
    pub filename: Option<String>,
    /// Optional file ID for ariadne (defaults to filename or "input")
    pub file_id: String,
}

impl DiagnosticContext {
    /// Create a new diagnostic context from source code
    pub fn new(source: String) -> Self {
        Self {
            file_id: "input".to_string(),
            filename: None,
            source,
        }
    }

    /// Create a new diagnostic context from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path)?;
        let filename = path.to_string_lossy().to_string();

        Ok(Self {
            file_id: filename.clone(),
            filename: Some(filename),
            source,
        })
    }

    /// Create a diagnostic context with a custom file ID
    pub fn with_file_id(source: String, file_id: String) -> Self {
        Self {
            file_id,
            filename: None,
            source,
        }
    }
}

/// Different kinds of diagnostics that can be reported
#[derive(Debug)]
pub enum DiagnosticKind {
    Lex(LexError),
    Parse(ParseError),
    Runtime(RuntimeError),
}

/// Print a diagnostic with enhanced formatting using ariadne
pub fn print_diagnostic(
    kind: DiagnosticKind,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match kind {
        DiagnosticKind::Lex(error) => print_lex_error(error, context),
        DiagnosticKind::Parse(error) => print_parse_error(error, context),
        DiagnosticKind::Runtime(error) => print_runtime_error(error, context),
    }
    Ok(())
}

/// Print a lexer error using ariadne
fn print_lex_error(error: LexError, context: &DiagnosticContext) {
    match error {
        LexError::UnterminatedString { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = template_functions::unterminated_string();
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range(range);
        }
        LexError::UnterminatedShellCommand { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = template_functions::unterminated_shell_command();
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range(range);
        }
        LexError::UnterminatedRegex { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = template_functions::unterminated_regex();
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range(range);
        }
        LexError::InvalidEscape {
            escape,
            line,
            column,
        } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = template_functions::invalid_escape(escape);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range(range);
        }
        LexError::InvalidNumber {
            literal,
            line,
            column,
        } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = template_functions::invalid_number(&literal);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range(range);
        }
        LexError::UnexpectedCharacter { ch, line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = template_functions::unexpected_character(ch);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range(range);
        }
    }
}

/// Print a parser error using ariadne
fn print_parse_error(error: ParseError, context: &DiagnosticContext) {
    match error {
        ParseError::Lex(lex_error) => print_lex_error(lex_error, context),
        ParseError::UnexpectedToken { token, span } => {
            let template = match &token {
                crate::token::Token::Identifier(name) => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion(&format!("Did you mean to use '{}' as a variable name? Variables must be assigned before use", name))
                }
                crate::token::Token::Number(_) => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("This number is not valid in this context. Check for missing operators or syntax")
                }
                crate::token::Token::StringStart => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to start a string literal? Use quotes: \"text\"")
                }
                crate::token::Token::ShellStart => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to start a shell command? Use backticks: `command`")
                }
                crate::token::Token::RegexStart => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to start a regex literal? Use slashes: /pattern/")
                }
                crate::token::Token::LeftBrace => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to start a block or map? Use braces: { }")
                }
                crate::token::Token::RightBrace => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to close a block or map? Check for matching opening brace")
                }
                crate::token::Token::LeftBracket => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to start a list? Use brackets: [ ]")
                }
                crate::token::Token::RightBracket => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to close a list? Check for matching opening bracket")
                }
                crate::token::Token::LeftParen => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to start a grouping? Use parentheses: ( )")
                }
                crate::token::Token::RightParen => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Did you mean to close a grouping? Check for matching opening parenthesis")
                }
                crate::token::Token::Comma => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Comma is not expected here. Check for missing values or extra commas")
                }
                crate::token::Token::Assign => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Assignment operator '=' is not expected here. Check for missing variable name")
                }
                crate::token::Token::Equal => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("Comparison operator '==' is not expected here. Check for missing left operand")
                }
                _ => {
                    ErrorTemplate::new(2, "Unexpected token", "Unexpected token")
                        .with_suggestion("This token is not expected here. Check for syntax errors or missing elements")
                }
            };

            let _ = ErrorBuilder::new(template, context.clone()).print_with_span(span);
        }
        ParseError::UnexpectedEof => {
            let range = context.source.len()..context.source.len();
            let _ = ErrorBuilder::new(template_functions::unexpected_eof(), context.clone())
                .print_with_range_no_label(range);
        }
        ParseError::Generic { message } => {
            let template = template_functions::generic_parse_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        ParseError::MultipleExports { span } => {
            let _ = ErrorBuilder::new(template_functions::multiple_exports(), context.clone())
                .print_with_span(span);
        }
    }
}

/// Print a runtime error using ariadne
fn print_runtime_error(error: RuntimeError, context: &DiagnosticContext) {
    match error {
        RuntimeError::TypeError { message } => {
            let template = template_functions::type_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::UndefinedVariable { name } => {
            let suggestions = find_similar_variables(&name, &context.source);
            let template = if suggestions.is_empty() {
                template_functions::undefined_variable(&name)
            } else {
                ErrorTemplate::new(11, "Undefined variable", "Undefined variable").with_suggestion(
                    &format!(
                        "Variable '{}' is not defined. Did you mean: {}?",
                        name,
                        suggestions.join(", ")
                    ),
                )
            };

            // Try to find the variable usage in the source code
            if let Some(span) = find_variable_usage(&name, &context.source) {
                let _ = ErrorBuilder::new(template, context.clone()).print_with_range(span);
            } else {
                let _ =
                    ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
            }
        }
        RuntimeError::InvalidOperation { message } => {
            let template = template_functions::invalid_operation(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::IndexOutOfBounds { message } => {
            let template = template_functions::index_out_of_bounds(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::KeyNotFound { message } => {
            let template = template_functions::key_not_found(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::InvalidKeyType { message } => {
            let template = template_functions::invalid_key_type(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::ShellError { message } => {
            let template = template_functions::shell_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::RegexError { message } => {
            let template = template_functions::regex_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::ArityMismatch { message } => {
            let template = template_functions::arity_mismatch(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::MethodError { message } => {
            // Attempt to parse method/type from the message and route to specific helpers
            if let Some((value_type, method_name)) = parse_method_error(&message) {
                let template = match value_type {
                    "String" => template_functions::string_method_error(&method_name, &message),
                    "List" => template_functions::list_method_error(&method_name, &message),
                    "Number" => template_functions::number_method_error(&method_name, &message),
                    "Tuple" => template_functions::tuple_method_error(&method_name, &message),
                    "Stream" => template_functions::stream_method_error(&method_name, &message),
                    _ => template_functions::method_error(&message),
                };
                let _ =
                    ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
            } else {
                let template = template_functions::method_error(&message);
                let _ =
                    ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
            }
        }
        RuntimeError::InvalidNumberConversion { message } => {
            let template = template_functions::invalid_number_conversion(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::ControlFlow { flow } => {
            let template = template_functions::control_flow_error(&format!("{:?}", flow));
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::StringIndexError {
            message,
            index,
            length,
        } => {
            let template = template_functions::string_index_error(&message, index, length as usize);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::RangeError {
            message,
            start,
            end,
        } => {
            let template = template_functions::range_error(
                &message,
                start.map(|s| s as i64),
                end.map(|e| e as i64),
            );
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::ListConcatenationError {
            message,
            left_type,
            right_type,
        } => {
            let template =
                template_functions::list_concatenation_error(&message, &left_type, &right_type);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::MapContainsError { message, key_type } => {
            let template = template_functions::map_contains_error(&message, &key_type);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::ConditionalMatchError { message } => {
            let template = template_functions::conditional_match_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::JsonParseError {
            message,
            json_input: _,
        } => {
            let template = template_functions::json_parse_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::JsonGenerateError {
            message,
            value_type,
        } => {
            let template = template_functions::json_generate_error(&message, &value_type);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::YamlParseError {
            message,
            yaml_input: _,
        } => {
            let template = template_functions::yaml_parse_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::YamlGenerateError {
            message,
            value_type,
        } => {
            let template = template_functions::yaml_generate_error(&message, &value_type);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::MapMethodError { method, message } => {
            let template = template_functions::map_method_error(&method, &message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::TomlParseError { message, .. } => {
            let template = template_functions::toml_parse_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::TomlGenerateError { message, .. } => {
            let template = template_functions::toml_generate_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::TomlConversionError { message } => {
            let template = template_functions::toml_conversion_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::StreamError { message } => {
            let template = template_functions::stream_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
        RuntimeError::SerializationError { message } => {
            let template = template_functions::serialization_error(&message);
            let _ = ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0);
        }
    }
}

/// Try to extract the value type and method name from a generic MethodError message
/// Expected formats:
/// - "String has no method 'name'"
/// - "List has no method 'name'"
/// - "Number has no method 'name'"
/// - "Tuple has no method 'name'"
fn parse_method_error(message: &str) -> Option<(&str, String)> {
    use regex::Regex;

    // Strict pattern to avoid false positives
    let regex = Regex::new(r"^(String|List|Number|Tuple) has no method '([^']+)'$").ok()?;
    let captures = regex.captures(message)?;
    let value_type = captures.get(1)?.as_str();
    let method_name = captures.get(2)?.as_str().to_string();
    Some((value_type, method_name))
}

/// Find the span of a variable usage in the source code
fn find_variable_usage(variable_name: &str, source: &str) -> Option<std::ops::Range<usize>> {
    use regex::Regex;

    // Create a regex pattern that matches the variable name as a whole word
    let pattern = format!(r"\b{}\b", regex::escape(variable_name));
    let regex = match Regex::new(&pattern) {
        Ok(regex) => regex,
        Err(_) => return None,
    };

    // Find the first match
    regex.find(source).map(|mat| mat.start()..mat.end())
}

/// Find similar variable names in the source code for suggestions
fn find_similar_variables(target: &str, source: &str) -> Vec<String> {
    use regex::Regex;

    let mut candidates = Vec::new();

    // Extract all identifiers from the source
    let identifier_regex = Regex::new(r"\b[a-zA-Z][a-zA-Z0-9_]*\b").unwrap();
    let mut seen = std::collections::HashSet::new();

    for mat in identifier_regex.find_iter(source) {
        let ident = mat.as_str();
        if ident != target && !seen.contains(ident) {
            seen.insert(ident.to_string());

            // Enhanced similarity check
            let similarity = calculate_similarity(target, ident);
            if similarity > 0.3 {
                // 30% similarity threshold
                candidates.push((ident.to_string(), similarity));
            }
        }
    }

    // Sort by similarity (highest first) and take top 3
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    candidates.truncate(3);
    candidates.into_iter().map(|(name, _)| name).collect()
}

/// Calculate similarity between two strings (simple implementation)
fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }

    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    // Check for exact prefix match
    let min_len = len1.min(len2);
    let mut common_prefix = 0;
    for i in 0..min_len {
        if s1.chars().nth(i) == s2.chars().nth(i) {
            common_prefix += 1;
        } else {
            break;
        }
    }

    // Check for substring match
    let substring_match = if len1 <= len2 {
        s2.contains(s1)
    } else {
        s1.contains(s2)
    };

    // Calculate similarity score
    let prefix_score = common_prefix as f64 / min_len as f64;
    let length_score = 1.0 - (len1 as f64 - len2 as f64).abs() / (len1 + len2) as f64;
    let substring_score = if substring_match { 0.5 } else { 0.0 };

    (prefix_score * 0.5 + length_score * 0.3 + substring_score * 0.2).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_context_creation() {
        let source = "hello world";
        let context = DiagnosticContext::new(source.to_string());
        assert_eq!(context.source, "hello world");
        assert_eq!(context.file_id, "input");
        assert!(context.filename.is_none());
    }

    #[test]
    fn test_find_similar_variables() {
        let source = "let my_variable = 42; let my_other_var = 10; let different = 5;";
        let suggestions = find_similar_variables("my_var", source);
        assert!(suggestions.contains(&"my_variable".to_string()));
        assert!(suggestions.contains(&"my_other_var".to_string()));
    }

    #[test]
    fn test_calculate_similarity() {
        assert_eq!(calculate_similarity("hello", "hello"), 1.0);
        assert!(calculate_similarity("hello", "hell") > 0.7);
        assert!(calculate_similarity("hello", "world") < 0.5);
        assert!(calculate_similarity("var", "variable") > 0.5);
    }
}
