//! Diagnostics: rich error reporting utilities.
use std::path::Path;

use suji_lexer::LexError;
use suji_parser::ParseError;
use suji_runtime::value::RuntimeError;
pub mod error_builder;
pub mod error_template;
pub mod lexer_errors;
pub mod parser_errors;
pub mod runtime_errors;

use error_builder::{ErrorBuilder, line_column_to_range};
use error_template::ErrorTemplate;
use runtime_errors::ErrorTemplateRouter;

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
        DiagnosticKind::Lex(error) => print_lex_error(error, context)?,
        DiagnosticKind::Parse(error) => print_parse_error(error, context)?,
        DiagnosticKind::Runtime(error) => print_runtime_error(error, context)?,
    }
    Ok(())
}

/// Print a lexer error using ariadne
fn print_lex_error(
    error: LexError,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match error {
        LexError::UnterminatedString { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = crate::diagnostics::lexer_errors::unterminated_string();
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::UnterminatedShellCommand { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = crate::diagnostics::lexer_errors::unterminated_shell_command();
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::UnterminatedRegex { line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = crate::diagnostics::lexer_errors::unterminated_regex();
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::InvalidEscape {
            escape,
            line,
            column,
        } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = crate::diagnostics::lexer_errors::invalid_escape(escape);
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::InvalidNumber {
            literal,
            line,
            column,
        } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = crate::diagnostics::lexer_errors::invalid_number(&literal);
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
        LexError::UnexpectedCharacter { ch, line, column } => {
            let range = line_column_to_range(&context.source, line, column);
            let template = crate::diagnostics::lexer_errors::unexpected_character(ch);
            ErrorBuilder::new(template, context.clone()).print_with_range(range)?;
        }
    }
    Ok(())
}

/// Print a parser error using ariadne
fn print_parse_error(
    error: ParseError,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match error {
        ParseError::Lex(lex_error) => print_lex_error(lex_error, context)?,
        ParseError::UnexpectedToken { token, span } => {
            let base = crate::diagnostics::parser_errors::unexpected_token();
            let suggestion = crate::diagnostics::parser_errors::unexpected_token_suggestion(&token);
            let template = ErrorTemplate::new(base.code, base.title, &base.message)
                .with_suggestion(&suggestion);
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        ParseError::UnexpectedEof => {
            let range = context.source.len()..context.source.len();
            ErrorBuilder::new(
                crate::diagnostics::parser_errors::unexpected_eof(),
                context.clone(),
            )
            .print_with_range_no_label(range)?;
        }
        ParseError::Generic { message } => {
            let template = crate::diagnostics::parser_errors::generic_parse_error(&message);
            ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0)?;
        }
        ParseError::MultipleExports { span } => {
            ErrorBuilder::new(
                crate::diagnostics::parser_errors::multiple_exports(),
                context.clone(),
            )
            .print_with_span(span)?;
        }
    }
    Ok(())
}

/// Print a runtime error using ariadne
fn print_runtime_error(
    error: RuntimeError,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if error has an embedded span (preferred path)
    if let Some(span) = error.span() {
        // Use the embedded span for precise error highlighting
        let template = error.without_span().to_template();
        ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        return Ok(());
    }

    // Fall back to special cases and heuristics for errors without spans

    // Handle special case for UndefinedVariable with variable name suggestions
    if let RuntimeError::UndefinedVariable { name } = error.without_span() {
        let suggestions = find_similar_variables(name, &context.source);
        let mut template = error.to_template();

        if !suggestions.is_empty() {
            template = ErrorTemplate::new(template.code, template.title, &template.message)
                .with_suggestion(&format!(
                    "Variable '{}' is not defined. Did you mean: {}?",
                    name,
                    suggestions.join(", ")
                ));
        }

        // Try to find the variable usage in the source code
        if let Some(span) = find_variable_usage(name, &context.source) {
            ErrorBuilder::new(template, context.clone()).print_with_range(span)?;
        } else {
            ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0)?;
        }
    } else {
        // Use centralized router for all other errors
        let template = error.to_template();
        ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0)?;
    }
    Ok(())
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

/// Calculate similarity between two strings (Unicode-safe implementation)
fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }

    let char_count1 = s1.chars().count();
    let char_count2 = s2.chars().count();

    if char_count1 == 0 && char_count2 == 0 {
        return 1.0;
    }
    if char_count1 == 0 || char_count2 == 0 {
        return 0.0;
    }

    // Check for exact prefix match using single-pass iterator
    let min_char_count = char_count1.min(char_count2);
    let mut common_prefix = 0;
    for (a, b) in s1.chars().zip(s2.chars()) {
        if a == b {
            common_prefix += 1;
        } else {
            break;
        }
    }

    // Check for substring match
    let substring_match = if char_count1 <= char_count2 {
        s2.contains(s1)
    } else {
        s1.contains(s2)
    };

    // Calculate similarity score using character counts
    let prefix_score = common_prefix as f64 / min_char_count as f64;
    let length_score =
        1.0 - (char_count1 as f64 - char_count2 as f64).abs() / (char_count1 + char_count2) as f64;
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

    #[test]
    fn test_calculate_similarity_unicode() {
        // Test exact matches
        assert_eq!(calculate_similarity("café", "café"), 1.0);
        assert_eq!(calculate_similarity("🙂", "🙂"), 1.0);
        assert_eq!(calculate_similarity("🙂🌍", "🙂🌍"), 1.0);

        // Test Unicode vs ASCII similarity
        assert!(calculate_similarity("café", "cafe") > 0.6); // ~0.675
        assert!(calculate_similarity("naïve", "naive") > 0.4); // ~0.5
        assert!(calculate_similarity("résumé", "resume") > 0.3); // ~0.38

        // Test emoji and mixed content similarity
        assert!(calculate_similarity("hello🌍", "hello🌎") > 0.7); // ~0.716
        assert!(calculate_similarity("test🚀", "test🛸") > 0.5); // Common prefix "test"
        assert!(calculate_similarity("a🙂b", "a🙂c") > 0.6); // Common prefix "a🙂"

        // Test comparative similarity (higher similarity should be greater)
        assert!(calculate_similarity("café", "cafe") > calculate_similarity("café", "xyz"));
        assert!(
            calculate_similarity("hello🌍", "hello🌎") > calculate_similarity("hello🌍", "world")
        );
    }
}
