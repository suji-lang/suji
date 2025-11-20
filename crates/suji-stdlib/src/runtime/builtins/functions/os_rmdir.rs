//! Built-in: os:rmdir(path) -> nil (remove empty directory).

use std::fs;
use suji_values::value::{RuntimeError, Value};

/// Removes an empty directory. Returns nil on success.
/// Errors if directory is not empty, doesn't exist, or is not a directory.
pub fn builtin_os_rmdir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "os:rmdir(path) expects 1 argument".to_string(),
        });
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "os:rmdir expects path to be a string".to_string(),
            });
        }
    };

    fs::remove_dir(path).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to remove directory '{}': {}", path, e),
    })?;

    Ok(Value::Nil)
}
