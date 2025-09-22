//! Built-in: print(text, out=std:FD:stdout) -> bytes written.

use crate::runtime::methods::{ValueRef, call_method};
use crate::runtime::value::{RuntimeError, Value};
use std::rc::Rc;

/// Write text to a stream; return bytes written.
pub fn builtin_print(args: &[Value]) -> Result<Value, RuntimeError> {
    // Expect 1 or 2 args: text, optional out stream
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "print(text, out=stdout) expects 1 or 2 arguments".to_string(),
        });
    }

    // Text to write
    let text_value = &args[0];
    let mut text_str = text_value.to_string();

    // Determine output stream: provided or default to stdout
    let out_value = if args.len() == 2 {
        match &args[1] {
            Value::Nil => Value::Stream(Rc::new(crate::runtime::value::StreamHandle::new_stdout())),
            v @ Value::Stream(_) => v.clone(),
            other => {
                return Err(RuntimeError::TypeError {
                    message: format!(
                        "Output parameter must be a stream, got {}",
                        other.type_name()
                    ),
                });
            }
        }
    } else {
        // Default to stdout
        Value::Stream(Rc::new(crate::runtime::value::StreamHandle::new_stdout()))
    };

    // Call stream::write(text)
    let write_args = vec![Value::String(std::mem::take(&mut text_str))];
    call_method(ValueRef::Immutable(&out_value), "write", write_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_returns_bytes_written_default_stdout() {
        let args = vec![Value::String("Hello".to_string())];
        let result = builtin_print(&args).unwrap();
        // UTF-8 byte length of "Hello"
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_print_with_custom_stream() {
        // Use test writable stream to capture output
        let out = Value::Stream(Rc::new(
            crate::runtime::value::StreamHandle::new_test_writable(),
        ));
        let args = vec![Value::String("abc".to_string()), out.clone()];
        let result = builtin_print(&args).unwrap();
        assert_eq!(result, Value::Number(3.0));

        if let Value::Stream(handle) = out {
            assert_eq!(handle.get_test_output().unwrap(), "abc".to_string());
        } else {
            panic!("expected stream");
        }
    }
}
