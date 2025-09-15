use nnlang::runtime::methods::{ValueRef, call_method};
use nnlang::runtime::value::Value;

/// Integration tests for the methods module
/// These tests verify that the main dispatcher correctly routes to individual method implementations
/// and that cross-module functionality works correctly.

#[test]
fn test_immutable_mutating_method_error() {
    let list = Value::List(vec![Value::Number(1.0)]);
    let receiver = ValueRef::Immutable(&list);

    let result = call_method(receiver, "push", vec![Value::Number(2.0)]);
    assert!(matches!(
        result,
        Err(nnlang::runtime::value::RuntimeError::MethodError { .. })
    ));
}

#[test]
fn test_method_dispatcher_routing() {
    // Test that the dispatcher correctly routes to string methods
    let s = Value::String("hello".to_string());
    let receiver = ValueRef::Immutable(&s);
    let result = call_method(receiver, "length", vec![]).unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Test that the dispatcher correctly routes to number methods
    let n = Value::Number(42.0);
    let receiver = ValueRef::Immutable(&n);
    let result = call_method(receiver, "to_string", vec![]).unwrap();
    assert_eq!(result, Value::String("42".to_string()));

    // Test that the dispatcher correctly routes to list methods
    let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
    let receiver = ValueRef::Immutable(&list);
    let result = call_method(receiver, "length", vec![]).unwrap();
    assert_eq!(result, Value::Number(2.0));

    // Test that the dispatcher correctly routes to map methods
    use indexmap::IndexMap;
    use nnlang::runtime::value::MapKey;
    let mut map_data = IndexMap::new();
    map_data.insert(
        MapKey::String("key".to_string()),
        Value::String("value".to_string()),
    );
    let map = Value::Map(map_data);
    let receiver = ValueRef::Immutable(&map);
    let result = call_method(receiver, "length", vec![]).unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_unknown_method_error() {
    let s = Value::String("hello".to_string());
    let receiver = ValueRef::Immutable(&s);
    let result = call_method(receiver, "unknown_method", vec![]);
    assert!(matches!(
        result,
        Err(nnlang::runtime::value::RuntimeError::MethodError { .. })
    ));
}

#[test]
fn test_unsupported_type_method_error() {
    let boolean = Value::Boolean(true);
    let receiver = ValueRef::Immutable(&boolean);
    let result = call_method(receiver, "length", vec![]);
    assert!(matches!(
        result,
        Err(nnlang::runtime::value::RuntimeError::MethodError { .. })
    ));
}
