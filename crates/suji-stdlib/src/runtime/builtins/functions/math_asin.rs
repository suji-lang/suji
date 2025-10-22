use crate::runtime::builtins::math::{ensure_in_unit_interval, from_f64, to_f64};
use suji_runtime::value::{RuntimeError, Value};

pub fn builtin_math_asin(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "math:asin expects 1 argument".to_string(),
        });
    }
    let x = to_f64(&args[0], "x")?;
    ensure_in_unit_interval(x, "asin")?;
    from_f64(x.asin())
}
