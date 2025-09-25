use crate::runtime::builtins::functions::random_core::rng_seed_with;
use crate::runtime::value::{RuntimeError, Value};

pub fn builtin_random_seed(args: &[Value]) -> Result<Value, RuntimeError> {
    match args.len() {
        0 => {
            rng_seed_with(None);
            Ok(Value::Nil)
        }
        1 => match &args[0] {
            Value::Nil => {
                rng_seed_with(None);
                Ok(Value::Nil)
            }
            Value::Number(n) => {
                rng_seed_with(Some(*n as u64));
                Ok(Value::Nil)
            }
            other => Err(RuntimeError::TypeError {
                message: format!(
                    "random:seed expects a number or nil, got {}",
                    other.type_name()
                ),
            }),
        },
        _ => Err(RuntimeError::ArityMismatch {
            message: "random:seed expects 0 or 1 argument".to_string(),
        }),
    }
}
