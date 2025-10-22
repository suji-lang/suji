use crate::runtime::builtins::common::one_string_arg;
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_encoding_base64_encode(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "encoding:base64_encode")?;
    use base64::Engine;
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
    ))
}
