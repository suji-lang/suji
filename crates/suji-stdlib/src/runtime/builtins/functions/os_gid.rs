//! Built-in: os:gid() -> number (group ID).

use suji_runtime::value::{DecimalNumber, RuntimeError, Value};

#[cfg(unix)]
use libc;

/// Returns the group ID (Unix) or 0 (Windows).
pub fn builtin_os_gid(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:gid() takes no arguments".to_string(),
        });
    }

    #[cfg(unix)]
    let gid = unsafe { libc::getgid() };

    #[cfg(windows)]
    let gid = 0; // Windows does not have a POSIX gid; return 0 as placeholder

    let gid_decimal =
        DecimalNumber::parse(&gid.to_string()).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to convert GID to number: {}", e),
        })?;

    Ok(Value::Number(gid_decimal))
}
