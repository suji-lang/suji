use std::path::Path;

use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::runtime::value::RuntimeError;
use crate::token::Span;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

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

/// Convert a Span to ariadne's range format
fn span_to_range(span: &Span) -> std::ops::Range<usize> {
    span.start..span.end
}

/// Convert line/column to ariadne's range format (for errors without spans)
fn line_column_to_range(source: &str, line: usize, column: usize) -> std::ops::Range<usize> {
    // Handle the case where the source ends with a newline (common case)
    let lines: Vec<&str> = source.lines().collect();
    let actual_line_count = if source.ends_with('\n') {
        lines.len() + 1
    } else {
        lines.len()
    };

    if line > 0 && line <= actual_line_count {
        // Calculate the byte position of the start of the line
        let mut pos = 0;
        for (i, line_content) in source.lines().enumerate() {
            if i == line - 1 {
                break;
            }
            pos += line_content.len() + 1; // +1 for the newline character
        }
        // Add the column offset (column is 1-based, so subtract 1)
        let final_pos = pos + column.saturating_sub(1);
        final_pos..final_pos
    } else {
        0..0
    }
}

/// Print a lexer error using ariadne
fn print_lex_error(error: LexError, context: &DiagnosticContext) {
    let mut colors = ColorGenerator::new();
    let color = colors.next();

    let (message, span, help) = match error {
        LexError::UnterminatedString { line, column } => (
            "Unterminated string literal".to_string(),
            line_column_to_range(&context.source, line, column),
            "Add a closing quote (\") to terminate the string".to_string(),
        ),
        LexError::UnterminatedShellCommand { line, column } => (
            "Unterminated shell command".to_string(),
            line_column_to_range(&context.source, line, column),
            "Add a closing backtick (`) to terminate the shell command".to_string(),
        ),
        LexError::UnterminatedRegex { line, column } => (
            "Unterminated regex literal".to_string(),
            line_column_to_range(&context.source, line, column),
            "Add a closing slash (/) to terminate the regex".to_string(),
        ),
        LexError::InvalidEscape {
            escape,
            line,
            column,
        } => {
            let suggestion = match escape {
                'n' => "Use \\n for newline",
                't' => "Use \\t for tab",
                '"' => "Use \\\" for quote",
                '\\' => "Use \\\\ for backslash",
                _ => "Valid escape sequences are: \\n, \\t, \\\", \\\\",
            };
            (
                format!("Invalid escape sequence '\\{}'", escape),
                line_column_to_range(&context.source, line, column),
                suggestion.to_string(),
            )
        }
        LexError::InvalidNumber {
            literal,
            line,
            column,
        } => (
            format!("Invalid number literal '{}'", literal),
            line_column_to_range(&context.source, line, column),
            "Numbers can be integers (42) or decimals (3.14)".to_string(),
        ),
        LexError::UnexpectedCharacter { ch, line, column } => {
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
                ';' => "NN language doesn't use semicolons. Remove this character",
                '@' => "This character is not used in NN language. Did you mean something else?",
                '#' => "Comments start with # and continue to the end of the line",
                _ => {
                    "This character is not valid in this context. Check for typos or missing quotes"
                }
            };
            (
                format!("Unexpected character '{}'", ch),
                line_column_to_range(&context.source, line, column),
                suggestion.to_string(),
            )
        }
    };

    let report = Report::build(ReportKind::Error, (&context.file_id, span.clone()))
        .with_code(1)
        .with_message(&message)
        .with_label(
            Label::new((&context.file_id, span))
                .with_message(&message)
                .with_color(color),
        )
        .with_note(help)
        .finish();

    report
        .print((&context.file_id, Source::from(&context.source)))
        .unwrap();
}

/// Print a parser error using ariadne
fn print_parse_error(error: ParseError, context: &DiagnosticContext) {
    let mut colors = ColorGenerator::new();
    let color = colors.next();

    match error {
        ParseError::Lex(lex_error) => print_lex_error(lex_error, context),
        ParseError::UnexpectedToken { token, span } => {
            let suggestion = match &token {
                crate::token::Token::Identifier(name) => {
                    format!("Did you mean to use '{}' as a variable name? Variables must be assigned before use", name)
                }
                crate::token::Token::Number(_) => "This number is not valid in this context. Check for missing operators or syntax".to_string(),
                crate::token::Token::StringStart => "Did you mean to start a string literal? Use quotes: \"text\"".to_string(),
                crate::token::Token::ShellStart => "Did you mean to start a shell command? Use backticks: `command`".to_string(),
                crate::token::Token::RegexStart => "Did you mean to start a regex literal? Use slashes: /pattern/".to_string(),
                crate::token::Token::LeftBrace => "Did you mean to start a block or map? Use braces: { }".to_string(),
                crate::token::Token::RightBrace => "Did you mean to close a block or map? Check for matching opening brace".to_string(),
                crate::token::Token::LeftBracket => "Did you mean to start a list? Use brackets: [ ]".to_string(),
                crate::token::Token::RightBracket => "Did you mean to close a list? Check for matching opening bracket".to_string(),
                crate::token::Token::LeftParen => "Did you mean to start a grouping? Use parentheses: ( )".to_string(),
                crate::token::Token::RightParen => "Did you mean to close a grouping? Check for matching opening parenthesis".to_string(),
                crate::token::Token::Comma => "Comma is not expected here. Check for missing values or extra commas".to_string(),
                crate::token::Token::Assign => "Assignment operator '=' is not expected here. Check for missing variable name".to_string(),
                crate::token::Token::Equal => "Comparison operator '==' is not expected here. Check for missing left operand".to_string(),
                _ => "This token is not expected here. Check for syntax errors or missing elements".to_string(),
            };

            let report = Report::build(
                ReportKind::Error,
                (&context.file_id, span.start..span.start),
            )
            .with_code(2)
            .with_message("Unexpected token")
            .with_label(
                Label::new((&context.file_id, span_to_range(&span)))
                    .with_message(format!("Unexpected {:?}", token))
                    .with_color(color),
            )
            .with_note(suggestion)
            .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        ParseError::UnexpectedEof => {
            let report = Report::build(ReportKind::Error, (&context.file_id, context.source.len()..context.source.len()))
                .with_code(3)
                .with_message("Unexpected end of input")
                .with_note("The input ended unexpectedly. Check for missing closing brackets, quotes, or other syntax")
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        ParseError::Generic { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(4)
                .with_message(&message)
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        ParseError::MultipleExports { span } => {
            let report = Report::build(
                ReportKind::Error,
                (&context.file_id, span.start..span.start),
            )
            .with_code(5)
            .with_message("Multiple export statements found")
            .with_label(
                Label::new((&context.file_id, span_to_range(&span)))
                    .with_message("Second export statement here")
                    .with_color(color),
            )
            .with_note("Only one export statement is allowed per file")
            .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
    }
}

/// Print a runtime error using ariadne
fn print_runtime_error(error: RuntimeError, context: &DiagnosticContext) {
    let mut colors = ColorGenerator::new();
    let color = colors.next();

    match error {
        RuntimeError::TypeError { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(10)
                .with_message("Type error")
                .with_note(message)
                .with_note(
                    "NN language is strongly typed. Check that you're using compatible types",
                )
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::UndefinedVariable { name } => {
            let suggestions = find_similar_variables(&name, &context.source);
            let help = if suggestions.is_empty() {
                format!("Variable '{}' is not defined in the current scope", name)
            } else {
                format!(
                    "Variable '{}' is not defined. Did you mean: {}?",
                    name,
                    suggestions.join(", ")
                )
            };

            // Try to find the variable usage in the source code
            let variable_span = find_variable_usage(&name, &context.source);

            let report = if let Some(span) = variable_span {
                Report::build(
                    ReportKind::Error,
                    (&context.file_id, span.start..span.start),
                )
                .with_code(11)
                .with_message("Undefined variable")
                .with_label(
                    Label::new((&context.file_id, span))
                        .with_message(format!("Variable '{}' is not defined", name))
                        .with_color(color),
                )
                .with_note(help)
                .finish()
            } else {
                Report::build(ReportKind::Error, (&context.file_id, 0..0))
                    .with_code(11)
                    .with_message("Undefined variable")
                    .with_note(help)
                    .finish()
            };

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::InvalidOperation { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(12)
                .with_message("Invalid operation")
                .with_note(message)
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::IndexOutOfBounds { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(13)
                .with_message("Index out of bounds")
                .with_note(message)
                .with_note("Use list::length() to check the size before indexing")
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::KeyNotFound { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(14)
                .with_message("Key not found")
                .with_note(message)
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::InvalidKeyType { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(15)
                .with_message("Invalid key type")
                .with_note(message)
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::ShellError { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(16)
                .with_message("Shell command failed")
                .with_note(message)
                .with_note("Shell commands use backticks: `command`. Check the command syntax and permissions")
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::RegexError { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(17)
                .with_message("Regex error")
                .with_note(message)
                .with_note("Regex patterns use slashes: /pattern/. Check the regex syntax")
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::ArityMismatch { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(18)
                .with_message("Arity mismatch")
                .with_note(message)
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::MethodError { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(19)
                .with_message("Method error")
                .with_note(message)
                .with_note("Use the :: operator to call methods: value::method()")
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::InvalidNumberConversion { message } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(20)
                .with_message("Invalid number conversion")
                .with_note(message)
                .with_note("Use string::to_number() to convert strings to numbers")
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
        RuntimeError::ControlFlow { flow } => {
            let report = Report::build(ReportKind::Error, (&context.file_id, 0..0))
                .with_code(21)
                .with_message("Internal control flow error")
                .with_note(format!("Unexpected control flow: {:?}", flow))
                .finish();

            report
                .print((&context.file_id, Source::from(&context.source)))
                .unwrap();
        }
    }
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
