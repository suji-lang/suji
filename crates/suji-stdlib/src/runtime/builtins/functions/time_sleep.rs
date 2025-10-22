use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_time_sleep(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "time:sleep expects 1 argument".to_string(),
        });
    }
    let ms = match &args[0] {
        Value::Number(n) => n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
            message: "time:sleep requires a non-negative integer milliseconds".to_string(),
        })?,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "time:sleep requires number argument".to_string(),
            });
        }
    };
    if ms < 0 {
        return Err(RuntimeError::InvalidOperation {
            message: "time:sleep requires non-negative duration".to_string(),
        });
    }
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    Ok(Value::Nil)
}
