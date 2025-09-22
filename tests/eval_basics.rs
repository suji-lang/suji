mod common;

use common::{eval_program, eval_string_expr};
use nnlang::runtime::value::Value;

#[test]
fn test_arithmetic_operations() {
    assert_eq!(eval_string_expr("2 + 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_string_expr("10 - 4").unwrap(), Value::Number(6.0));
    assert_eq!(eval_string_expr("3 * 4").unwrap(), Value::Number(12.0));
    assert_eq!(eval_string_expr("15 / 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_string_expr("17 % 5").unwrap(), Value::Number(2.0));
    assert_eq!(eval_string_expr("2 ^ 3").unwrap(), Value::Number(8.0));

    // Precedence
    assert_eq!(eval_string_expr("2 + 3 * 4").unwrap(), Value::Number(14.0));
    assert_eq!(
        eval_string_expr("(2 + 3) * 4").unwrap(),
        Value::Number(20.0)
    );
    assert_eq!(eval_string_expr("2 ^ 3 ^ 2").unwrap(), Value::Number(512.0));
}

#[test]
fn test_unary_operations() {
    assert_eq!(eval_string_expr("-5").unwrap(), Value::Number(-5.0));
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("-(-10)").unwrap(), Value::Number(10.0));
}

#[test]
fn test_comparison_operations() {
    assert_eq!(eval_string_expr("5 > 3").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("2 < 1").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("4 >= 4").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 <= 2").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("5 == 5").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("3 != 4").unwrap(), Value::Boolean(true));
}

#[test]
fn test_logical_operations() {
    assert_eq!(
        eval_string_expr("true && true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true && false").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("false || true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false || false").unwrap(),
        Value::Boolean(false)
    );
}

#[test]
fn test_basic_literals() {
    assert_eq!(eval_string_expr("42").unwrap(), Value::Number(42.0));
    assert_eq!(eval_string_expr("2.5").unwrap(), Value::Number(2.5));
    assert_eq!(eval_string_expr("true").unwrap(), Value::Boolean(true));
    assert_eq!(eval_string_expr("false").unwrap(), Value::Boolean(false));
}

#[test]
fn test_error_cases() {
    assert!(eval_string_expr("5 / 0").is_err());
    assert!(eval_string_expr("5 % 0").is_err());
    assert!(eval_string_expr(r#"5 + \"hello\""#).is_err());
    assert!(eval_string_expr("true * 3").is_err());
    assert!(eval_string_expr("!5").is_err());
    assert!(eval_string_expr("-true").is_err());
}

#[test]
fn test_short_circuit_evaluation() {
    assert_eq!(
        eval_string_expr("false && (5 / 0)").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr("true || (5 / 0)").unwrap(),
        Value::Boolean(true)
    );
}

#[test]
fn test_complex_literals() {
    let Value::String(s) = eval_string_expr(r#""hello""#).unwrap() else {
        panic!("Expected string")
    };
    assert_eq!(s, "hello");

    let Value::List(items) = eval_string_expr("[1, 2, 3]").unwrap() else {
        panic!("Expected list")
    };
    assert_eq!(
        items,
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
    );

    let Value::Tuple(items) = eval_string_expr("(1, 2)").unwrap() else {
        panic!("Expected tuple")
    };
    assert_eq!(items, vec![Value::Number(1.0), Value::Number(2.0)]);

    let Value::Map(_) = eval_string_expr(r#"{ "a": 1 }"#).unwrap() else {
        panic!("Expected map")
    };

    let Value::List(items) = eval_string_expr("1..4").unwrap() else {
        panic!("Expected range list")
    };
    assert_eq!(
        items,
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
    );
}

#[test]
fn test_single_element_tuple_syntax() {
    // Test single-element tuple syntax (x,) vs grouping (x)

    // Single-element tuple with trailing comma should create a tuple
    let Value::Tuple(items) = eval_string_expr("(42,)").unwrap() else {
        panic!("Expected single-element tuple from (42,)")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::Number(42.0));

    // Grouping without trailing comma should just return the value
    let result = eval_string_expr("(42)").unwrap();
    assert_eq!(result, Value::Number(42.0));
    assert!(
        !matches!(result, Value::Tuple(_)),
        "Expected number, not tuple from (42)"
    );

    // Test with string values too
    let Value::Tuple(items) = eval_string_expr("(\"hello\",)").unwrap() else {
        panic!("Expected single-element tuple from (\"hello\",)")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::String("hello".to_string()));

    // Grouping with string
    let result = eval_string_expr("(\"hello\")").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));
    assert!(
        !matches!(result, Value::Tuple(_)),
        "Expected string, not tuple from (\"hello\")"
    );

    // Test with expressions
    let Value::Tuple(items) = eval_string_expr("(2 + 3,)").unwrap() else {
        panic!("Expected single-element tuple from (2 + 3,)")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::Number(5.0));

    // Grouping with expression
    let result = eval_string_expr("(2 + 3)").unwrap();
    assert_eq!(result, Value::Number(5.0));
    assert!(
        !matches!(result, Value::Tuple(_)),
        "Expected number, not tuple from (2 + 3)"
    );
}

#[test]
fn test_program_evaluation() {
    assert_eq!(eval_program("42").unwrap(), Value::Number(42.0));
    assert_eq!(eval_program("2 + 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_program("x = 5").unwrap(), Value::Number(5.0));
}
