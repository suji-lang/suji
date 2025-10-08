use super::super::value::{RuntimeError, Value};

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

/// Evaluate a closure (function value) with the given arguments
pub fn eval_closure(closure: &Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match closure {
        Value::Function(func) => {
            // Use canonical function call API so methods honor arity/defaults
            super::super::eval::call_function(func, args, None)
        }
        _ => Err(RuntimeError::TypeError {
            message: "Expected a function for closure evaluation".to_string(),
        }),
    }
}
