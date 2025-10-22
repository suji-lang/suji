use crate::runtime::builtins::random::rng_f64;
use suji_runtime::value::{DecimalNumber, RuntimeError, Value};

pub fn builtin_random_random(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            message: "random() expects 0 arguments".to_string(),
        });
    }
    // Convert f64 to decimal string and then parse for precision
    let random_f64 = rng_f64();
    let random_str = random_f64.to_string();
    match DecimalNumber::parse(&random_str) {
        Ok(decimal) => Ok(Value::Number(decimal)),
        Err(_) => Ok(Value::Number(DecimalNumber::from_i64(0))), // fallback to 0
    }
}
