use crate::runtime::builtins::time::map_now;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_time_now(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "time:now expects 0 arguments".to_string(),
        });
    }
    let now = chrono::Utc::now();
    let epoch_ms = now.timestamp_millis();
    let iso = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    Ok(map_now(epoch_ms, iso, "Z".to_string()))
}
