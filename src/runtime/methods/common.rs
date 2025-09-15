use super::super::env::Env;
use super::super::value::{RuntimeError, Value};
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

/// Evaluate a closure (function value) with the given arguments
pub fn eval_closure(closure: &Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match closure {
        Value::Function(func) => {
            // Create a new environment for the closure
            let call_env = Rc::new(Env::new_child(func.env.clone()));

            // Bind arguments to parameters
            for (param, arg_value) in func.params.iter().zip(args) {
                call_env.define_or_set(&param.name, arg_value);
            }

            // Execute the function body
            let mut loop_stack = Vec::new();
            match super::super::eval::eval_stmt(&func.body, call_env, &mut loop_stack) {
                Ok(result) => Ok(result.unwrap_or(Value::Nil)),
                Err(RuntimeError::ControlFlow {
                    flow: super::super::value::ControlFlow::Return(value),
                }) => Ok(*value),
                Err(other_error) => Err(other_error),
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: "Expected a function for closure evaluation".to_string(),
        }),
    }
}
