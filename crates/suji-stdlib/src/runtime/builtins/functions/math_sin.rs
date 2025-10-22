use crate::runtime::builtins::math::{from_decimal, to_decimal};
use rust_decimal::MathematicalOps;
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_math_sin(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "math:sin expects 1 argument".to_string(),
        });
    }
    let x = to_decimal(&args[0], "x")?;
    let y = x
        .checked_sin()
        .ok_or_else(|| RuntimeError::InvalidOperation {
            message: "math:sin overflow".to_string(),
        })?;
    Ok(from_decimal(y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::ToPrimitive;
    #[test]
    fn test_sin_zero() {
        let v = builtin_math_sin(
            &[Value::Number(suji_runtime::value::DecimalNumber::from_i64(
                0,
            ))],
        )
        .unwrap();
        if let Value::Number(n) = v {
            assert!(n.inner().to_f64().unwrap().abs() < 1e-12);
        } else {
            panic!()
        }
    }
}
