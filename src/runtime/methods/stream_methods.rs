use super::super::value::{RuntimeError, StreamBackend, StreamHandle, Value};
use super::common::ValueRef;
use std::io::{BufRead, Read, Write};

/// Stream methods: read(chunk_kb=8), write(text), read_all(), read_lines(), close(), to_string()
pub fn call_stream_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match receiver.get() {
        Value::Stream(stream_handle) => {
            // Check if stream is closed first
            if stream_handle.is_closed.get() {
                return Err(RuntimeError::StreamError {
                    message: "Operation on closed stream".to_string(),
                });
            }

            match method {
                "read" => {
                    // Optional arg: chunk_kb (default 8)
                    let chunk_kb = match args.as_slice() {
                        [] => 8.0,
                        [Value::Number(n)] => *n,
                        _ => {
                            return Err(RuntimeError::ArityMismatch {
                                message: "stream::read(chunk_kb=8) takes 0 or 1 numeric argument"
                                    .to_string(),
                            });
                        }
                    };

                    if !chunk_kb.is_finite() || chunk_kb <= 0.0 {
                        return Err(RuntimeError::StreamError {
                            message: "chunk_kb must be a positive finite number".to_string(),
                        });
                    }

                    // Check if stream is readable
                    if !stream_handle.is_readable() {
                        return Err(RuntimeError::StreamError {
                            message: format!(
                                "Cannot read from write-only stream: {}",
                                stream_handle.name
                            ),
                        });
                    }

                    stream_read_chunk(stream_handle, (chunk_kb * 1024.0) as usize)
                }
                "write" => {
                    if args.len() != 1 {
                        return Err(RuntimeError::ArityMismatch {
                            message: "stream::write(text) expects exactly one argument".to_string(),
                        });
                    }

                    // Check if stream is writable
                    if !stream_handle.is_writable() {
                        return Err(RuntimeError::StreamError {
                            message: format!(
                                "Cannot write to read-only stream: {}",
                                stream_handle.name
                            ),
                        });
                    }

                    let text = args[0].to_string();
                    stream_write(stream_handle, &text)
                }
                "read_all" => {
                    if !args.is_empty() {
                        return Err(RuntimeError::ArityMismatch {
                            message: "stream::read_all() takes no arguments".to_string(),
                        });
                    }

                    // Check if stream is readable
                    if !stream_handle.is_readable() {
                        return Err(RuntimeError::StreamError {
                            message: format!(
                                "Cannot read from write-only stream: {}",
                                stream_handle.name
                            ),
                        });
                    }

                    stream_read_all(stream_handle)
                }
                "read_lines" => {
                    if !args.is_empty() {
                        return Err(RuntimeError::ArityMismatch {
                            message: "stream::read_lines() takes no arguments".to_string(),
                        });
                    }

                    // Check if stream is readable
                    if !stream_handle.is_readable() {
                        return Err(RuntimeError::StreamError {
                            message: format!(
                                "Cannot read from write-only stream: {}",
                                stream_handle.name
                            ),
                        });
                    }

                    stream_read_lines(stream_handle)
                }
                "close" => {
                    if !args.is_empty() {
                        return Err(RuntimeError::ArityMismatch {
                            message: "stream::close() takes no arguments".to_string(),
                        });
                    }
                    stream_handle.is_closed.set(true);
                    Ok(Value::Nil)
                }
                "to_string" => {
                    if !args.is_empty() {
                        return Err(RuntimeError::ArityMismatch {
                            message: "to_string() takes no arguments".to_string(),
                        });
                    }
                    Ok(Value::String(format!("<stream:{}>", stream_handle.name)))
                }
                _ => Err(RuntimeError::MethodError {
                    message: format!("Stream has no method '{}'", method),
                }),
            }
        }
        _ => unreachable!(),
    }
}

/// Read a chunk from a stream
fn stream_read_chunk(
    stream_handle: &StreamHandle,
    chunk_size: usize,
) -> Result<Value, RuntimeError> {
    let chunk_size = chunk_size.max(1); // Ensure at least 1 byte

    match &stream_handle.backend {
        StreamBackend::Stdin(reader_ref) => {
            let mut reader = reader_ref.borrow_mut();
            let mut buffer = vec![0u8; chunk_size];

            match reader.read(&mut buffer) {
                Ok(0) => Ok(Value::Nil), // EOF
                Ok(bytes_read) => {
                    buffer.truncate(bytes_read);
                    match String::from_utf8(buffer) {
                        Ok(text) => Ok(Value::String(text)),
                        Err(_) => Err(RuntimeError::StreamError {
                            message: "Stream read produced invalid UTF-8".to_string(),
                        }),
                    }
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read from stream: {}", e),
                }),
            }
        }
        #[cfg(test)]
        StreamBackend::TestReadable(cursor_ref) => {
            let mut cursor = cursor_ref.borrow_mut();
            let mut buffer = vec![0u8; chunk_size];

            match cursor.read(&mut buffer) {
                Ok(0) => Ok(Value::Nil), // EOF
                Ok(bytes_read) => {
                    buffer.truncate(bytes_read);
                    match String::from_utf8(buffer) {
                        Ok(text) => Ok(Value::String(text)),
                        Err(_) => Err(RuntimeError::StreamError {
                            message: "Stream read produced invalid UTF-8".to_string(),
                        }),
                    }
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read from stream: {}", e),
                }),
            }
        }
        _ => Err(RuntimeError::StreamError {
            message: format!("Cannot read from stream: {}", stream_handle.name),
        }),
    }
}

/// Write text to a stream
fn stream_write(stream_handle: &StreamHandle, text: &str) -> Result<Value, RuntimeError> {
    let bytes = text.as_bytes();

    match &stream_handle.backend {
        StreamBackend::Stdout(stdout_ref) => {
            let mut stdout = stdout_ref.borrow_mut();
            match stdout.write_all(bytes) {
                Ok(()) => {
                    stdout.flush().ok(); // Best effort flush
                    Ok(Value::Number(bytes.len() as f64))
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to write to stdout: {}", e),
                }),
            }
        }
        StreamBackend::Stderr(stderr_ref) => {
            let mut stderr = stderr_ref.borrow_mut();
            match stderr.write_all(bytes) {
                Ok(()) => {
                    stderr.flush().ok(); // Best effort flush
                    Ok(Value::Number(bytes.len() as f64))
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to write to stderr: {}", e),
                }),
            }
        }
        #[cfg(test)]
        StreamBackend::TestWritable(buffer_ref) => {
            let mut buffer = buffer_ref.borrow_mut();
            buffer.extend_from_slice(bytes);
            Ok(Value::Number(bytes.len() as f64))
        }
        _ => Err(RuntimeError::StreamError {
            message: format!("Cannot write to stream: {}", stream_handle.name),
        }),
    }
}

/// Read all content from a stream
fn stream_read_all(stream_handle: &StreamHandle) -> Result<Value, RuntimeError> {
    match &stream_handle.backend {
        StreamBackend::Stdin(reader_ref) => {
            let mut reader = reader_ref.borrow_mut();
            let mut content = String::new();

            match reader.read_to_string(&mut content) {
                Ok(_) => Ok(Value::String(content)),
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read from stream: {}", e),
                }),
            }
        }
        #[cfg(test)]
        StreamBackend::TestReadable(cursor_ref) => {
            let mut cursor = cursor_ref.borrow_mut();
            let mut content = String::new();

            match cursor.read_to_string(&mut content) {
                Ok(_) => Ok(Value::String(content)),
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read from stream: {}", e),
                }),
            }
        }
        _ => Err(RuntimeError::StreamError {
            message: format!("Cannot read from stream: {}", stream_handle.name),
        }),
    }
}

/// Read lines from a stream
fn stream_read_lines(stream_handle: &StreamHandle) -> Result<Value, RuntimeError> {
    match &stream_handle.backend {
        StreamBackend::Stdin(reader_ref) => {
            let mut reader = reader_ref.borrow_mut();
            let mut lines = Vec::new();

            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        // Remove trailing newline if present
                        if line.ends_with('\n') {
                            line.pop();
                            if line.ends_with('\r') {
                                line.pop();
                            }
                        }
                        lines.push(Value::String(line));
                    }
                    Err(e) => {
                        return Err(RuntimeError::StreamError {
                            message: format!("Failed to read line from stream: {}", e),
                        });
                    }
                }
            }

            Ok(Value::List(lines))
        }
        #[cfg(test)]
        StreamBackend::TestReadable(cursor_ref) => {
            let mut cursor = cursor_ref.borrow_mut();
            let mut lines = Vec::new();

            loop {
                let mut line = String::new();
                match cursor.read_line(&mut line) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        // Remove trailing newline if present
                        if line.ends_with('\n') {
                            line.pop();
                            if line.ends_with('\r') {
                                line.pop();
                            }
                        }
                        lines.push(Value::String(line));
                    }
                    Err(e) => {
                        return Err(RuntimeError::StreamError {
                            message: format!("Failed to read line from stream: {}", e),
                        });
                    }
                }
            }

            Ok(Value::List(lines))
        }
        _ => Err(RuntimeError::StreamError {
            message: format!("Cannot read from stream: {}", stream_handle.name),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::value::{StreamHandle, Value};
    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_stream_read_chunk() {
        let stream = Rc::new(StreamHandle::new_test_readable("hello world"));
        let receiver = ValueRef::Immutable(&Value::Stream(stream));

        // Read with default chunk size
        let result = call_stream_method(receiver, "read", vec![]).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_stream_write() {
        let stream = Rc::new(StreamHandle::new_test_writable());
        let stream_value = Value::Stream(stream.clone());
        let receiver = ValueRef::Immutable(&stream_value);

        // Write some text
        let result =
            call_stream_method(receiver, "write", vec![Value::String("hello".to_string())])
                .unwrap();
        assert_eq!(result, Value::Number(5.0)); // 5 bytes written

        // Check the output
        assert_eq!(stream.get_test_output(), Some("hello".to_string()));
    }

    #[test]
    fn test_stream_read_all() {
        let stream = Rc::new(StreamHandle::new_test_readable("line1\nline2\nline3"));
        let receiver = ValueRef::Immutable(&Value::Stream(stream));

        let result = call_stream_method(receiver, "read_all", vec![]).unwrap();
        assert_eq!(result, Value::String("line1\nline2\nline3".to_string()));
    }

    #[test]
    fn test_stream_read_lines() {
        let stream = Rc::new(StreamHandle::new_test_readable("line1\nline2\nline3"));
        let receiver = ValueRef::Immutable(&Value::Stream(stream));

        let result = call_stream_method(receiver, "read_lines", vec![]).unwrap();
        let expected = Value::List(vec![
            Value::String("line1".to_string()),
            Value::String("line2".to_string()),
            Value::String("line3".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_stream_close() {
        let stream = Rc::new(StreamHandle::new_test_readable("test"));
        let stream_value = Value::Stream(stream.clone());

        // Close the stream
        let receiver = ValueRef::Immutable(&stream_value);
        let result = call_stream_method(receiver, "close", vec![]).unwrap();
        assert_eq!(result, Value::Nil);
        assert!(stream.is_closed.get());

        // Try to read from closed stream
        let receiver = ValueRef::Immutable(&stream_value);
        let result = call_stream_method(receiver, "read", vec![]);
        assert!(matches!(result, Err(RuntimeError::StreamError { .. })));
    }

    #[test]
    fn test_stream_to_string() {
        let stream = Rc::new(StreamHandle::new_test_readable("test"));
        let receiver = ValueRef::Immutable(&Value::Stream(stream));

        let result = call_stream_method(receiver, "to_string", vec![]).unwrap();
        assert_eq!(result, Value::String("<stream:test_readable>".to_string()));
    }

    #[test]
    fn test_stream_read_write_restrictions() {
        // Test reading from write-only stream
        let write_stream = Rc::new(StreamHandle::new_test_writable());
        let write_value = Value::Stream(write_stream);
        let receiver = ValueRef::Immutable(&write_value);

        let result = call_stream_method(receiver, "read", vec![]);
        assert!(matches!(result, Err(RuntimeError::StreamError { .. })));

        // Test writing to read-only stream
        let read_stream = Rc::new(StreamHandle::new_test_readable("test"));
        let read_value = Value::Stream(read_stream);
        let receiver = ValueRef::Immutable(&read_value);

        let result = call_stream_method(receiver, "write", vec![Value::String("test".to_string())]);
        assert!(matches!(result, Err(RuntimeError::StreamError { .. })));
    }

    #[test]
    fn test_stream_arity_errors() {
        let stream = Rc::new(StreamHandle::new_test_readable("test"));
        let stream_value = Value::Stream(stream);

        // read with too many args
        let receiver = ValueRef::Immutable(&stream_value);
        let result = call_stream_method(
            receiver,
            "read",
            vec![Value::Number(1.0), Value::Number(2.0)],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        // write with no args
        let receiver = ValueRef::Immutable(&stream_value);
        let result = call_stream_method(receiver, "write", vec![]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        // read_all with args
        let receiver = ValueRef::Immutable(&stream_value);
        let result = call_stream_method(receiver, "read_all", vec![Value::Number(1.0)]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_stream_invalid_method() {
        let stream = Rc::new(StreamHandle::new_test_readable("test"));
        let stream_value = Value::Stream(stream);
        let receiver = ValueRef::Immutable(&stream_value);

        let result = call_stream_method(receiver, "invalid_method", vec![]);
        assert!(matches!(result, Err(RuntimeError::MethodError { .. })));
    }
}
