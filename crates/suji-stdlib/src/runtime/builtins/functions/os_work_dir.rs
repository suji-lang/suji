//! Built-in: os:work_dir() -> string (current working directory).

use suji_values::value::{RuntimeError, Value};

/// Returns the current working directory path.
pub fn builtin_os_work_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:work_dir() takes no arguments".to_string(),
        });
    }

    let work_dir = std::env::current_dir().map_err(|e| RuntimeError::InvalidOperation {
        message: format!("Failed to retrieve current directory: {}", e),
    })?;

    let work_dir_str = work_dir.to_string_lossy().to_string();

    Ok(Value::String(work_dir_str))
}
