use crate::runtime::builtins::common::one_string_arg;
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_encoding_percent_decode(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "encoding:percent_decode")?;
    let decoded = percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .map_err(|_| RuntimeError::TypeError {
            message: "invalid percent encoding".to_string(),
        })?;
    Ok(Value::String(decoded.to_string()))
}
