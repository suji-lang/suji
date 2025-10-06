use suji_lang::runtime::value::DecimalNumber;
mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

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
