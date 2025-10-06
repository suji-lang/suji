//! Integration tests for io streams.

use suji_lang::runtime::methods::{ValueRef, call_method};
use suji_lang::runtime::module::ModuleRegistry;
use suji_lang::runtime::value::{DecimalNumber, MapKey, Value};

#[test]
fn test_io_stream_method_calls() {
    let registry = ModuleRegistry::new();

    // Get io module from registry
    let io_module = registry.resolve_module_item("std", "io").unwrap();

    if let Value::Map(io_map) = io_module {
        let stdout_value = io_map.get(&MapKey::String("stdout".to_string())).unwrap();

        // Test to_string method
        let result = call_method(ValueRef::Immutable(stdout_value), "to_string", vec![]).unwrap();

        assert_eq!(result, Value::String("<stream:stdout>".to_string()));
    } else {
        panic!("io should be a map");
    }
}

#[test]
fn test_io_stream_write_method() {
    let registry = ModuleRegistry::new();

    // Get stdout stream from io module
    let stdout_value = registry
        .resolve_nested_module_item("std:io", "stdout")
        .unwrap();

    // Test write method - this will actually write to stdout but should return byte count
    let result = call_method(
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
    let registry = ModuleRegistry::new();

    // Get stdout from two different resolution paths
    let stdout1 = registry
        .resolve_nested_module_item("std:io", "stdout")
        .unwrap();
    let stdout2 = registry
        .resolve_nested_module_item("std:io", "stdout")
        .unwrap();

    // Both should be stream values
    assert!(matches!(stdout1, Value::Stream(_)));
    assert!(matches!(stdout2, Value::Stream(_)));

    // They should have the same properties (shared backend)
    if let (Value::Stream(handle1), Value::Stream(handle2)) = (&stdout1, &stdout2) {
        assert_eq!(handle1.name, handle2.name);
        assert_eq!(handle1.is_readable(), handle2.is_readable());
        assert_eq!(handle1.is_writable(), handle2.is_writable());
        assert_eq!(handle1.is_closed.get(), handle2.is_closed.get());
    }
}

#[test]
fn test_io_module_registry_resolution() {
    let registry = ModuleRegistry::new();

    // Test resolving io module directly
    let io_module = registry.resolve_module_item("std", "io").unwrap();
    assert!(matches!(io_module, Value::Map(_)));

    // Test resolving specific io items
    let stdout = registry
        .resolve_nested_module_item("std:io", "stdout")
        .unwrap();
    assert!(matches!(stdout, Value::Stream(_)));

    let stderr = registry
        .resolve_nested_module_item("std:io", "stderr")
        .unwrap();
    assert!(matches!(stderr, Value::Stream(_)));

    let stdin = registry
        .resolve_nested_module_item("std:io", "stdin")
        .unwrap();
    assert!(matches!(stdin, Value::Stream(_)));

    let open_fn = registry
        .resolve_nested_module_item("std:io", "open")
        .unwrap();
    assert!(matches!(open_fn, Value::Function(_)));
}

#[test]
fn test_io_error_handling() {
    let registry = ModuleRegistry::new();

    // Test resolving non-existent io item
    let result = registry.resolve_nested_module_item("std:io", "nonexistent");
    assert!(result.is_err());

    // Test resolving from non-existent nested module
    let result = registry.resolve_nested_module_item("std:NONEXISTENT", "stdout");
    assert!(result.is_err());
}
