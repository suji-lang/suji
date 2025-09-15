//! Println builtin function implementation
//!
//! This module contains the implementation of the println builtin function.

use crate::runtime::value::{RuntimeError, Value};

/// println function - prints values to stdout with newline
pub fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        println!();
    } else {
        let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
        println!("{}", output.join(" "));
    }
    Ok(Value::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_println_builtin() {
        // Test println with no arguments
        assert_eq!(builtin_println(&[]).unwrap(), Value::Nil);

        // Test println with one argument
        let args = vec![Value::String("Hello".to_string())];
        assert_eq!(builtin_println(&args).unwrap(), Value::Nil);

        // Test println with multiple arguments
        let args = vec![
            Value::String("Hello".to_string()),
            Value::Number(42.0),
            Value::Boolean(true),
        ];
        assert_eq!(builtin_println(&args).unwrap(), Value::Nil);
    }
}
