use super::types::ControlFlow;

/// Runtime errors that can occur during evaluation
#[derive(Debug, Clone, thiserror::Error)]
pub enum RuntimeError {
    #[error("Type error: {message}")]
    TypeError { message: String },

    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Index out of bounds: {message}")]
    IndexOutOfBounds { message: String },

    #[error("Key not found: {message}")]
    KeyNotFound { message: String },

    #[error("Invalid key type: {message}")]
    InvalidKeyType { message: String },

    #[error("Shell command failed: {message}")]
    ShellError { message: String },

    #[error("Regex error: {message}")]
    RegexError { message: String },

    #[error("Arity mismatch: {message}")]
    ArityMismatch { message: String },

    #[error("Method error: {message}")]
    MethodError { message: String },

    #[error("Invalid number conversion: {message}")]
    InvalidNumberConversion { message: String },

    #[error("Control flow: {flow:?}")]
    ControlFlow { flow: ControlFlow },

    #[error("String index error: {message}")]
    StringIndexError {
        message: String,
        index: i64,
        length: i64,
    },

    #[error("Range error: {message}")]
    RangeError {
        message: String,
        start: Option<f64>,
        end: Option<f64>,
    },

    #[error("List concatenation error: {message}")]
    ListConcatenationError {
        message: String,
        left_type: String,
        right_type: String,
    },

    #[error("Map contains error: {message}")]
    MapContainsError { message: String, key_type: String },

    #[error("Conditional match error: {message}")]
    ConditionalMatchError { message: String },

    #[error("JSON parse error: {message}")]
    JsonParseError {
        message: String,
        json_input: Option<String>,
    },

    #[error("JSON generation error: {message}")]
    JsonGenerateError { message: String, value_type: String },

    #[error("YAML parse error: {message}")]
    YamlParseError {
        message: String,
        yaml_input: Option<String>,
    },

    #[error("YAML generation error: {message}")]
    YamlGenerateError { message: String, value_type: String },

    #[error("TOML parse error: {message}")]
    TomlParseError {
        message: String,
        toml_input: Option<String>,
    },

    #[error("TOML generation error: {message}")]
    TomlGenerateError { message: String, value_type: String },

    #[error("TOML conversion error: {message}")]
    TomlConversionError { message: String },

    #[error("Map method error: {message}")]
    MapMethodError { method: String, message: String },
}
