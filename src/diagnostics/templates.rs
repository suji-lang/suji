/// Error template for consistent error message formatting
#[derive(Debug, Clone)]
pub struct ErrorTemplate {
    /// Error code for identification
    pub code: u32,
    /// Short title for the error
    pub title: &'static str,
    /// Main error message
    pub message: String,
    /// Helpful suggestions for fixing the error
    pub suggestions: Vec<String>,
}

impl ErrorTemplate {
    /// Create a new error template
    pub fn new(code: u32, title: &'static str, message: &str) -> Self {
        Self {
            code,
            title,
            message: message.to_string(),
            suggestions: Vec::new(),
        }
    }

    /// Add a suggestion to the template
    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestions.push(suggestion.to_string());
        self
    }

    /// Add multiple suggestions to the template
    pub fn with_suggestions(mut self, suggestions: Vec<&str>) -> Self {
        self.suggestions
            .extend(suggestions.iter().map(|s| s.to_string()));
        self
    }
}

/// Predefined error templates for common error types
pub mod predefined {
    use super::ErrorTemplate;

    // Lexer error templates
    pub fn unterminated_string() -> ErrorTemplate {
        ErrorTemplate::new(
            1,
            "Unterminated string literal",
            "Unterminated string literal",
        )
        .with_suggestion("Add a closing quote (\") to terminate the string")
    }

    pub fn unterminated_shell_command() -> ErrorTemplate {
        ErrorTemplate::new(
            1,
            "Unterminated shell command",
            "Unterminated shell command",
        )
        .with_suggestion("Add a closing backtick (`) to terminate the shell command")
    }

    pub fn unterminated_regex() -> ErrorTemplate {
        ErrorTemplate::new(
            1,
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
            1,
            "Invalid escape sequence",
            &format!("Invalid escape sequence '\\{}'", escape),
        )
        .with_suggestion(suggestion)
    }

    pub fn invalid_number(literal: &str) -> ErrorTemplate {
        ErrorTemplate::new(
            1,
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
            ';' => "NN language doesn't use semicolons. Remove this character",
            '@' => "This character is not used in NN language. Did you mean something else?",
            '#' => "Comments start with # and continue to the end of the line",
            _ => "This character is not valid in this context. Check for typos or missing quotes",
        };
        ErrorTemplate::new(
            1,
            "Unexpected character",
            &format!("Unexpected character '{}'", ch),
        )
        .with_suggestion(suggestion)
    }

    // Parser error templates
    pub fn unexpected_token() -> ErrorTemplate {
        ErrorTemplate::new(2, "Unexpected token", "Unexpected token").with_suggestion(
            "This token is not expected here. Check for syntax errors or missing elements",
        )
    }

    pub fn unexpected_eof() -> ErrorTemplate {
        ErrorTemplate::new(3, "Unexpected end of input", "Unexpected end of input")
            .with_suggestion("The input ended unexpectedly. Check for missing closing brackets, quotes, or other syntax")
    }

    pub fn generic_parse_error(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(4, "Parse error", message)
    }

    pub fn multiple_exports() -> ErrorTemplate {
        ErrorTemplate::new(
            5,
            "Multiple export statements found",
            "Multiple export statements found",
        )
        .with_suggestion("Only one export statement is allowed per file")
    }

    // Runtime error templates
    pub fn type_error(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(10, "Type error", "Type error")
            .with_suggestion(message)
            .with_suggestion(
                "NN language is strongly typed. Check that you're using compatible types",
            )
    }

    pub fn undefined_variable(name: &str) -> ErrorTemplate {
        ErrorTemplate::new(11, "Undefined variable", "Undefined variable").with_suggestion(
            &format!("Variable '{}' is not defined in the current scope", name),
        )
    }

    pub fn invalid_operation(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(12, "Invalid operation", "Invalid operation").with_suggestion(message)
    }

    pub fn index_out_of_bounds(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(13, "Index out of bounds", "Index out of bounds")
            .with_suggestion(message)
            .with_suggestion("Use list::length() to check the size before indexing")
    }

    pub fn key_not_found(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(14, "Key not found", "Key not found").with_suggestion(message)
    }

    pub fn invalid_key_type(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(15, "Invalid key type", "Invalid key type").with_suggestion(message)
    }

    pub fn shell_error(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(16, "Shell command failed", "Shell command failed")
            .with_suggestion(message)
            .with_suggestion(
                "Shell commands use backticks: `command`. Check the command syntax and permissions",
            )
    }

    pub fn regex_error(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(17, "Regex error", "Regex error")
            .with_suggestion(message)
            .with_suggestion("Regex patterns use slashes: /pattern/. Check the regex syntax")
    }

    pub fn arity_mismatch(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(18, "Arity mismatch", "Arity mismatch").with_suggestion(message)
    }

    pub fn method_error(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(19, "Method error", "Method error")
            .with_suggestion(message)
            .with_suggestion("Use the :: operator to call methods: value::method()")
    }

    pub fn invalid_number_conversion(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(20, "Invalid number conversion", "Invalid number conversion")
            .with_suggestion(message)
            .with_suggestion("Use string::to_number() to convert strings to numbers")
    }

    pub fn control_flow_error(flow: &str) -> ErrorTemplate {
        ErrorTemplate::new(
            21,
            "Internal control flow error",
            "Internal control flow error",
        )
        .with_suggestion(&format!("Unexpected control flow: {}", flow))
    }

    pub fn string_index_error(message: &str, index: i64, length: usize) -> ErrorTemplate {
        ErrorTemplate::new(22, "String index error", "String index error")
            .with_suggestion(message)
            .with_suggestion(&format!(
                "Attempted to access index {} on string of length {}",
                index, length
            ))
            .with_suggestion(&format!("String has {} characters (0-indexed)", length))
            .with_suggestion(&format!(
                "Valid indices: 0 to {} or -{} to -1",
                length - 1,
                length
            ))
            .with_suggestion("Use string::length() to check the string size before indexing")
    }

    pub fn range_error(message: &str, start: Option<i64>, end: Option<i64>) -> ErrorTemplate {
        let mut template =
            ErrorTemplate::new(23, "Range error", "Range error").with_suggestion(message);

        if let (Some(s), Some(e)) = (start, end) {
            let size = (s - e).unsigned_abs();
            if size > 100_000 {
                template
                    .suggestions
                    .push("Large ranges may consume significant memory".to_string());
            }
        }

        template
            .with_suggestion("Range bounds must be integers")
            .with_suggestion("Use smaller ranges or consider alternative approaches")
    }

    pub fn list_concatenation_error(
        message: &str,
        left_type: &str,
        right_type: &str,
    ) -> ErrorTemplate {
        ErrorTemplate::new(24, "List concatenation error", "List concatenation error")
            .with_suggestion(message)
            .with_suggestion(&format!(
                "Cannot concatenate {} and {}",
                left_type, right_type
            ))
            .with_suggestion("List concatenation requires both operands to be lists")
            .with_suggestion("Use list::push() to add individual items to a list")
    }

    pub fn map_contains_error(message: &str, key_type: &str) -> ErrorTemplate {
        ErrorTemplate::new(25, "Map contains error", "Map contains error")
            .with_suggestion(message)
            .with_suggestion(&format!(
                "Key type '{}' is not valid for map contains",
                key_type
            ))
            .with_suggestion("Valid key types: number, boolean, string, tuple")
            .with_suggestion("Example: map::contains(\"key\") or map::contains(42)")
    }

    pub fn json_parse_error(message: &str) -> ErrorTemplate {
        ErrorTemplate::new(27, "JSON parse error", "JSON parse error")
            .with_suggestion(message)
            .with_suggestion("JSON syntax: objects use {}, arrays use [], strings use \"\"")
            .with_suggestion("Example: json:parse('{\"name\": \"Alice\", \"age\": 30}')")
            .with_suggestion("Check for missing quotes, commas, or brackets")
    }

    pub fn json_generate_error(message: &str, value_type: &str) -> ErrorTemplate {
        ErrorTemplate::new(28, "JSON generation error", "JSON generation error")
            .with_suggestion(message)
            .with_suggestion(&format!("Cannot convert {} to JSON", value_type))
            .with_suggestion("JSON supports: maps, lists, numbers, strings, booleans, nil")
            .with_suggestion("Functions and regex values cannot be converted to JSON")
    }

    pub fn map_method_error(method: &str, message: &str) -> ErrorTemplate {
        let method_help = match method {
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

        ErrorTemplate::new(29, "Map method error", "Map method error")
            .with_suggestion(message)
            .with_suggestion(&format!("Map method '{}' usage:", method))
            .with_suggestion(method_help)
    }

    pub fn string_method_error(method: &str, message: &str) -> ErrorTemplate {
        let method_help = match method {
            "contains" => "string::contains(substring) - returns true if substring exists",
            "starts_with" => {
                "string::starts_with(prefix) - returns true if string starts with prefix"
            }
            "ends_with" => "string::ends_with(suffix) - returns true if string ends with suffix",
            "replace" => "string::replace(old, new) - replace all occurrences of old with new",
            "trim" => "string::trim() - remove leading and trailing whitespace",
            "upper" => "string::upper() - convert to uppercase",
            "lower" => "string::lower() - convert to lowercase",
            "reverse" => "string::reverse() - reverse characters",
            "repeat" => "string::repeat(count) - repeat string count times",
            _ => "Check the method name and arguments",
        };

        ErrorTemplate::new(34, "String method error", "String method error")
            .with_suggestion(message)
            .with_suggestion(&format!("String method '{}' usage:", method))
            .with_suggestion(method_help)
    }

    pub fn list_method_error(method: &str, message: &str) -> ErrorTemplate {
        let method_help = match method {
            "contains" => "list::contains(item) - returns true if item exists",
            "reverse" => "list::reverse() - returns a new list reversed",
            "sort" => "list::sort() - returns a new sorted list",
            "min" => "list::min() - returns minimum number in list",
            "max" => "list::max() - returns maximum number in list",
            "first" => "list::first(default=nil) - returns first element or default",
            "last" => "list::last(default=nil) - returns last element or default",
            "average" => "list::average() - returns arithmetic mean of numbers",
            _ => "Check the method name and arguments",
        };

        ErrorTemplate::new(35, "List method error", "List method error")
            .with_suggestion(message)
            .with_suggestion(&format!("List method '{}' usage:", method))
            .with_suggestion(method_help)
    }

    pub fn number_method_error(method: &str, message: &str) -> ErrorTemplate {
        let method_help = match method {
            "abs" => "number::abs() - absolute value",
            "ceil" => "number::ceil() - smallest integer >= number",
            "floor" => "number::floor() - largest integer <= number",
            "round" => "number::round() - nearest integer",
            "sqrt" => "number::sqrt() - square root (non-negative only)",
            "pow" => "number::pow(exponent) - raise to power",
            "min" => "number::min(other) - minimum of two numbers",
            "max" => "number::max(other) - maximum of two numbers",
            _ => "Check the method name and arguments",
        };

        ErrorTemplate::new(36, "Number method error", "Number method error")
            .with_suggestion(message)
            .with_suggestion(&format!("Number method '{}' usage:", method))
            .with_suggestion(method_help)
    }

    pub fn tuple_method_error(method: &str, message: &str) -> ErrorTemplate {
        let method_help = match method {
            "length" => "tuple::length() - number of elements",
            "to_list" => "tuple::to_list() - convert to list",
            "to_string" => "tuple::to_string() - string representation",
            _ => "Check the method name and arguments",
        };

        ErrorTemplate::new(37, "Tuple method error", "Tuple method error")
            .with_suggestion(message)
            .with_suggestion(&format!("Tuple method '{}' usage:", method))
            .with_suggestion(method_help)
    }
}
