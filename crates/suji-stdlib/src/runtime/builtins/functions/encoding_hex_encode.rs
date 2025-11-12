use crate::runtime::builtins::common::one_string_arg;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_encoding_hex_encode(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "encoding:hex_encode")?;
    Ok(Value::String(hex::encode(s.as_bytes())))
}
