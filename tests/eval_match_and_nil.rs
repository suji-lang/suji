use nnlang::runtime::value::DecimalNumber;
mod common;

use common::eval_program;
use nnlang::runtime::value::Value;

#[test]
fn test_nil_pattern_matching() {
    let result = eval_program(
        r#"
        user = nil
        result = match user {
            nil: "No user found"
            _: "Some user found"
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("No user found".to_string()));
}

#[test]
fn test_return_statements_in_match_arms() {
    let result = eval_program(
        r#"
        f = || {
            match 1 {
                1: return 1
                2: return 2
                _: return 0
            }
        }
        f()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(1)));
}

#[test]
fn test_implicit_returns_in_match_arms() {
    let result = eval_program(
        r#"
        g = || {
            match 1 {
                1: 1
                2: 2
                _: 0
            }
        }
        g()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(1)));
}

#[test]
fn test_map_literals_in_match_arms() {
    let result = eval_program(
        r#"
        result = match 1 {
            1: { "status": "success", "value": 42 }
            2: { "status": "error", "message": "failed" }
            _: { "status": "unknown" }
        }
        result:status
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("success".to_string()));
}

#[test]
fn test_mixed_match_arm_syntax() {
    let result = eval_program(
        r#"
        import std:println
        h = || {
            match 1 {
                1: {
                    println("one")
                    return 1
                }
                2: 2
                _: return 0
            }
        }
        h()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(1)));
}

#[test]
fn test_complex_nil_handling() {
    let result = eval_program(
        r#"
        find_user = |id| {
            match id {
                1: { "name": "Alice" }
                2: { "name": "Bob" }
                _: nil
            }
        }
        user_info = match find_user(3) {
            nil: "User not found"
            user: "User: " + user:name
        }
        user_info
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("User not found".to_string()));
}
