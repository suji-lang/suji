//! Built-in `std:io:open` implementation.

use std::cell::Cell;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::rc::Rc;

use crate::runtime::value::{RuntimeError, StreamBackend, StreamHandle, Value};

/// Open a file as a stream for reading and writing (binary mode).
pub fn builtin_io_open(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            message: "std:io:open(path) expects exactly one argument".to_string(),
        });
    }

    let path = match &args[0] {
        Value::String(path) => path.clone(),
        other => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "std:io:open(path) expects a string path, got {}",
                    other.type_name()
                ),
            });
        }
    };

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
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
