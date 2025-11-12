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
