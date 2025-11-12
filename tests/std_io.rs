//! Integration tests for io streams.

use suji_interpreter::AstInterpreter;
use suji_runtime::{ModuleRegistry, call_builtin};
use suji_values::{DecimalNumber, MapKey, RuntimeError, Value};
use suji_values::{ValueRef, call_method};

#[test]
fn test_io_stream_method_calls() {
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut registry = ModuleRegistry::new();
    suji_stdlib::setup_module_registry(&mut registry);
    let executor = AstInterpreter;

    // Get io module from registry
    let io_module = registry
        .resolve_module_item(&executor, "std", "io")
        .unwrap();

    if let Value::Map(io_map) = io_module {
        let stdout_value = io_map.get(&MapKey::String("stdout".to_string())).unwrap();

        // Test to_string method
        let result =
            call_method(None, ValueRef::Immutable(stdout_value), "to_string", vec![]).unwrap();

        assert_eq!(result, Value::String("<stream:stdout>".to_string()));
    } else {
        panic!("io should be a map");
    }
}

#[test]
fn test_io_stream_write_method() {
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut registry = ModuleRegistry::new();
    suji_stdlib::setup_module_registry(&mut registry);
    let executor = AstInterpreter;

    // Get stdout stream from io module
    let stdout_value = registry
        .resolve_nested_module_item(&executor, "std:io", "stdout")
        .unwrap();

    // Test write method - this will actually write to stdout but should return byte count
    let result = call_method(
        None,
        ValueRef::Immutable(&stdout_value),
        "write",
        vec![Value::String("test".to_string())],
    )
    .unwrap();

    // Should return number of bytes written
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(4)));
}

#[test]
fn test_io_multiple_resolutions_share_streams() {
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut registry = ModuleRegistry::new();
    suji_stdlib::setup_module_registry(&mut registry);
    let executor = AstInterpreter;

    // Get stdout from two different resolution paths
    let stdout1 = registry
        .resolve_nested_module_item(&executor, "std:io", "stdout")
        .unwrap();
    let stdout2 = registry
        .resolve_nested_module_item(&executor, "std:io", "stdout")
        .unwrap();

    // Both should be stream proxy values (dynamic resolution)
    assert!(matches!(stdout1, Value::StreamProxy(_)));
    assert!(matches!(stdout2, Value::StreamProxy(_)));

    // They should be the same proxy kind
    if let (Value::StreamProxy(kind1), Value::StreamProxy(kind2)) = (&stdout1, &stdout2) {
        assert_eq!(kind1, kind2);
    }
}

#[test]
fn test_io_module_registry_resolution() {
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut registry = ModuleRegistry::new();
    suji_stdlib::setup_module_registry(&mut registry);
    let executor = AstInterpreter;

    // Test resolving io module directly
    let io_module = registry
        .resolve_module_item(&executor, "std", "io")
        .unwrap();
    assert!(matches!(io_module, Value::Map(_)));

    // Test resolving specific io items (now StreamProxy for dynamic resolution)
    let stdout = registry
        .resolve_nested_module_item(&executor, "std:io", "stdout")
        .unwrap();
    assert!(matches!(stdout, Value::StreamProxy(_)));

    let stderr = registry
        .resolve_nested_module_item(&executor, "std:io", "stderr")
        .unwrap();
    assert!(matches!(stderr, Value::StreamProxy(_)));

    let stdin = registry
        .resolve_nested_module_item(&executor, "std:io", "stdin")
        .unwrap();
    assert!(matches!(stdin, Value::StreamProxy(_)));

    let open_fn = registry
        .resolve_nested_module_item(&executor, "std:io", "open")
        .unwrap();
    assert!(matches!(open_fn, Value::Function(_)));
}

#[test]
fn test_io_error_handling() {
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut registry = ModuleRegistry::new();
    suji_stdlib::setup_module_registry(&mut registry);
    let executor = AstInterpreter;

    // Test resolving non-existent io item
    let result = registry.resolve_nested_module_item(&executor, "std:io", "nonexistent");
    assert!(result.is_err());

    // Test resolving from non-existent nested module
    let result = registry.resolve_nested_module_item(&executor, "std:NONEXISTENT", "stdout");
    assert!(result.is_err());
}

#[test]
fn test_io_open_default_no_create() {
    use std::fs;

    suji_stdlib::runtime::builtins::register_all_builtins();

    // Path to a file that definitely doesn't exist
    let nonexistent_path = "/tmp/suji_test_nonexistent_file_12345.txt";
    let _ = fs::remove_file(nonexistent_path); // Ensure it doesn't exist

    // Attempt to open with default args (create=false)
    let result = call_builtin("io_open", &[Value::String(nonexistent_path.to_string())]);

    // Should fail because file doesn't exist and create=false by default
    assert!(result.is_err());
    if let Err(RuntimeError::StreamError { message }) = result {
        assert!(message.contains("Failed to open"));
    } else {
        panic!("Expected StreamError");
    }
}

#[test]
fn test_io_open_create_true() {
    use std::fs;

    suji_stdlib::runtime::builtins::register_all_builtins();

    let test_path = "/tmp/suji_test_create_true.txt";
    let _ = fs::remove_file(test_path); // Clean up from any previous run

    // Open with create=true
    let result = call_builtin(
        "io_open",
        &[
            Value::String(test_path.to_string()),
            Value::Boolean(true),
            Value::Boolean(false),
        ],
    );

    assert!(result.is_ok());
    assert!(fs::metadata(test_path).is_ok());

    // Clean up
    let _ = fs::remove_file(test_path);
}

#[test]
fn test_io_open_truncate_with_create() {
    use std::fs;

    suji_stdlib::runtime::builtins::register_all_builtins();

    let test_path = "/tmp/suji_test_truncate_create.txt";

    // Create file with content
    fs::write(test_path, "initial content").unwrap();

    // Open with create=true, truncate=true
    let result = call_builtin(
        "io_open",
        &[
            Value::String(test_path.to_string()),
            Value::Boolean(true),
            Value::Boolean(true),
        ],
    );

    assert!(result.is_ok());

    // Write new content via stream
    if let Ok(Value::Stream(stream)) = result {
        let content = "new";
        let bytes_written = call_method(
            None,
            ValueRef::Immutable(&Value::Stream(stream.clone())),
            "write",
            vec![Value::String(content.to_string())],
        )
        .unwrap();
        assert_eq!(bytes_written, Value::Number(DecimalNumber::from_i64(3)));

        // Close the stream
        call_method(
            None,
            ValueRef::Immutable(&Value::Stream(stream)),
            "close",
            vec![],
        )
        .unwrap();
    }

    // Read file to verify truncation
    let contents = fs::read_to_string(test_path).unwrap();
    assert_eq!(contents, "new");

    // Clean up
    let _ = fs::remove_file(test_path);
}

#[test]
fn test_io_open_truncate_existing_without_create() {
    use std::fs;

    suji_stdlib::runtime::builtins::register_all_builtins();

    let test_path = "/tmp/suji_test_truncate_no_create.txt";

    // Create file with content
    fs::write(test_path, "old content").unwrap();

    // Open with create=false, truncate=true (should work since file exists)
    let result = call_builtin(
        "io_open",
        &[
            Value::String(test_path.to_string()),
            Value::Boolean(false),
            Value::Boolean(true),
        ],
    );

    assert!(result.is_ok());

    // File should be truncated (empty)
    if let Ok(Value::Stream(stream)) = result {
        // Close without writing
        call_method(
            None,
            ValueRef::Immutable(&Value::Stream(stream)),
            "close",
            vec![],
        )
        .unwrap();
    }

    let contents = fs::read_to_string(test_path).unwrap();
    assert_eq!(contents, "");

    // Clean up
    let _ = fs::remove_file(test_path);
}

#[test]
fn test_io_open_preserve_content() {
    use std::fs;

    suji_stdlib::runtime::builtins::register_all_builtins();

    let test_path = "/tmp/suji_test_preserve.txt";

    // Create file with content
    fs::write(test_path, "preserved").unwrap();

    // Open with create=false, truncate=false
    let result = call_builtin(
        "io_open",
        &[
            Value::String(test_path.to_string()),
            Value::Boolean(false),
            Value::Boolean(false),
        ],
    );

    assert!(result.is_ok());

    if let Ok(Value::Stream(stream)) = result {
        // Read content
        let content = call_method(
            None,
            ValueRef::Immutable(&Value::Stream(stream.clone())),
            "read_all",
            vec![],
        )
        .unwrap();

        assert_eq!(content, Value::String("preserved".to_string()));

        call_method(
            None,
            ValueRef::Immutable(&Value::Stream(stream)),
            "close",
            vec![],
        )
        .unwrap();
    }

    // Clean up
    let _ = fs::remove_file(test_path);
}

#[test]
fn test_io_open_arity_errors() {
    suji_stdlib::runtime::builtins::register_all_builtins();

    // No arguments
    let result = call_builtin("io_open", &[]);
    assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

    // Too many arguments
    let result = call_builtin(
        "io_open",
        &[
            Value::String("/tmp/test.txt".to_string()),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Boolean(false), // 4th arg - too many
        ],
    );
    assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
}

#[test]
fn test_io_open_type_errors() {
    suji_stdlib::runtime::builtins::register_all_builtins();

    // Non-string path
    let result = call_builtin("io_open", &[Value::Number(DecimalNumber::from_i64(123))]);
    assert!(matches!(result, Err(RuntimeError::TypeError { .. })));

    // Non-boolean create
    let result = call_builtin(
        "io_open",
        &[
            Value::String("/tmp/test.txt".to_string()),
            Value::Number(DecimalNumber::from_i64(1)),
        ],
    );
    assert!(matches!(result, Err(RuntimeError::TypeError { .. })));

    // Non-boolean truncate
    let result = call_builtin(
        "io_open",
        &[
            Value::String("/tmp/test.txt".to_string()),
            Value::Boolean(true),
            Value::String("false".to_string()),
        ],
    );
    assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
}
