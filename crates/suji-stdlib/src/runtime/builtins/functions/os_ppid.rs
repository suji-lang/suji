//! Built-in: os:ppid() -> number (parent process ID).

use suji_values::value::{DecimalNumber, RuntimeError, Value};

#[cfg(unix)]
use libc;

/// Returns the parent process ID.
pub fn builtin_os_ppid(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:ppid() takes no arguments".to_string(),
        });
    }

    #[cfg(unix)]
    let ppid = unsafe { libc::getppid() };

    #[cfg(windows)]
    let ppid = 0; // Windows does not have a simple equivalent; return 0 as placeholder

    let ppid_decimal =
        DecimalNumber::parse(&ppid.to_string()).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Failed to convert PPID to number: {}", e),
        })?;

    Ok(Value::Number(ppid_decimal))
}
