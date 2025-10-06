use super::error_template::ErrorTemplate;
use suji_runtime::value::RuntimeError;

/// Trait for converting runtime errors to diagnostic templates
pub trait ErrorTemplateRouter {
    /// Convert this error to an ErrorTemplate for display
    fn to_template(&self) -> ErrorTemplate;
}

/// Error category classifications for grouping similar errors
#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
    /// Type-related errors (TypeError, InvalidOperation, etc.)
    Type,
    /// Variable and scope errors (UndefinedVariable)
    Scope,
    /// Index and key access errors (IndexOutOfBounds, KeyNotFound, etc.)
    Access,
    /// Method dispatch errors (MethodError, MapMethodError, etc.)
    Method,
    /// Serialization format errors (JSON/YAML/TOML parse/generate)
    Serialization,
    /// System and I/O errors (ShellError, StreamError, etc.)
    System,
    /// Control flow and execution errors (ControlFlow, ArityMismatch)
    Execution,
    /// String and range operations (StringIndexError, RangeError)
    Range,
}

/// Structured error context extracted from RuntimeError variants
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The error category for grouping similar errors
    pub category: ErrorCategory,
    /// Primary error message
    pub primary_message: String,
    /// Contextual suggestions based on error content
    pub suggestions: Vec<String>,
    /// Unique error code derived from variant discriminant
    pub error_code: u32,
    /// Error title for display
    pub title: &'static str,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(
        category: ErrorCategory,
        error_code: u32,
        title: &'static str,
        primary_message: String,
    ) -> Self {
        Self {
            category,
            primary_message,
            suggestions: Vec::new(),
            error_code,
            title,
        }
    }

    /// Add a suggestion to the context
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add multiple suggestions to the context
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// Convert to ErrorTemplate
    pub fn to_template(self) -> ErrorTemplate {
        let mut template = ErrorTemplate::new(self.error_code, self.title, &self.primary_message);
        for suggestion in self.suggestions {
            template = template.with_suggestion(&suggestion);
        }
        template
    }
}

/// Generate error code from RuntimeError variant discriminant
/// This ensures consistent, unique error codes for each variant
pub fn error_code_for_variant(error: &RuntimeError) -> u32 {
    // Base code 100 for runtime errors, add variant-specific offset
    match error {
        RuntimeError::TypeError { .. } => 100,
        RuntimeError::UndefinedVariable { .. } => 101,
        RuntimeError::InvalidOperation { .. } => 102,
        RuntimeError::IndexOutOfBounds { .. } => 103,
        RuntimeError::KeyNotFound { .. } => 104,
        RuntimeError::InvalidKeyType { .. } => 105,
        RuntimeError::ShellError { .. } => 106,
        RuntimeError::RegexError { .. } => 107,
        RuntimeError::ArityMismatch { .. } => 108,
        RuntimeError::MethodError { .. } => 109,
        RuntimeError::InvalidNumberConversion { .. } => 110,
        RuntimeError::ControlFlow { .. } => 111,
        RuntimeError::StringIndexError { .. } => 112,
        RuntimeError::RangeError { .. } => 113,
        RuntimeError::ListConcatenationError { .. } => 114,
        RuntimeError::MapContainsError { .. } => 115,
        RuntimeError::ConditionalMatchError { .. } => 116,
        RuntimeError::JsonParseError { .. } => 117,
        RuntimeError::JsonGenerateError { .. } => 118,
        RuntimeError::YamlParseError { .. } => 119,
        RuntimeError::YamlGenerateError { .. } => 120,
        RuntimeError::TomlParseError { .. } => 121,
        RuntimeError::TomlGenerateError { .. } => 122,
        RuntimeError::TomlConversionError { .. } => 123,
        RuntimeError::MapMethodError { .. } => 124,
        RuntimeError::StreamError { .. } => 125,
        RuntimeError::SerializationError { .. } => 126,
        RuntimeError::PipeStageTypeError { .. } => 127,
        RuntimeError::EmptyPipeExpression { .. } => 128,
        RuntimeError::PipeExecutionError { .. } => 129,
        RuntimeError::PipeApplyRightTypeError => 130,
        RuntimeError::PipeApplyLeftTypeError => 131,
        RuntimeError::DestructureTypeError => 132,
        RuntimeError::DestructureArityMismatch { .. } => 133,
        RuntimeError::DestructureInvalidTarget { .. } => 134,
        // WithSpan wraps another error, unwrap and recurse
        RuntimeError::WithSpan { error, .. } => error_code_for_variant(error),
    }
}

/// Extract method information from method error messages
/// Returns (value_type, method_name) if parseable
pub fn parse_method_error_info(message: &str) -> Option<(String, String)> {
    use regex::Regex;

    // Pattern: "Type has no method 'method_name'"
    let regex = Regex::new(r"^(\w+) has no method '([^']+)'$").ok()?;
    let captures = regex.captures(message)?;

    let value_type = captures.get(1)?.as_str().to_string();
    let method_name = captures.get(2)?.as_str().to_string();

    Some((value_type, method_name))
}

/// Generate contextual suggestions based on error category and content
pub fn generate_category_suggestions(category: ErrorCategory, error: &RuntimeError) -> Vec<String> {
    match category {
        ErrorCategory::Type => vec![
            "The SUJI language is strongly typed. Check that you're using compatible types"
                .to_string(),
        ],
        ErrorCategory::Scope => vec![
            "Variables must be defined before use".to_string(),
            "Check variable name spelling and scope".to_string(),
        ],
        ErrorCategory::Access => vec![
            "Check array/map bounds and key existence".to_string(),
            "Use length() methods to verify bounds before access".to_string(),
        ],
        ErrorCategory::Method => {
            let mut suggestions =
                vec!["Use the :: operator to call methods: value::method()".to_string()];

            // Add type-specific method suggestions
            if let RuntimeError::MethodError { message } = error
                && let Some((value_type, _)) = parse_method_error_info(message)
            {
                suggestions.push(generate_method_help(&value_type));
            }

            suggestions
        }
        ErrorCategory::Serialization => vec![
            "Check syntax and data type compatibility".to_string(),
            "Some values like functions and regex cannot be serialized".to_string(),
        ],
        ErrorCategory::System => vec![
            "Check system permissions and resource availability".to_string(),
            "Verify command syntax and arguments".to_string(),
        ],
        ErrorCategory::Execution => {
            vec!["Check function signatures and control flow usage".to_string()]
        }
        ErrorCategory::Range => vec![
            "Verify index bounds and range parameters".to_string(),
            "Use negative indices to count from the end".to_string(),
        ],
    }
}

/// Generate method help text for specific value types
pub fn generate_method_help(value_type: &str) -> String {
    match value_type {
        "String" => "Available methods: length, contains, starts_with, ends_with, replace, trim, upper, lower, reverse, repeat".to_string(),
        "List" => "Available methods: length, push, pop, contains, reverse, sort, min, max, first, last, average, join".to_string(),
        "Map" => "Available methods: keys, values, to_list, length, contains, delete, get, merge".to_string(),
        "Number" => "Available methods: abs, ceil, floor, round, sqrt, pow, min, max, to_string".to_string(),
        "Tuple" => "Available methods: length, to_list, to_string".to_string(),
        "Stream" => "Available methods: read, write, read_all, read_lines, read_line, is_terminal, close, to_string".to_string(),
        _ => format!("Check available methods for {} type", value_type),
    }
}

// Extension trait to add diagnostic methods to RuntimeError without violating orphan rules
pub trait RuntimeErrorExt {
    fn to_error_context(&self) -> ErrorContext;
}

impl RuntimeErrorExt for RuntimeError {
    /// Convert RuntimeError to structured ErrorContext
    fn to_error_context(&self) -> ErrorContext {
        let error_code = error_code_for_variant(self);

        match self {
            // Type-related errors
            RuntimeError::TypeError { message } => {
                ErrorContext::new(
                    ErrorCategory::Type,
                    error_code,
                    "Type error",
                    message.clone(),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Type, self))
            }
            RuntimeError::InvalidOperation { message } => {
                ErrorContext::new(
                    ErrorCategory::Type,
                    error_code,
                    "Invalid operation",
                    message.clone(),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Type, self))
            }
            // Pipe-related structured errors
            RuntimeError::PipeStageTypeError { message } => {
                ErrorContext::new(
                    ErrorCategory::Execution,
                    error_code,
                    "Pipe stage type error",
                    message.clone(),
                )
            }
            RuntimeError::EmptyPipeExpression { message } => {
                ErrorContext::new(
                    ErrorCategory::Execution,
                    error_code,
                    "Empty pipe expression",
                    message.clone(),
                )
            }
            RuntimeError::PipeExecutionError { stage, message } => {
                ErrorContext::new(
                    ErrorCategory::Execution,
                    error_code,
                    "Pipe execution error",
                    format!("{}: {}", stage, message),
                )
            }
            RuntimeError::PipeApplyRightTypeError => {
                ErrorContext::new(
                    ErrorCategory::Type,
                    error_code,
                    "Pipe apply type error",
                    "Pipe apply (|>) requires a function on the right-hand side".to_string(),
                )
            }
            RuntimeError::PipeApplyLeftTypeError => {
                ErrorContext::new(
                    ErrorCategory::Type,
                    error_code,
                    "Pipe apply type error",
                    "Pipe apply (<|) requires a function on the left-hand side".to_string(),
                )
            }
            RuntimeError::DestructureTypeError => {
                ErrorContext::new(
                    ErrorCategory::Type,
                    error_code,
                    "Destructuring type error",
                    "Destructuring assignment requires a tuple value".to_string(),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Type, self))
            }
            RuntimeError::DestructureArityMismatch { expected, actual } => {
                ErrorContext::new(
                    ErrorCategory::Execution,
                    error_code,
                    "Destructuring arity mismatch",
                    format!("Expected {} values but got {}", expected, actual),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Execution, self))
            }
            RuntimeError::DestructureInvalidTarget { message } => {
                ErrorContext::new(
                    ErrorCategory::Type,
                    error_code,
                    "Invalid destructuring target",
                    message.clone(),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Type, self))
            }
            RuntimeError::InvalidNumberConversion { message } => {
                ErrorContext::new(
                    ErrorCategory::Type,
                    error_code,
                    "Invalid number conversion",
                    message.clone(),
                ).with_suggestion("Use string::to_number() to convert strings to numbers".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Type, self))
            }
            // Scope-related errors
            RuntimeError::UndefinedVariable { name } => {
                ErrorContext::new(
                    ErrorCategory::Scope,
                    error_code,
                    "Undefined variable",
                    format!("Variable '{}' is not defined", name),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Scope, self))
            }
            // Access-related errors
            RuntimeError::IndexOutOfBounds { message } => {
                ErrorContext::new(
                    ErrorCategory::Access,
                    error_code,
                    "Index out of bounds",
                    message.clone(),
                ).with_suggestion("Use list::length() to check the size before indexing".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Access, self))
            }
            RuntimeError::KeyNotFound { message } => {
                ErrorContext::new(
                    ErrorCategory::Access,
                    error_code,
                    "Key not found",
                    message.clone(),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Access, self))
            }
            RuntimeError::InvalidKeyType { message } => {
                ErrorContext::new(
                    ErrorCategory::Access,
                    error_code,
                    "Invalid key type",
                    message.clone(),
                ).with_suggestion("Valid key types: number, boolean, string, tuple".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Access, self))
            }
            RuntimeError::MapContainsError { message, key_type } => {
                ErrorContext::new(
                    ErrorCategory::Access,
                    error_code,
                    "Map contains error",
                    message.clone(),
                ).with_suggestion(format!("Key type '{}' is not valid for map contains", key_type))
                .with_suggestion("Valid key types: number, boolean, string, tuple".to_string())
                .with_suggestion("Example: map::contains(\"key\") or map::contains(42)".to_string())
            }
            // Method-related errors
            RuntimeError::MethodError { message } => {
                let mut context = ErrorContext::new(
                    ErrorCategory::Method,
                    error_code,
                    "Method error",
                    message.clone(),
                );

                // Add type-specific suggestions if we can parse the method error
                if let Some((value_type, method_name)) = parse_method_error_info(message) {
                    context = context.with_suggestion(format!("Method '{}' is not available for {} type", method_name, value_type))
                        .with_suggestion(generate_method_help(&value_type));
                }

                context.with_suggestions(generate_category_suggestions(ErrorCategory::Method, self))
            }
            RuntimeError::MapMethodError { method, message } => {
                let method_help = match method.as_str() {
                    "keys" => "map::keys() - returns list of all keys",
                    "values" => "map::values() - returns list of all values",
                    "to_list" => "map::to_list() - returns list of key-value tuples",
                    "length" => "map::length() - returns number of key-value pairs",
                    "contains" => "map::contains(key) - checks if key exists",
                    "delete" => "map::delete(key) - removes key-value pair",
                    "get" => "map::get(key, default=nil) - value or default",
                    "merge" => "map::merge(other_map) - merge entries, overwriting existing keys",
                    _ => "Check the method name and arguments",
                };

                ErrorContext::new(
                    ErrorCategory::Method,
                    error_code,
                    "Map method error",
                    message.clone(),
                ).with_suggestion(format!("Map method '{}' usage:", method))
                .with_suggestion(method_help.to_string())
            }
            // Serialization errors (JSON/YAML/TOML)
            RuntimeError::JsonParseError { message, .. } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "JSON parse error",
                    message.clone(),
                ).with_suggestion("JSON syntax: objects use {}, arrays use [], strings use \"\"".to_string())
                .with_suggestion("Example: json:parse('{\"name\": \"Alice\", \"age\": 30}')".to_string())
                .with_suggestion("Check for missing quotes, commas, or brackets".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            RuntimeError::JsonGenerateError { message, value_type } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "JSON generation error",
                    message.clone(),
                ).with_suggestion(format!("Cannot convert {} to JSON", value_type))
                .with_suggestion("JSON supports: maps, lists, numbers, strings, booleans, nil".to_string())
                .with_suggestion("Functions and regex values cannot be converted to JSON".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            RuntimeError::YamlParseError { message, .. } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "YAML parse error",
                    message.clone(),
                ).with_suggestion("YAML syntax: use colons for key-value pairs, dashes for lists".to_string())
                .with_suggestion("Example: yaml:parse('name: Alice\nage: 30')".to_string())
                .with_suggestion("Check for proper indentation and syntax".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            RuntimeError::YamlGenerateError { message, value_type } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "YAML generation error",
                    message.clone(),
                ).with_suggestion(format!("Cannot convert {} to YAML", value_type))
                .with_suggestion("YAML supports: maps, lists, numbers, strings, booleans, nil".to_string())
                .with_suggestion("Functions and regex values cannot be converted to YAML".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            RuntimeError::TomlParseError { message, .. } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "TOML parse error",
                    message.clone(),
                ).with_suggestion("TOML syntax: use equals for key-value pairs, brackets for tables".to_string())
                .with_suggestion("Example: toml:parse('name = \"Alice\"\nage = 30')".to_string())
                .with_suggestion("Check for proper TOML syntax and formatting".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            RuntimeError::TomlGenerateError { message, .. } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "TOML generation error",
                    message.clone(),
                ).with_suggestion("TOML supports: maps, lists, numbers, strings, booleans".to_string())
                .with_suggestion("TOML does not support nil values".to_string())
                .with_suggestion("Functions and regex values cannot be converted to TOML".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            RuntimeError::TomlConversionError { message } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "TOML conversion error",
                    message.clone(),
                ).with_suggestion("TOML keys must be strings".to_string())
                .with_suggestion("TOML does not support nil values".to_string())
                .with_suggestion("Check the data types being converted".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            RuntimeError::SerializationError { message } => {
                ErrorContext::new(
                    ErrorCategory::Serialization,
                    error_code,
                    "Serialization error",
                    message.clone(),
                ).with_suggestion("Some values like streams, functions, and regex cannot be serialized to JSON/YAML/TOML".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Serialization, self))
            }
            // System errors
            RuntimeError::ShellError { message } => {
                ErrorContext::new(
                    ErrorCategory::System,
                    error_code,
                    "Shell command failed",
                    message.clone(),
                ).with_suggestion("Shell commands use backticks: `command`. Check the command syntax and permissions".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::System, self))
            }
            RuntimeError::StreamError { message } => {
                ErrorContext::new(
                    ErrorCategory::System,
                    error_code,
                    "Stream error",
                    message.clone(),
                ).with_suggestion("Stream operations may block while waiting for I/O".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::System, self))
            }
            RuntimeError::RegexError { message } => {
                ErrorContext::new(
                    ErrorCategory::System,
                    error_code,
                    "Regex error",
                    message.clone(),
                ).with_suggestion("Regex patterns use slashes: /pattern/. Check the regex syntax".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::System, self))
            }
            // Execution errors
            RuntimeError::ArityMismatch { message } => {
                ErrorContext::new(
                    ErrorCategory::Execution,
                    error_code,
                    "Arity mismatch",
                    message.clone(),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Execution, self))
            }
            RuntimeError::ControlFlow { flow } => {
                ErrorContext::new(
                    ErrorCategory::Execution,
                    error_code,
                    "Internal control flow error",
                    format!("Unexpected control flow: {:?}", flow),
                ).with_suggestions(generate_category_suggestions(ErrorCategory::Execution, self))
            }
            RuntimeError::ConditionalMatchError { message } => {
                ErrorContext::new(
                    ErrorCategory::Execution,
                    error_code,
                    "Conditional match error",
                    message.clone(),
                ).with_suggestion("Conditional match syntax: match { condition: body, ... }".to_string())
                .with_suggestion("Each condition must evaluate to a boolean value".to_string())
                .with_suggestion("Example: match { x > 0: \"positive\", _: \"non-positive\" }".to_string())
            }
            // Range errors
            RuntimeError::StringIndexError { message, index, length } => {
                ErrorContext::new(
                    ErrorCategory::Range,
                    error_code,
                    "String index error",
                    message.clone(),
                ).with_suggestion(format!("Attempted to access index {} on string of length {}", index, length))
                .with_suggestion(format!("String has {} characters (0-indexed)", length))
                .with_suggestion(format!("Valid indices: 0 to {} or -{} to -1", length - 1, length))
                .with_suggestion("Use string::length() to check the string size before indexing".to_string())
                .with_suggestions(generate_category_suggestions(ErrorCategory::Range, self))
            }
            RuntimeError::RangeError { message, start, end } => {
                let mut context = ErrorContext::new(
                    ErrorCategory::Range,
                    error_code,
                    "Range error",
                    message.clone(),
                );

                if let (Some(s), Some(e)) = (start, end) {
                    let size = (s - e).abs() as u64;
                    if size > 100_000 {
                        context = context.with_suggestion("Large ranges may consume significant memory".to_string());
                    }
                }

                context.with_suggestion("Range bounds must be integers".to_string())
                    .with_suggestion("Use smaller ranges or consider alternative approaches".to_string())
                    .with_suggestions(generate_category_suggestions(ErrorCategory::Range, self))
            }
            RuntimeError::ListConcatenationError { message, left_type, right_type } => {
                ErrorContext::new(
                    ErrorCategory::Range,
                    error_code,
                    "List concatenation error",
                    message.clone(),
                ).with_suggestion(format!("Cannot concatenate {} and {}", left_type, right_type))
                .with_suggestion("List concatenation requires both operands to be lists".to_string())
                .with_suggestion("Use list::push() to add individual items to a list".to_string())
            }
            // WithSpan wraps another error, unwrap and recurse
            RuntimeError::WithSpan { error, .. } => error.to_error_context(),
        }
    }
}

impl ErrorTemplateRouter for RuntimeError {
    fn to_template(&self) -> ErrorTemplate {
        let context = self.to_error_context();
        context.to_template()
    }
}
