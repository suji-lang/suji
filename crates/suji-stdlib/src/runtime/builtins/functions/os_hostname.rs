//! Built-in: os:hostname() -> string (system hostname).

use suji_runtime::value::{RuntimeError, Value};

/// Returns the system hostname.
pub fn builtin_os_hostname(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:hostname() takes no arguments".to_string(),
        });
    }

    let hostname = hostname::get()
        .map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to retrieve hostname: {}", e),
        })?
        .to_string_lossy()
        .to_string();

    Ok(Value::String(hostname))
}
