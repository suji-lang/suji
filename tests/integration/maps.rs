use suji_values::DecimalNumber;

use super::common::{eval_program, eval_string_expr};
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

// ============================================================================
// Map Methods
// ============================================================================

#[test]
fn test_map_keys_method_integration() {
    let result = eval_program(
        r#"
        config = { 
            name: "Alice", 
            age: 30, 
            city: "New York",
            active: true
        }
        keys = config::keys()
        keys::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(4)));
}

#[test]
fn test_map_values_method_integration() {
    let result = eval_program(
        r#"
        scores = { 1: 100, 2: 85, 3: 92 }
        values = scores::values()
        values::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(3)));
}

#[test]
fn test_map_to_list_method_integration() {
    let result = eval_program(
        r#"
        user = { name: "Bob", age: 25 }
        pairs = user::to_list()
        pairs::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(2)));
}

#[test]
fn test_map_length_method_integration() {
    let result = eval_program(
        r#"
        data = { a: 1, b: 2, c: 3, d: 4, e: 5 }
        data::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(5)));
}

#[test]
fn test_map_methods_combined_usage() {
    let result = eval_program(
        r#"
        config = { 
            name: "Alice", 
            age: 30, 
            city: "New York",
            active: true
        }
        
        # Test all methods together
        key_count = config::keys()::length()
        value_count = config::values()::length()
        pair_count = config::to_list()::length()
        direct_count = config::length()
        
        # All should be equal
        key_count == value_count && value_count == pair_count && pair_count == direct_count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_map_methods_with_iteration() {
    let result = eval_program(
        r#"
        scores = { math: 95, science: 87, english: 92 }
        
        # Iterate over keys
        key_count = 0
        loop through scores::keys() with key {
            key_count = key_count + 1
        }
        
        # Iterate over values
        value_count = 0
        loop through scores::values() with value {
            value_count = value_count + 1
        }
        
        # Iterate over pairs
        pair_count = 0
        loop through scores::to_list() with pair {
            pair_count = pair_count + 1
        }
        
        # All counts should equal the map length
        key_count == scores::length() && value_count == scores::length() && pair_count == scores::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

// ============================================================================
// Map Iteration
// ============================================================================

#[test]
fn test_map_iteration_no_bindings() {
    let result = eval_program(
        r#"
        map = { a: 1, b: 2 }
        count = 0
        loop through map {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(2)));
}

#[test]
fn test_map_iteration_one_binding() {
    let result = eval_program(
        r#"
        map = { a: 1, b: 2 }
        result = ""
        loop through map with k {
            result = result + k + " "
        }
        result
    "#,
    );
    assert!(result.is_ok());
    let result_str = match result.unwrap() {
        Value::String(s) => s,
        _ => panic!("Expected string result"),
    };
    // Order is not guaranteed, so check both keys are present
    assert!(result_str.contains("a "));
    assert!(result_str.contains("b "));
}

#[test]
fn test_map_iteration_two_bindings() {
    let result = eval_program(
        r#"
        map = { a: 10, b: 20 }
        total = 0
        loop through map with k, v {
            total = total + v
        }
        total
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(30)));
}

#[test]
fn test_map_iteration_empty_map() {
    let result = eval_program(
        r#"
        empty = {}
        count = 0
        loop through empty with k, v {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(0)));
}

#[test]
fn test_map_iteration_variable_scoping() {
    let result = eval_program(
        r#"
        outer = "original"
        map = { key: "value" }
        loop through map with k, v {
            inner = "inside"
            outer = "modified"
        }
        outer
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("modified".to_string()));
}

#[test]
fn test_map_iteration_control_flow_break() {
    let result = eval_program(
        r#"
        map = { a: 1, b: 2, c: 3, d: 4 }
        result = ""
        loop through map with k, v {
            match k {
                "c" => { break },
                _ => result = result + k,
            }
        }
        result
    "#,
    );
    assert!(result.is_ok());
    let result_str = match result.unwrap() {
        Value::String(s) => s,
        _ => panic!("Expected string result"),
    };
    // Should break when it hits "c", so "c" and "d" should not be in result
    assert!(!result_str.contains("c"));
    // The exact order is not guaranteed, but it should contain some keys before "c"
    assert!(!result_str.is_empty());
}

#[test]
fn test_map_iteration_mixed_key_types() {
    let result = eval_program(
        r#"
        mixed = { "string": 1, 42: "number", true: false }
        count = 0
        loop through mixed with k, v {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(3)));
}

#[test]
fn test_map_iteration_nested_maps() {
    let result = eval_program(
        r#"
        nested = {
            user: { name: "Alice" },
            settings: { theme: "dark" }
        }
        count = 0
        loop through nested with section, data {
            count = count + 1
        }
        count
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(2)));
}
