use suji_values::DecimalNumber;
use suji_values::Value;
use suji_values::{ValueRef, call_method};

/// Integration tests for the methods module
/// These tests verify that the main dispatcher correctly routes to individual method implementations
/// and that cross-module functionality works correctly.

#[test]
fn test_immutable_mutating_method_error() {
    let list = Value::List(vec![Value::Number(DecimalNumber::from_i64(1))]);
    let receiver = ValueRef::Immutable(&list);

    let result = call_method(
        None,
        receiver,
        "push",
        vec![Value::Number(DecimalNumber::from_i64(2))],
    );
    assert!(matches!(
        result,
        Err(suji_values::RuntimeError::MethodError { .. })
    ));
}

#[test]
fn test_method_dispatcher_routing() {
    // Test that the dispatcher correctly routes to string methods
    let s = Value::String("hello".to_string());
    let receiver = ValueRef::Immutable(&s);
    let result = call_method(None, receiver, "length", vec![]).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

    // Test that the dispatcher correctly routes to number methods
    let n = Value::Number(DecimalNumber::from_i64(42));
    let receiver = ValueRef::Immutable(&n);
    let result = call_method(None, receiver, "to_string", vec![]).unwrap();
    assert_eq!(result, Value::String("42".to_string()));

    // Test that the dispatcher correctly routes to list methods
    let list = Value::List(vec![
        Value::Number(DecimalNumber::from_i64(1)),
        Value::Number(DecimalNumber::from_i64(2)),
    ]);
    let receiver = ValueRef::Immutable(&list);
    let result = call_method(None, receiver, "length", vec![]).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    // Test that the dispatcher correctly routes to map methods
    use indexmap::IndexMap;
    use suji_values::MapKey;
    let mut map_data = IndexMap::new();
    map_data.insert(
        MapKey::String("key".to_string()),
        Value::String("value".to_string()),
    );
    let map = Value::Map(map_data);
    let receiver = ValueRef::Immutable(&map);
    let result = call_method(None, receiver, "length", vec![]).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));
}

#[test]
fn test_unknown_method_error() {
    let s = Value::String("hello".to_string());
    let receiver = ValueRef::Immutable(&s);
    let result = call_method(None, receiver, "unknown_method", vec![]);
    assert!(matches!(
        result,
        Err(suji_values::RuntimeError::MethodError { .. })
    ));
}

#[test]
fn test_unsupported_type_method_error() {
    let boolean = Value::Boolean(true);
    let receiver = ValueRef::Immutable(&boolean);
    let result = call_method(None, receiver, "length", vec![]);
    assert!(matches!(
        result,
        Err(suji_values::RuntimeError::MethodError { .. })
    ));
}
