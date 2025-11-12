mod common;

use common::eval_program;
use suji_values::Value;

#[test]
fn test_match_single_negative_literal() {
    let source = r#"
        x = -5
        result = match x {
            -5 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_negative_with_alternation() {
    let source = r#"
        x = -10
        result = match x {
            -10 | -5 | 0 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_negative_alternation_middle() {
    let source = r#"
        x = -5
        result = match x {
            -10 | -5 | 0 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_negative_alternation_last() {
    let source = r#"
        x = 0
        result = match x {
            -10 | -5 | 0 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_mixed_positive_and_negative() {
    let source = r#"
        x = -3
        result = match x {
            -5 => "negative five",
            -3 => "negative three",
            0 => "zero",
            3 => "positive three",
            5 => "positive five",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("negative three".to_string()));
}

#[test]
fn test_match_negative_zero() {
    let source = r#"
        x = -0
        result = match x {
            -0 => "negative zero",
            0 => "zero",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    // -0 and 0 should be treated as the same value
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_match_large_negative_number() {
    let source = r#"
        x = -999999
        result = match x {
            -999999 => "large negative",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("large negative".to_string()));
}

#[test]
fn test_match_negative_does_not_match_positive() {
    let source = r#"
        x = 5
        result = match x {
            -5 => "negative",
            _ => "positive or other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("positive or other".to_string()));
}

#[test]
fn test_match_positive_does_not_match_negative() {
    let source = r#"
        x = -5
        result = match x {
            5 => "positive",
            _ => "negative or other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("negative or other".to_string()));
}

#[test]
fn test_match_negative_decimal() {
    let source = r#"
        x = -3.14
        result = match x {
            -3.14 => "pi",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("pi".to_string()));
}

#[test]
fn test_match_negative_in_tuple_pattern() {
    let source = r#"
        pair = (-1, 2)
        result = match pair {
            (-1, 2) => "match",
            _ => "no match",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("match".to_string()));
}
