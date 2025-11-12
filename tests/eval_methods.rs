use suji_values::DecimalNumber;
mod common;

use common::{eval_program, eval_string_expr};
use suji_values::Value;

#[test]
fn test_method_calls() {
    assert_eq!(
        eval_string_expr("[1, 2, 3]::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(3))
    );
    assert_eq!(
        eval_string_expr(r#""hello"::length()"#).unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_string_expr(r#""hello world"::contains("world")"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""document.pdf"::starts_with("doc")"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""document.pdf"::ends_with(".pdf")"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""hello world"::replace("world", "universe")"#).unwrap(),
        Value::String("hello universe".to_string())
    );
    assert_eq!(
        eval_string_expr(r#""  hello world  "::trim()"#).unwrap(),
        Value::String("hello world".to_string())
    );
    assert_eq!(
        eval_string_expr(r#""hello world"::upper()"#).unwrap(),
        Value::String("HELLO WORLD".to_string())
    );
    assert_eq!(
        eval_string_expr(r#""HELLO WORLD"::lower()"#).unwrap(),
        Value::String("hello world".to_string())
    );
    assert_eq!(
        eval_string_expr(r#""hello"::reverse()"#).unwrap(),
        Value::String("olleh".to_string())
    );
    assert_eq!(
        eval_string_expr(r#""ha"::repeat(3)"#).unwrap(),
        Value::String("hahaha".to_string())
    );
    assert_eq!(
        eval_program("list = [1, 2, 3, 4]\nlist::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(4))
    );
    assert_eq!(
        eval_string_expr("(1..5)::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(4))
    );
}

#[test]
fn test_pipe_single_hop() {
    // Source prints a line; destination reads from io:stdin and returns a string
    let program = r#"
import std:println
import std:io

make_destination = || {
    return || {
        lines = io:stdin::read_lines()
        match lines::length() > 0 {
            true => { "output received" },
            false => { "no output" },
        }
    }
}

make_source = || {
    return || {
        println("test")
    }
}

result = make_source() | make_destination()
result
"#;

    let val = eval_program(program).unwrap();
    assert_eq!(val, Value::String("output received".to_string()));
}

#[test]
fn test_method_error_handling() {
    assert!(eval_string_expr(r#""hello"::contains(42)"#).is_err());
    assert!(eval_string_expr(r#""hello"::replace("world")"#).is_err());
    assert!(eval_string_expr(r#""hello"::repeat(-1)"#).is_err());
    assert!(eval_string_expr(r#""hello"::trim("a", "b")"#).is_err()); // Too many args
    assert!(eval_string_expr(r#""hello"::trim(42)"#).is_err()); // Wrong type
    assert!(eval_string_expr(r#""hello"::upper("test")"#).is_err());
    assert!(eval_string_expr(r#""hello"::lower("test")"#).is_err());
    assert!(eval_string_expr(r#""hello"::reverse("test")"#).is_err());
    assert!(eval_string_expr(r#""hello"::starts_with(42)"#).is_err());
    assert!(eval_string_expr(r#""hello"::ends_with(42)"#).is_err());
}
