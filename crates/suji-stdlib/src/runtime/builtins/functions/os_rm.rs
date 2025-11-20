//! Built-in: os:rm(path) -> nil (remove file).

use std::fs;
use suji_values::value::{RuntimeError, Value};

/// Removes a file. Returns nil on success.
/// Errors if path is a directory (use os:rmdir instead) or if file doesn't exist.
pub fn builtin_os_rm(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "os:rm(path) expects 1 argument".to_string(),
        });
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "os:rm expects path to be a string".to_string(),
            });
        }
    };

    // Check if path is a directory
    if let Ok(metadata) = fs::metadata(path)
        && metadata.is_dir()
    {
        return Err(RuntimeError::InvalidOperation {
            message: format!("Cannot remove directory '{}'; use os:rmdir", path),
        });
    }

    fs::remove_file(path).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to remove file '{}': {}", path, e),
    })?;

    Ok(Value::Nil)
}
