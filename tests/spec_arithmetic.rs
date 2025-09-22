mod common;

use common::{eval_program, eval_string_expr};
use nnlang::runtime::value::Value;

#[test]
fn test_comprehensive_arithmetic() {
    assert_eq!(eval_string_expr("2 + 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_string_expr("10 - 4").unwrap(), Value::Number(6.0));
    assert_eq!(eval_string_expr("3 * 4").unwrap(), Value::Number(12.0));
    assert_eq!(eval_string_expr("15 / 3").unwrap(), Value::Number(5.0));
    assert_eq!(eval_string_expr("17 % 5").unwrap(), Value::Number(2.0));
    assert_eq!(eval_string_expr("2 ^ 3").unwrap(), Value::Number(8.0));

    assert_eq!(eval_string_expr("-5").unwrap(), Value::Number(-5.0));
    assert_eq!(eval_string_expr("-(-10)").unwrap(), Value::Number(10.0));
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));

    assert_eq!(eval_string_expr("2 + 3 * 4").unwrap(), Value::Number(14.0));
    assert_eq!(eval_string_expr("2 * 3 + 4").unwrap(), Value::Number(10.0));
    assert_eq!(eval_string_expr("10 - 2 * 3").unwrap(), Value::Number(4.0));
    assert_eq!(eval_string_expr("20 / 4 + 1").unwrap(), Value::Number(6.0));

    assert_eq!(eval_string_expr("2 ^ 3 ^ 2").unwrap(), Value::Number(512.0));
    assert_eq!(eval_string_expr("3 ^ 2 ^ 2").unwrap(), Value::Number(81.0));

    assert_eq!(
        eval_string_expr("(2 + 3) * 4").unwrap(),
        Value::Number(20.0)
    );
    assert_eq!(
        eval_string_expr("2 * (3 + 4)").unwrap(),
        Value::Number(14.0)
    );
    assert_eq!(
        eval_string_expr("(2 ^ 3) ^ 2").unwrap(),
        Value::Number(64.0)
    );

    assert_eq!(eval_string_expr("0 + 0").unwrap(), Value::Number(0.0));
    assert_eq!(eval_string_expr("1 * 0").unwrap(), Value::Number(0.0));
    assert_eq!(eval_string_expr("5 ^ 0").unwrap(), Value::Number(1.0));
    assert_eq!(eval_string_expr("0 ^ 5").unwrap(), Value::Number(0.0));
}

#[test]
fn test_arithmetic_error_cases() {
    assert!(eval_string_expr("5 / 0").is_err());
    assert!(eval_string_expr("1.0 / 0.0").is_err());
    assert!(eval_string_expr("5 % 0").is_err());
    assert!(eval_string_expr(r#"5 + \"hello\""#).is_err());
    assert!(eval_string_expr("true * 3").is_err());
    assert!(eval_string_expr("false + 1").is_err());
    assert!(eval_string_expr("!5").is_err());
    assert!(eval_string_expr("-true").is_err());
    assert!(eval_string_expr("-\"hello\"").is_err());
    assert!(eval_program("\"hello\"++").is_err());
    assert!(eval_program("true--").is_err());
}

#[test]
fn test_comprehensive_precedence() {
    assert_eq!(eval_string_expr("2 ^ 3 * 4").unwrap(), Value::Number(32.0));
    assert_eq!(eval_string_expr("2 * 3 ^ 4").unwrap(), Value::Number(162.0));
    assert_eq!(eval_string_expr("-2 ^ 2").unwrap(), Value::Number(-4.0));
    assert_eq!(eval_string_expr("(-2) ^ 2").unwrap(), Value::Number(4.0));
    assert_eq!(
        eval_string_expr("2 + 3 * 4 - 1").unwrap(),
        Value::Number(13.0)
    );

    let range_result = eval_string_expr("0..2+3").unwrap();
    if let Value::List(items) = range_result {
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(0.0));
        assert_eq!(items[4], Value::Number(4.0));
    } else {
        panic!("Expected list from range expression");
    }

    assert_eq!(eval_string_expr("2 + 3 > 4").unwrap(), Value::Boolean(true));
    assert_eq!(
        eval_string_expr("5 > 3 == true").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("true == true && false == false").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("false && true || true").unwrap(),
        Value::Boolean(true)
    );
}
