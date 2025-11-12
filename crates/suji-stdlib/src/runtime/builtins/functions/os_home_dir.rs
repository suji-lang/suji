//! Built-in: os:home_dir() -> string (user home directory path).

use suji_values::value::{RuntimeError, Value};

/// Returns the user's home directory path.
pub fn builtin_os_home_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:home_dir() takes no arguments".to_string(),
        });
    }

    #[cfg(unix)]
    let home_dir = std::env::var("HOME").map_err(|_| RuntimeError::InvalidOperation {
        message: "Home directory not available".to_string(),
    })?;

    #[cfg(windows)]
    let home_dir = std::env::var("USERPROFILE").map_err(|_| RuntimeError::InvalidOperation {
        message: "Home directory not available".to_string(),
    })?;

    Ok(Value::String(home_dir))
}
