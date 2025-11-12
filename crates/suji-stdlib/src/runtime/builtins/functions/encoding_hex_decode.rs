use crate::runtime::builtins::common::one_string_arg;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_encoding_hex_decode(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "encoding:hex_decode")?;
    let bytes = hex::decode(s).map_err(|_| RuntimeError::TypeError {
        message: "invalid hex".to_string(),
    })?;
    let text = String::from_utf8(bytes).map_err(|_| RuntimeError::TypeError {
        message: "decoded hex is not valid UTF-8".to_string(),
    })?;
    Ok(Value::String(text))
}
