use crate::ast::{Param, Stmt};
use indexmap::IndexMap;
use regex::Regex;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Runtime values in the NN language
#[derive(Debug, Clone)]
pub enum Value {
    /// 64-bit floating point number
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Unicode string
    String(String),
    /// Growable list of values (pass-by-value)
    List(Vec<Value>),
    /// Key-value map with deterministic iteration
    Map(IndexMap<MapKey, Value>),
    /// Immutable tuple
    Tuple(Vec<Value>),
    /// Compiled regular expression
    Regex(Regex),
    /// Function with closure
    Function(FunctionValue),
    /// Null value (for functions without explicit return)
    Null,
}

/// Function value with parameters, body, and captured environment
#[derive(Clone)]
pub struct FunctionValue {
    /// Parameter specifications with optional defaults
    pub params: Vec<ParamSpec>,
    /// Function body AST
    pub body: Stmt,
    /// Captured closure environment
    pub env: Rc<super::env::Env>,
}

impl fmt::Debug for FunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Custom Debug implementation to avoid circular reference with env
        f.debug_struct("FunctionValue")
            .field("params", &self.params)
            .field("body", &self.body)
            .field(
                "env",
                &format_args!("Env{{ {} bindings }}", self.env.binding_count()),
            )
            .finish()
    }
}

/// Runtime parameter specification
#[derive(Debug, Clone, PartialEq)]
pub struct ParamSpec {
    /// Parameter name
    pub name: String,
    /// Optional default value expression (evaluated at call time)
    pub default: Option<crate::ast::Expr>,
}

/// Wrapper for map keys that implements Hash and Eq for valid key types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapKey {
    Number(OrderedFloat),
    Boolean(bool),
    String(String),
    Tuple(Vec<MapKey>),
}

/// Wrapper for f64 that implements Eq and Hash for use as map keys
#[derive(Debug, Clone, Copy, PartialOrd)]
pub struct OrderedFloat(pub f64);

/// Control flow signals for break/continue/return
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Return(Value),
    Break(Option<String>),    // Optional label
    Continue(Option<String>), // Optional label
}

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

    #[error("Control flow: {flow:?}")]
    ControlFlow { flow: ControlFlow },
}

impl Value {
    /// Get the type name of this value for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Tuple(_) => "tuple",
            Value::Regex(_) => "regex",
            Value::Function(_) => "function",
            Value::Null => "null",
        }
    }

    /// Check if this value is truthy (only true for Boolean(true))
    pub fn is_truthy(&self) -> bool {
        matches!(self, Value::Boolean(true))
    }

    /// Convert value to string representation
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::List(items) => {
                let item_strings: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                format!("[{}]", item_strings.join(", "))
            }
            Value::Map(map) => {
                let entries: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string()))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
            Value::Tuple(items) => {
                let item_strings: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                if items.len() == 1 {
                    format!("({},)", item_strings[0])
                } else {
                    format!("({})", item_strings.join(", "))
                }
            }
            Value::Regex(regex) => format!("/{}/", regex.as_str()),
            Value::Function(_) => "<function>".to_string(),
            Value::Null => "null".to_string(),
        }
    }

    /// Try to convert this value to a MapKey
    pub fn try_into_map_key(self) -> Result<MapKey, RuntimeError> {
        match self {
            Value::Number(n) => Ok(MapKey::Number(OrderedFloat(n))),
            Value::Boolean(b) => Ok(MapKey::Boolean(b)),
            Value::String(s) => Ok(MapKey::String(s)),
            Value::Tuple(items) => {
                let key_items: Result<Vec<MapKey>, RuntimeError> = items
                    .into_iter()
                    .map(|item| item.try_into_map_key())
                    .collect();
                Ok(MapKey::Tuple(key_items?))
            }
            _ => Err(RuntimeError::InvalidKeyType {
                message: format!("{} cannot be used as a map key", self.type_name()),
            }),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        // Functions are equal if they have the same parameters and body
        // Environment comparison is intentionally omitted
        self.params == other.params && self.body == other.body
    }
}

impl From<Param> for ParamSpec {
    fn from(param: Param) -> Self {
        ParamSpec {
            name: param.name,
            default: param.default,
        }
    }
}

impl MapKey {
    /// Convert MapKey back to a Value
    pub fn to_value(&self) -> Value {
        match self {
            MapKey::Number(OrderedFloat(n)) => Value::Number(*n),
            MapKey::Boolean(b) => Value::Boolean(*b),
            MapKey::String(s) => Value::String(s.clone()),
            MapKey::Tuple(items) => Value::Tuple(items.iter().map(|k| k.to_value()).collect()),
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.to_value().to_string()
    }
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Hash for MapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MapKey::Number(OrderedFloat(n)) => {
                0u8.hash(state); // discriminant
                n.to_bits().hash(state);
            }
            MapKey::Boolean(b) => {
                1u8.hash(state); // discriminant
                b.hash(state);
            }
            MapKey::String(s) => {
                2u8.hash(state); // discriminant
                s.hash(state);
            }
            MapKey::Tuple(items) => {
                3u8.hash(state); // discriminant
                items.hash(state);
            }
        }
    }
}

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedFloat {}

impl Hash for OrderedFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}
