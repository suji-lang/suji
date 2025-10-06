use suji_lang::runtime::value::DecimalNumber;
mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

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
