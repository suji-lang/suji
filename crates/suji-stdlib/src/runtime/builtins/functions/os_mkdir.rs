//! Built-in: os:mkdir(path[, create_all]) -> nil (create directory).

use std::fs;
use suji_values::value::{RuntimeError, Value};

/// Creates a directory. Returns nil on success.
/// If create_all is true (default), creates all intermediate directories.
/// If create_all is false, only creates the final directory (parent must exist).
pub fn builtin_os_mkdir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "os:mkdir(path[, create_all]) expects 1 or 2 arguments".to_string(),
        });
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "os:mkdir expects path to be a string".to_string(),
            });
        }
    };

    let create_all = if args.len() == 2 {
        match &args[1] {
            Value::Boolean(b) => *b,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "os:mkdir expects create_all to be a boolean".to_string(),
                });
            }
        }
    } else {
        true // default
    };

    let result = if create_all {
        fs::create_dir_all(path)
    } else {
        fs::create_dir(path)
    };

    result.map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to create directory '{}': {}", path, e),
    })?;

    Ok(Value::Nil)
}
