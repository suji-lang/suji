use crate::runtime::builtins::functions::random_core::rng_f64;
use crate::runtime::value::{RuntimeError, Value};

pub fn builtin_random_random(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "random() expects 0 arguments".to_string(),
        });
    }
    Ok(Value::Number(rng_f64()))
}
