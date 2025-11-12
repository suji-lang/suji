//! Built-in: os:exit(code) -> (terminates process, never returns).

use suji_values::value::{RuntimeError, Value};

/// Terminates the process with the given exit code.
pub fn builtin_os_exit(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "os:exit() takes exactly one argument".to_string(),
        });
    }

    let code = match &args[0] {
        Value::Number(n) => {
            let code_i64 = n.to_i64_checked().ok_or_else(|| RuntimeError::TypeError {
                message: "os:exit() requires an integer exit code".to_string(),
            })?;

            if code_i64 < 0 {
                return Err(RuntimeError::TypeError {
                    message: "os:exit() requires a non-negative exit code".to_string(),
                });
            }

            // Convert i64 to i32, clamping to i32 range if needed
            if code_i64 > i32::MAX as i64 {
                i32::MAX
            } else {
                code_i64 as i32
            }
        }
        _ => {
            return Err(RuntimeError::TypeError {
                message: "os:exit() argument must be a number".to_string(),
            });
        }
    };

    std::process::exit(code);
}
