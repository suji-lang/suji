use crate::runtime::builtins::math::{from_f64, to_f64};
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_math_atan2(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "math:atan2 expects 2 arguments".to_string(),
        });
    }
    let y = to_f64(&args[0], "y")?;
    let x = to_f64(&args[1], "x")?;
    from_f64(y.atan2(x))
}
