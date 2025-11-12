//! Built-in `std:io:open` implementation.

use std::cell::Cell;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::rc::Rc;

use suji_values::value::{RuntimeError, StreamBackend, StreamHandle, Value};

/// Open a file as a stream for reading and writing (binary mode).
/// Signature: `io:open(path, create = false, truncate = false)`
pub fn builtin_io_open(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(RuntimeError::ArityMismatch {
            message: format!(
                "std:io:open(path[, create[, truncate]]) expects 1 to 3 arguments, got {}",
                args.len()
            ),
        });
    }

    let path = match &args[0] {
        Value::String(path) => path.clone(),
        other => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "std:io:open(path, create, truncate) expects path to be a string, got {}",
                    other.type_name()
                ),
            });
        }
    };

    let create = if args.len() >= 2 {
        match &args[1] {
            Value::Boolean(b) => *b,
            other => {
                return Err(RuntimeError::TypeError {
                    message: format!(
                        "std:io:open(path, create, truncate) expects create to be a boolean, got {}",
                        other.type_name()
                    ),
                });
            }
        }
    } else {
        false
    };

    let truncate = if args.len() >= 3 {
        match &args[2] {
            Value::Boolean(b) => *b,
            other => {
                return Err(RuntimeError::TypeError {
                    message: format!(
                        "std:io:open(path, create, truncate) expects truncate to be a boolean, got {}",
                        other.type_name()
                    ),
                });
            }
        }
    } else {
        false
    };

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .truncate(truncate)
        .open(&path)
        .map_err(|err| RuntimeError::StreamError {
            message: format!("Failed to open '{}': {}", path, err),
        })?;

    let handle = StreamHandle {
        backend: StreamBackend::File(RefCell::new(file)),
        is_closed: Cell::new(false),
        name: path,
    };

    Ok(Value::Stream(Rc::new(handle)))
}
