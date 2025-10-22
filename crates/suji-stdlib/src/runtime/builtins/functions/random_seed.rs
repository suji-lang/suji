use crate::runtime::builtins::random::rng_seed_with;
use suji_runtime::value::{RuntimeError, Value};

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
                if let Some(seed) = n.to_i64_checked() {
                    rng_seed_with(Some(seed as u64));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "random:seed number must be an integer within valid range"
                            .to_string(),
                    });
                }
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
