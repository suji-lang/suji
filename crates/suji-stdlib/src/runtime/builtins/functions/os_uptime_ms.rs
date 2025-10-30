//! Built-in: os:uptime_ms() -> number (milliseconds since boot).

use suji_runtime::value::{DecimalNumber, RuntimeError, Value};
use sysinfo::System;

/// Returns system uptime in milliseconds since boot.
pub fn builtin_os_uptime_ms(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "os:uptime_ms() takes no arguments".to_string(),
        });
    }

    let uptime_secs = System::uptime();

    // Convert seconds to milliseconds
    let uptime_ms = uptime_secs.saturating_mul(1000);

    // Convert to DecimalNumber
    let uptime_decimal = DecimalNumber::parse(&uptime_ms.to_string()).map_err(|e| {
        RuntimeError::InvalidOperation {
            message: format!("Failed to convert uptime to number: {}", e),
        }
    })?;

    Ok(Value::Number(uptime_decimal))
}
