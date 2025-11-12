//! Built-in: os:tmp_dir() -> string (temporary directory path).

use suji_values::value::{RuntimeError, Value};

/// Returns the platform-specific temporary directory path.
pub fn builtin_os_tmp_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:tmp_dir() takes no arguments".to_string(),
        });
    }

    let tmp_dir = std::env::temp_dir();
    let tmp_dir_str = tmp_dir.to_string_lossy().to_string();

    Ok(Value::String(tmp_dir_str))
}
