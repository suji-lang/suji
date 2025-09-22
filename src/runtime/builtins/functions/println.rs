//! Built-in: println(text="", out=std:FD:stdout) -> bytes written.

use crate::runtime::builtins::functions::print::builtin_print;
use crate::runtime::value::{RuntimeError, Value};

/// Write text + newline to a stream; return bytes written.
pub fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "println(text=\"\", out=stdout) expects 0, 1, or 2 arguments".to_string(),
        });
    }

    // Build arguments for print: text + "\n", optional out stream
    let mut new_args = Vec::with_capacity(2);
    let mut s = if args.is_empty() {
        String::new()
    } else {
        args[0].to_string()
    };
    s.push('\n');
    new_args.push(Value::String(s));
    if args.len() == 2 {
        new_args.push(args[1].clone());
    }

    builtin_print(&new_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_println_returns_bytes_written_default_stdout() {
        let args = vec![Value::String("Hi".to_string())];
        let result = builtin_println(&args).unwrap();
        // "Hi\n" is 3 bytes
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_println_no_args_returns_newline_bytes() {
        let result = builtin_println(&[]).unwrap();
        // "\n" is 1 byte
        assert_eq!(result, Value::Number(1.0));
    }
}
