use suji_values::value::{RuntimeError, Value};

pub fn one_string_arg<'a>(args: &'a [Value], fname: &str) -> Result<&'a str, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: format!("{} expects 1 argument", fname),
        });
    }
    match &args[0] {
        Value::String(s) => Ok(s.as_str()),
        _ => Err(RuntimeError::TypeError {
            message: "argument must be string".to_string(),
        }),
    }
}
