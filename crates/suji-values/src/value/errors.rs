use super::types::ControlFlow;
use suji_ast::Span;
use suji_parser::ParseError;

/// Runtime errors that can occur during evaluation
#[derive(Debug, Clone, thiserror::Error)]
pub enum RuntimeError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

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

    #[error("CSV parse error: {message}")]
    CsvParseError {
        message: String,
        csv_input: Option<String>,
    },

    #[error("CSV generation error: {message}")]
    CsvGenerateError { message: String, value_type: String },

    #[error("Map method error: {message}")]
    MapMethodError { method: String, message: String },

    #[error("Stream error: {message}")]
    StreamError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    // Pipe-related errors (structured variants)
    #[error("Pipe error: {message}")]
    PipeStageTypeError { message: String },

    #[error("Pipe error: {message}")]
    EmptyPipeExpression { message: String },

    #[error("Pipe execution error ({stage}): {message}")]
    PipeExecutionError { stage: String, message: String },

    #[error("Pipe apply (|>) requires a function on the right-hand side")]
    PipeApplyRightTypeError,

    #[error("Pipe apply (<|) requires a function on the left-hand side")]
    PipeApplyLeftTypeError,

    #[error("Destructuring assignment requires a tuple value")]
    DestructureTypeError,

    #[error("Destructuring arity mismatch: expected {expected}, got {actual}")]
    DestructureArityMismatch { expected: usize, actual: usize },

    #[error("Invalid destructuring target: {message}")]
    DestructureInvalidTarget { message: String },

    /// Runtime error with source location span
    #[error("{error}")]
    WithSpan {
        error: Box<RuntimeError>,
        span: Span,
    },
}

impl RuntimeError {
    /// Wrap this error with a source span
    pub fn with_span(self, span: Span) -> RuntimeError {
        // Check if already wrapped to avoid double-wrapping
        if matches!(self, RuntimeError::WithSpan { .. }) {
            return self;
        }

        // Don't wrap ControlFlow errors - they're signals, not real errors
        if matches!(self, RuntimeError::ControlFlow { .. }) {
            return self;
        }

        RuntimeError::WithSpan {
            error: Box::new(self),
            span,
        }
    }

    /// Get the span if this error carries one
    pub fn span(&self) -> Option<Span> {
        match self {
            RuntimeError::WithSpan { span, .. } => Some(span.clone()),
            _ => None,
        }
    }

    /// Get the underlying error, unwrapping WithSpan if present
    pub fn without_span(&self) -> &RuntimeError {
        match self {
            RuntimeError::WithSpan { error, .. } => error.without_span(),
            _ => self,
        }
    }
}
