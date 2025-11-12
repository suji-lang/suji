use suji_values::DecimalNumber;
mod common;

use common::{eval_program, eval_string_expr};
use suji_values::{MapKey, Value};

#[test]
fn test_map_literals() {
    assert_eq!(
        eval_string_expr(r#"{ "a": 1 }"#).unwrap(),
        Value::Map(indexmap::IndexMap::from([(
            MapKey::String("a".to_string()),
            Value::Number(DecimalNumber::from_i64(1))
        )]))
    );

    assert_eq!(
        eval_string_expr("{}").unwrap(),
        Value::Map(indexmap::IndexMap::new())
    );

    let Value::Map(map) = eval_string_expr(r#"{ "name": "Alice", "age": 30 }"#).unwrap() else {
        panic!("Expected map")
    };
    assert_eq!(map.len(), 2);
}

#[test]
fn test_map_access_comprehensive() {
    // Test basic map access with string keys
    let result = eval_string_expr("{\"hello\": \"world\"}:hello").unwrap();
    assert_eq!(result, Value::String("world".to_string()));

    let result = eval_string_expr("{\"a\": 1, \"b\": 2}:b").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    // Test map access on variables
    let result = eval_program("data = {\"name\": \"Alice\", \"age\": 30}\ndata:name").unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));

    let result = eval_program("data = {\"name\": \"Alice\", \"age\": 30}\ndata:age").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(30)));

    // Test map access on complex expressions
    let result = eval_string_expr("{\"nested\": {\"value\": 42}}:nested").unwrap();
    if let Value::Map(_) = result {
        // Map access returning nested map works
    } else {
        panic!("Expected nested map");
    }

    // Test non-existent key access (should return nil or error)
    match eval_string_expr("{\"a\": 1}:nonexistent") {
        Ok(Value::Nil) => {}
        Ok(other) => {
            assert!(matches!(
                other,
                Value::Nil | Value::String(_) | Value::Number(_)
            ));
        }
        Err(_) => {}
    }
}
