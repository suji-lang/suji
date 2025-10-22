use crate::runtime::builtins::math::{from_f64, to_f64};
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_math_atan(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "math:atan expects 1 argument".to_string(),
        });
    }
    from_f64(to_f64(&args[0], "x")?.atan())
}
