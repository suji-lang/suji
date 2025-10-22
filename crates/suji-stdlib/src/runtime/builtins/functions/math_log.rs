use crate::runtime::builtins::math::{ensure_positive_decimal, from_decimal, to_decimal};
use rust_decimal::MathematicalOps;
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_math_log(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "math:log expects 1 argument".to_string(),
        });
    }
    let x = to_decimal(&args[0], "x")?;
    ensure_positive_decimal(x, "log")?;
    let y = x
        .checked_ln()
        .ok_or_else(|| RuntimeError::InvalidOperation {
            message: "log domain is (0, +inf)".to_string(),
        })?;
    Ok(from_decimal(y))
}
