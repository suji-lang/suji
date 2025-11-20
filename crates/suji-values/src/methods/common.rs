use super::super::env::Env;
use super::super::value::{FunctionValue, RuntimeError, Value};
use std::rc::Rc;

/// Reference to a value that may be mutable (for method calls)
pub enum ValueRef<'a> {
    /// Mutable reference to a value (can be modified by methods)
    Mutable(&'a mut Value),
    /// Immutable reference to a value (read-only methods only)
    Immutable(&'a Value),
}

impl<'a> ValueRef<'a> {
    /// Get the value (immutable access)
    pub fn get(&self) -> &Value {
        match self {
            ValueRef::Mutable(v) => v,
            ValueRef::Immutable(v) => v,
        }
    }

    /// Get mutable access to the value, or error if immutable
    pub fn get_mut(&mut self) -> Result<&mut Value, RuntimeError> {
        match self {
            ValueRef::Mutable(v) => Ok(v),
            ValueRef::Immutable(_) => Err(RuntimeError::MethodError {
                message: "Cannot call mutating method on immutable value".to_string(),
            }),
        }
    }
}

/// Type alias for closure evaluation callback
///
/// This callback is used to invoke closures (function values) from within
/// method implementations, allowing methods to be independent of any specific
/// Executor implementation.
///
/// Using a trait object with HRTB (higher-ranked trait bound) allows the callback
/// to capture context (like module_registry) while working with any lifetime.
pub type ClosureEvaluator<'a> = &'a dyn for<'b> Fn(
    &'b FunctionValue,
    Vec<Value>,
    Option<Rc<Env>>,
) -> Result<Value, RuntimeError>;

/// Evaluate a closure (function value) with the given arguments
///
/// This takes a callback function to actually invoke the function,
/// allowing the method to be independent of any specific Executor implementation.
///
/// # Parameters
/// - `call_fn`: Callback to invoke the function
/// - `closure`: The value that should be a function
/// - `args`: Arguments to pass to the function
/// - `caller_env`: Optional caller environment
pub fn eval_closure<'a>(
    call_fn: ClosureEvaluator<'a>,
    closure: &Value,
    args: Vec<Value>,
    caller_env: Option<Rc<Env>>,
) -> Result<Value, RuntimeError> {
    match closure {
        Value::Function(func) => {
            // Use provided callback to call function
            call_fn(func, args, caller_env)
        }
        _ => Err(RuntimeError::TypeError {
            message: "Expected a function for closure evaluation".to_string(),
        }),
    }
}

/// Handle type-checking methods (is_number, is_bool, is_string, etc.)
pub fn call_type_checking_method(
    method: &str,
    receiver: &Value,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    // Validate arity
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: format!("{}() takes no arguments", method),
        });
    }

    // Determine which type we're checking for
    let expected_type = match method {
        "is_number" => Some("number"),
        "is_bool" => Some("boolean"),
        "is_string" => Some("string"),
        "is_list" => Some("list"),
        "is_map" => Some("map"),
        "is_stream" => Some("stream"),
        "is_function" => Some("function"),
        "is_tuple" => Some("tuple"),
        "is_regex" => Some("regex"),
        _ => None,
    };

    let expected_type = match expected_type {
        Some(t) => t,
        None => {
            return Err(RuntimeError::MethodError {
                message: format!("Unknown type-checking method '{}'", method),
            });
        }
    };

    // Check if receiver matches expected type
    let result = match receiver {
        Value::Nil => false,
        Value::Number(_) => expected_type == "number",
        Value::Boolean(_) => expected_type == "boolean",
        Value::String(_) => expected_type == "string",
        Value::List(_) => expected_type == "list",
        Value::Map(_) => expected_type == "map",
        Value::Tuple(_) => expected_type == "tuple",
        Value::Function(_) => expected_type == "function",
        Value::Stream(_) | Value::StreamProxy(_) => expected_type == "stream",
        Value::Regex(_) => expected_type == "regex",
        Value::EnvMap(_) => expected_type == "map",
        Value::Module(_) => false, // Module is not one of the checked types
    };

    Ok(Value::Boolean(result))
}
