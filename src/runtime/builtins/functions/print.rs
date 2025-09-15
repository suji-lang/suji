//! Print builtin function implementation
//!
//! This module contains the implementation of the print builtin function.

use crate::runtime::value::{RuntimeError, Value};

/// print function - prints values to stdout without newline
pub fn builtin_print(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        print!("");
    } else {
        let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
        print!("{}", output.join(" "));
    }
    Ok(Value::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_builtin() {
        // Test print with no arguments
        assert_eq!(builtin_print(&[]).unwrap(), Value::Nil);

        // Test print with one argument
        let args = vec![Value::String("Hello".to_string())];
        assert_eq!(builtin_print(&args).unwrap(), Value::Nil);

        // Test print with multiple arguments
        let args = vec![
            Value::String("Hello".to_string()),
            Value::Number(42.0),
            Value::Boolean(true),
        ];
        assert_eq!(builtin_print(&args).unwrap(), Value::Nil);
    }
}
