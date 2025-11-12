use suji_values::value::{RuntimeError, Value};

pub fn builtin_time_format_iso(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "time:format_iso expects 1 or 2 arguments".to_string(),
        });
    }
    let epoch_ms = match &args[0] {
        Value::Number(n) => n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
            message: "epoch_ms must be an integer".to_string(),
        })?,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "epoch_ms must be a number".to_string(),
            });
        }
    };
    let tz = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "tz must be a string".to_string(),
                });
            }
        }
    } else {
        "Z".to_string()
    };
    let dt_utc: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_timestamp_millis(epoch_ms)
        .ok_or_else(|| RuntimeError::InvalidOperation {
        message: "invalid epoch".to_string(),
    })?;
    let out = if tz == "Z" {
        dt_utc.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    } else {
        use std::str::FromStr;
        let offset =
            chrono::FixedOffset::from_str(&tz).map_err(|e| RuntimeError::InvalidOperation {
                message: format!("invalid tz offset: {}", e),
            })?;
        dt_utc
            .with_timezone(&offset)
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
    };
    Ok(Value::String(out))
}
