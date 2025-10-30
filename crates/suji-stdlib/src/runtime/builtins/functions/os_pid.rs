//! Built-in: os:pid() -> number (process ID).

use suji_runtime::value::{DecimalNumber, RuntimeError, Value};

/// Returns the current process ID.
pub fn builtin_os_pid(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:pid() takes no arguments".to_string(),
        });
    }

    let pid = std::process::id();
    let pid_decimal =
        DecimalNumber::parse(&pid.to_string()).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to convert PID to number: {}", e),
        })?;

    Ok(Value::Number(pid_decimal))
}
