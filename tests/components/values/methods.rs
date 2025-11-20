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

#[test]
fn test_type_checking_methods() {
    // Test is_number()
    let n = Value::Number(DecimalNumber::from_i64(42));
    let receiver = ValueRef::Immutable(&n);
    assert_eq!(
        call_method(None, receiver, "is_number", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_string", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_bool", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_list", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_map", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_tuple", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_function", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_stream", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&n), "is_regex", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_string()
    let s = Value::String("hello".to_string());
    assert_eq!(
        call_method(None, ValueRef::Immutable(&s), "is_string", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&s), "is_number", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_bool()
    let b = Value::Boolean(true);
    assert_eq!(
        call_method(None, ValueRef::Immutable(&b), "is_bool", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&b), "is_number", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_list()
    let list = Value::List(vec![Value::Number(DecimalNumber::from_i64(1))]);
    assert_eq!(
        call_method(None, ValueRef::Immutable(&list), "is_list", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&list), "is_tuple", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_map()
    use indexmap::IndexMap;
    use suji_values::MapKey;
    let mut map_data = IndexMap::new();
    map_data.insert(
        MapKey::String("key".to_string()),
        Value::String("value".to_string()),
    );
    let map = Value::Map(map_data);
    assert_eq!(
        call_method(None, ValueRef::Immutable(&map), "is_map", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&map), "is_list", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_tuple()
    let tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(1))]);
    assert_eq!(
        call_method(None, ValueRef::Immutable(&tuple), "is_tuple", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&tuple), "is_list", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_function()
    use std::rc::Rc;
    use suji_ast::{Expr, Stmt};
    use suji_lexer::Span;
    use suji_values::FunctionBody;
    use suji_values::FunctionValue;
    let func = Value::Function(FunctionValue {
        params: vec![],
        body: FunctionBody::Ast(Stmt::Expr(Expr::Return {
            values: Vec::new(),
            span: Span::default(),
        })),
        env: Rc::new(suji_values::env::Env::new()),
    });
    assert_eq!(
        call_method(None, ValueRef::Immutable(&func), "is_function", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&func), "is_number", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_stream()
    use suji_values::StreamProxyKind;
    let stream = Value::StreamProxy(StreamProxyKind::Stdin);
    assert_eq!(
        call_method(None, ValueRef::Immutable(&stream), "is_stream", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&stream), "is_number", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test is_regex()
    use regex::Regex;
    let regex = Value::Regex(Regex::new("test").unwrap());
    assert_eq!(
        call_method(None, ValueRef::Immutable(&regex), "is_regex", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&regex), "is_string", vec![]).unwrap(),
        Value::Boolean(false)
    );

    // Test that EnvMap returns true for is_map()
    use suji_values::env_overlay::EnvProxy;
    let env_map = Value::EnvMap(Rc::new(EnvProxy::new()));
    assert_eq!(
        call_method(None, ValueRef::Immutable(&env_map), "is_map", vec![]).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&env_map), "is_string", vec![]).unwrap(),
        Value::Boolean(false)
    );
}

#[test]
fn test_type_checking_methods_nil() {
    // Nil should return false for all type-checking methods
    let nil = Value::Nil;
    let receiver = ValueRef::Immutable(&nil);

    assert_eq!(
        call_method(None, receiver, "is_number", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_bool", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_string", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_list", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_map", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_tuple", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_function", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_stream", vec![]).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        call_method(None, ValueRef::Immutable(&nil), "is_regex", vec![]).unwrap(),
        Value::Boolean(false)
    );
}

#[test]
fn test_type_checking_methods_arity_errors() {
    // All type-checking methods should reject arguments
    let n = Value::Number(DecimalNumber::from_i64(42));
    let receiver = ValueRef::Immutable(&n);

    assert!(matches!(
        call_method(
            None,
            receiver,
            "is_number",
            vec![Value::Number(DecimalNumber::from_i64(1))]
        ),
        Err(suji_values::RuntimeError::ArityMismatch { .. })
    ));

    let s = Value::String("hello".to_string());
    assert!(matches!(
        call_method(
            None,
            ValueRef::Immutable(&s),
            "is_string",
            vec![Value::String("arg".to_string())]
        ),
        Err(suji_values::RuntimeError::ArityMismatch { .. })
    ));
}
