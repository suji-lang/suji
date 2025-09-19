//! Println builtin function implementation
//!
//! Implements std:println(text, out = std:FD:stdout) -> number of bytes written

use crate::runtime::builtins::functions::print::builtin_print;
use crate::runtime::value::{RuntimeError, Value};

/// println function - writes text followed by newline to a stream
/// Signature: println(text, out = std:FD:stdout)
pub fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "println(text, out=stdout) expects 1 or 2 arguments".to_string(),
        });
    }

    // Build arguments for print: text + "\n", optional out stream
    let mut new_args = Vec::with_capacity(2);
    let mut s = args[0].to_string();
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
}
