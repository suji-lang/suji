mod common;

use common::eval_program;
use suji_lang::runtime::value::{DecimalNumber, Value};

#[test]
fn test_conditional_match_basic() {
    let result = eval_program(
        r#"
        x = 5
        result = match {
            x > 0 => "positive",
            _ => "non-positive",
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("positive".to_string()));
}

#[test]
fn test_conditional_match_multiple_conditions() {
    let result = eval_program(
        r#"
        x = 7
        result = match {
            x > 10 => "very large",
            x > 5 => "large",
            x > 0 => "positive",
            _ => "non-positive",
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("large".to_string()));
}

#[test]
fn test_conditional_match_complex_boolean() {
    let result = eval_program(
        r#"
        x = 3
        y = 4
        result = match {
            x > 0 && y > 0 => "both positive",
            x > 0 => "only x positive",
            y > 0 => "only y positive",
            _ => "neither positive",
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("both positive".to_string()));
}

#[test]
fn test_conditional_match_in_function() {
    let result = eval_program(
        r#"
        f = |x| {
            return match {
                x > 10 => "very large",
                x > 5 => "large",
                x > 0 => "positive",
                _ => "non-positive",
            }
        }
        f(15)
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("very large".to_string()));
}

#[test]
fn test_conditional_match_mixed_with_traditional() {
    let result = eval_program(
        r#"
        x = 2
        y = 3

        result1 = match x {
            1 => "one",
            2 => "two",
            _ => "other",
        }

        result2 = match {
            y > 2 => "y is large",
            _ => "y is small",
        }

        result1 + " " + result2
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("two y is large".to_string()));
}

#[test]
fn test_conditional_match_all_conditions_false() {
    let result = eval_program(
        r#"
        x = -5
        result = match {
            x > 0 => "positive",
            x > -10 => "small negative",
            _ => "large negative",
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("small negative".to_string()));
}

#[test]
fn test_conditional_match_with_wildcard() {
    let result = eval_program(
        r#"
        x = 0
        result = match {
            x > 0 => "positive",
            x < 0 => "negative",
            _ => "zero",
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("zero".to_string()));
}

#[test]
fn test_conditional_match_nested() {
    let result = eval_program(
        r#"
        x = 3
        y = 4
        result = match {
            x > 0 => match {
                y > 0 => "both positive",
                _ => "x positive, y non-positive",
            },
            _ => "x non-positive",
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("both positive".to_string()));
}

#[test]
fn test_conditional_match_in_binary_operation() {
    let result = eval_program(
        r#"
        x = 5
        y = 3
        result = match {
            x > 0 => 10,
            _ => 0,
        } + match {
            y > 0 => 5,
            _ => 0,
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(15)));
}

#[test]
fn test_conditional_match_with_string_comparison() {
    let result = eval_program(
        r#"
        name = "alice"
        result = match {
            name == "alice" => "Hello Alice",
            name == "bob" => "Hello Bob",
            _ => "Hello stranger",
        }
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello Alice".to_string()));
}
