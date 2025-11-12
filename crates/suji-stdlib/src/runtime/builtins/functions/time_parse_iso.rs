use crate::runtime::builtins::time::map_epoch_tz;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_time_parse_iso(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "time:parse_iso expects 1 argument".to_string(),
        });
    }
    let s = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "time:parse_iso requires string argument".to_string(),
            });
        }
    };
    let dt =
        chrono::DateTime::parse_from_rfc3339(s).map_err(|e| RuntimeError::InvalidOperation {
            message: format!("Invalid ISO-8601 time: {}", e),
        })?;
    let epoch_ms = dt.timestamp_millis();
    let tz = dt.offset().to_string();
    Ok(map_epoch_tz(epoch_ms, tz))
}
