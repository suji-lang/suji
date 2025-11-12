//! Built-in: os:uid() -> number (user ID).

use suji_values::value::{DecimalNumber, RuntimeError, Value};

#[cfg(unix)]
use libc;

/// Returns the user ID (Unix) or 0 (Windows).
pub fn builtin_os_uid(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:uid() takes no arguments".to_string(),
        });
    }

    #[cfg(unix)]
    let uid = unsafe { libc::getuid() };

    #[cfg(windows)]
    let uid = 0; // Windows does not have a POSIX uid; return 0 as placeholder

    let uid_decimal =
        DecimalNumber::parse(&uid.to_string()).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to convert UID to number: {}", e),
        })?;

    Ok(Value::Number(uid_decimal))
}
