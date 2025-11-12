// No executor needed
use super::super::io_context::IoContext;
use super::super::value::{DecimalNumber, RuntimeError, StreamBackend, StreamHandle, Value};
use super::common::ValueRef;
use std::io::IsTerminal;
use std::io::{BufRead, Read, Write};
use std::rc::Rc;

/// Resolve a Value to a concrete StreamHandle (handles both Stream and StreamProxy)
fn resolve_stream(value: &Value) -> Result<Rc<StreamHandle>, RuntimeError> {
    match value {
        Value::Stream(handle) => Ok(handle.clone()),
        Value::StreamProxy(kind) => {
            // Resolve proxy dynamically from IO context
            use super::super::value::StreamProxyKind;
            Ok(match kind {
                StreamProxyKind::Stdin => IoContext::effective_stdin(),
                StreamProxyKind::Stdout => IoContext::effective_stdout(),
                StreamProxyKind::Stderr => IoContext::effective_stderr(),
            })
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("Expected stream, got {}", value.type_name()),
        }),
    }
}

/// Stream methods: read(chunk_kb=8), write(text), read_all(), read_lines(), close(), to_string()
pub fn call_stream_method(
    receiver: ValueRef,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    // Resolve the stream (handles both Stream and StreamProxy)
    let stream_handle = resolve_stream(receiver.get())?;

    // Now operate on the resolved stream
    {
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
                    [] => DecimalNumber::from_i64(8),
                    [Value::Number(n)] => n.clone(),
                    _ => {
                        return Err(RuntimeError::ArityMismatch {
                            message: "stream::read(chunk_kb=8) takes 0 or 1 numeric argument"
                                .to_string(),
                        });
                    }
                };

                let zero = DecimalNumber::from_i64(0);
                if chunk_kb <= zero {
                    return Err(RuntimeError::StreamError {
                        message: "chunk_kb must be a positive number".to_string(),
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

                // Convert chunk_kb to bytes (multiply by 1024)
                let kb_1024 = DecimalNumber::from_i64(1024);
                let chunk_bytes = chunk_kb.mul(&kb_1024);
                let chunk_bytes_i64 =
                    chunk_bytes
                        .to_i64_checked()
                        .ok_or_else(|| RuntimeError::StreamError {
                            message: "chunk size too large".to_string(),
                        })?;
                stream_read_chunk(&stream_handle, chunk_bytes_i64 as usize)
            }
            "read_line" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "stream::read_line() takes no arguments".to_string(),
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

                stream_read_line(&stream_handle)
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
                stream_write(&stream_handle, &text)
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

                stream_read_all(&stream_handle)
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

                stream_read_lines(&stream_handle)
            }
            "is_terminal" => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        message: "stream::is_terminal() takes no arguments".to_string(),
                    });
                }

                let is_tty = match &stream_handle.backend {
                    StreamBackend::Stdin(reader_ref) => {
                        let reader = reader_ref.borrow();
                        reader.get_ref().is_terminal()
                    }
                    StreamBackend::Stdout(stdout_ref) => {
                        let stdout = stdout_ref.borrow();
                        stdout.is_terminal()
                    }
                    StreamBackend::Stderr(stderr_ref) => {
                        let stderr = stderr_ref.borrow();
                        stderr.is_terminal()
                    }
                    _ => false,
                };

                Ok(Value::Boolean(is_tty))
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
        StreamBackend::File(file_ref) => {
            let mut file = file_ref.borrow_mut();
            let mut buffer = vec![0u8; chunk_size];

            match file.read(&mut buffer) {
                Ok(0) => Ok(Value::Nil),
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
        StreamBackend::MemoryReadable(cursor_ref) => {
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

/// Read a single line from a stream
fn stream_read_line(stream_handle: &StreamHandle) -> Result<Value, RuntimeError> {
    match &stream_handle.backend {
        StreamBackend::Stdin(reader_ref) => {
            let mut reader = reader_ref.borrow_mut();
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => Ok(Value::Nil),
                Ok(_) => {
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Ok(Value::String(line))
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read line from stream: {}", e),
                }),
            }
        }
        StreamBackend::File(file_ref) => {
            let mut file = file_ref.borrow_mut();
            let mut buffer = Vec::new();
            let mut byte = [0u8; 1];

            loop {
                match file.read(&mut byte) {
                    Ok(0) => {
                        if buffer.is_empty() {
                            return Ok(Value::Nil);
                        } else {
                            break;
                        }
                    }
                    Ok(_) => {
                        buffer.push(byte[0]);
                        if byte[0] == b'\n' {
                            break;
                        }
                    }
                    Err(e) => {
                        return Err(RuntimeError::StreamError {
                            message: format!("Failed to read line from stream: {}", e),
                        });
                    }
                }
            }

            if buffer.last().is_some_and(|last| *last == b'\n') {
                buffer.pop();
                if buffer.last().is_some_and(|carriage| *carriage == b'\r') {
                    buffer.pop();
                }
            }

            match String::from_utf8(buffer) {
                Ok(line) => Ok(Value::String(line)),
                Err(_) => Err(RuntimeError::StreamError {
                    message: "Stream read produced invalid UTF-8".to_string(),
                }),
            }
        }
        StreamBackend::MemoryReadable(cursor_ref) => {
            let mut cursor = cursor_ref.borrow_mut();
            let mut line = String::new();
            match cursor.read_line(&mut line) {
                Ok(0) => Ok(Value::Nil),
                Ok(_) => {
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Ok(Value::String(line))
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read line from stream: {}", e),
                }),
            }
        }
        #[cfg(test)]
        StreamBackend::TestReadable(cursor_ref) => {
            let mut cursor = cursor_ref.borrow_mut();
            let mut line = String::new();
            match cursor.read_line(&mut line) {
                Ok(0) => Ok(Value::Nil),
                Ok(_) => {
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Ok(Value::String(line))
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read line from stream: {}", e),
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
                    Ok(Value::Number(DecimalNumber::from_usize(bytes.len())))
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
                    Ok(Value::Number(DecimalNumber::from_usize(bytes.len())))
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to write to stderr: {}", e),
                }),
            }
        }
        StreamBackend::File(file_ref) => {
            let mut file = file_ref.borrow_mut();
            match file.write_all(bytes) {
                Ok(()) => {
                    file.flush().ok();
                    Ok(Value::Number(DecimalNumber::from_usize(bytes.len())))
                }
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to write to stream: {}", e),
                }),
            }
        }
        StreamBackend::MemoryWritable(buffer_ref) => {
            let mut buffer = buffer_ref.borrow_mut();
            buffer.extend_from_slice(bytes);
            Ok(Value::Number(DecimalNumber::from_usize(bytes.len())))
        }
        #[cfg(test)]
        StreamBackend::TestWritable(buffer_ref) => {
            let mut buffer = buffer_ref.borrow_mut();
            buffer.extend_from_slice(bytes);
            Ok(Value::Number(DecimalNumber::from_usize(bytes.len())))
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
        StreamBackend::File(file_ref) => {
            let mut file = file_ref.borrow_mut();
            let mut content = String::new();

            match file.read_to_string(&mut content) {
                Ok(_) => Ok(Value::String(content)),
                Err(e) => Err(RuntimeError::StreamError {
                    message: format!("Failed to read from stream: {}", e),
                }),
            }
        }
        StreamBackend::MemoryReadable(cursor_ref) => {
            let mut cursor = cursor_ref.borrow_mut();
            let mut content = String::new();

            match cursor.read_to_string(&mut content) {
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
        StreamBackend::File(_) => {
            let mut lines = Vec::new();

            loop {
                match stream_read_line(stream_handle)? {
                    Value::Nil => break,
                    Value::String(line) => lines.push(Value::String(line)),
                    _ => {
                        return Err(RuntimeError::StreamError {
                            message: "Unexpected value while reading lines".to_string(),
                        });
                    }
                }
            }

            Ok(Value::List(lines))
        }
        StreamBackend::MemoryReadable(cursor_ref) => {
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
        assert_eq!(result, Value::Number(DecimalNumber::from_i64(5))); // 5 bytes written

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
    fn test_stream_read_line_basic() {
        let stream = Rc::new(StreamHandle::new_test_readable("a\nb\n"));
        let stream_value = Value::Stream(stream);

        let receiver = ValueRef::Immutable(&stream_value);
        let r1 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r1, Value::String("a".to_string()));

        let receiver = ValueRef::Immutable(&stream_value);
        let r2 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r2, Value::String("b".to_string()));

        let receiver = ValueRef::Immutable(&stream_value);
        let r3 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r3, Value::Nil);
    }

    #[test]
    fn test_stream_read_line_crlf_and_edge_cases() {
        let stream = Rc::new(StreamHandle::new_test_readable("x\r\ny\r\n\nlast"));
        let stream_value = Value::Stream(stream);

        // x\r\n -> x
        let receiver = ValueRef::Immutable(&stream_value);
        let r1 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r1, Value::String("x".to_string()));

        // y\r\n -> y
        let receiver = ValueRef::Immutable(&stream_value);
        let r2 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r2, Value::String("y".to_string()));

        // \n -> empty line
        let receiver = ValueRef::Immutable(&stream_value);
        let r3 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r3, Value::String("".to_string()));

        // last (no trailing newline)
        let receiver = ValueRef::Immutable(&stream_value);
        let r4 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r4, Value::String("last".to_string()));

        // EOF -> nil
        let receiver = ValueRef::Immutable(&stream_value);
        let r5 = call_stream_method(receiver, "read_line", vec![]).unwrap();
        assert_eq!(r5, Value::Nil);
    }

    #[test]
    fn test_stream_read_line_arity_error() {
        let stream = Rc::new(StreamHandle::new_test_readable("test"));
        let stream_value = Value::Stream(stream);
        let receiver = ValueRef::Immutable(&stream_value);

        let result = call_stream_method(
            receiver,
            "read_line",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_stream_is_terminal_on_test_backends_is_false() {
        // Readable test backend
        let read_stream = Rc::new(StreamHandle::new_test_readable("data"));
        let read_value = Value::Stream(read_stream);
        let receiver_r = ValueRef::Immutable(&read_value);
        let r = call_stream_method(receiver_r, "is_terminal", vec![]).unwrap();
        assert_eq!(r, Value::Boolean(false));

        // Writable test backend
        let write_stream = Rc::new(StreamHandle::new_test_writable());
        let write_value = Value::Stream(write_stream);
        let receiver_w = ValueRef::Immutable(&write_value);
        let w = call_stream_method(receiver_w, "is_terminal", vec![]).unwrap();
        assert_eq!(w, Value::Boolean(false));
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
            vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::Number(DecimalNumber::from_i64(2)),
            ],
        );
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        // write with no args
        let receiver = ValueRef::Immutable(&stream_value);
        let result = call_stream_method(receiver, "write", vec![]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        // read_all with args
        let receiver = ValueRef::Immutable(&stream_value);
        let result = call_stream_method(
            receiver,
            "read_all",
            vec![Value::Number(DecimalNumber::from_i64(1))],
        );
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
