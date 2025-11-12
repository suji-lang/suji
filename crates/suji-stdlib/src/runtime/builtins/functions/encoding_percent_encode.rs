use crate::runtime::builtins::common::one_string_arg;
use suji_values::value::{RuntimeError, Value};

pub fn builtin_encoding_percent_encode(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = one_string_arg(args, "encoding:percent_encode")?;
    let encoded =
        percent_encoding::percent_encode(s.as_bytes(), percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    Ok(Value::String(encoded))
}
