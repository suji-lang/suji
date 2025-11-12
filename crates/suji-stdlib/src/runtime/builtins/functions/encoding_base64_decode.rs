use crate::runtime::builtins::common::one_string_arg;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_encoding_base64_decode(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "encoding:base64_decode")?;
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|_| RuntimeError::TypeError {
            message: "invalid base64".to_string(),
        })?;
    let text = String::from_utf8(bytes).map_err(|_| RuntimeError::TypeError {
        message: "decoded base64 is not valid UTF-8".to_string(),
    })?;
    Ok(Value::String(text))
}
