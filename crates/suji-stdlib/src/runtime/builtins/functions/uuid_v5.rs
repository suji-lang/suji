use suji_values::value::{RuntimeError, Value};

pub fn builtin_uuid_v5(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "uuid:v5 expects 2 arguments".to_string(),
        });
    }
    let ns = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "uuid:v5 namespace must be string".to_string(),
            });
        }
    };
    let name = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "uuid:v5 name must be string".to_string(),
            });
        }
    };
    let ns_uuid = uuid::Uuid::parse_str(ns).map_err(|e| RuntimeError::InvalidOperation {
        message: format!("invalid namespace uuid: {}", e),
    })?;
    let result = uuid::Uuid::new_v5(&ns_uuid, name.as_bytes());
    Ok(Value::String(
        result.hyphenated().to_string().to_lowercase(),
    ))
}
