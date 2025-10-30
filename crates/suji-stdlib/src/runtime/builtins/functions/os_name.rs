//! Built-in: os:name() -> string (operating system name).

use suji_runtime::value::{RuntimeError, Value};

/// Returns the operating system name ("linux", "darwin", or "windows").
pub fn builtin_os_name(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:name() takes no arguments".to_string(),
        });
    }

    let os_name = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        return Err(RuntimeError::InvalidOperation {
            message: "Unsupported operating system".to_string(),
        });
    };

    Ok(Value::String(os_name.to_string()))
}
