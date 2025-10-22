use crate::runtime::builtins::math::{from_decimal, to_decimal};
use rust_decimal::MathematicalOps;
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_math_exp(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "math:exp expects 1 argument".to_string(),
        });
    }
    let x = to_decimal(&args[0], "x")?;
    let y = x
        .checked_exp()
        .ok_or_else(|| RuntimeError::InvalidOperation {
            message: "math result overflow".to_string(),
        })?;
    Ok(from_decimal(y))
}
