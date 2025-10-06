use suji_lang::runtime::value::DecimalNumber;
mod common;

use common::{eval_program, eval_string_expr};
use suji_lang::runtime::value::Value;

#[test]
fn test_comprehensive_arithmetic() {
    assert_eq!(
        eval_string_expr("2 + 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_string_expr("10 - 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(6))
    );
    assert_eq!(
        eval_string_expr("3 * 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(12))
    );
    assert_eq!(
        eval_string_expr("15 / 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_string_expr("17 % 5").unwrap(),
        Value::Number(DecimalNumber::from_i64(2))
    );
    assert_eq!(
        eval_string_expr("2 ^ 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(8))
    );

    assert_eq!(
        eval_string_expr("-5").unwrap(),
        Value::Number(DecimalNumber::from_i64(-5))
    );
    assert_eq!(
        eval_string_expr("-(-10)").unwrap(),
        Value::Number(DecimalNumber::from_i64(10))
    );
    assert_eq!(eval_string_expr("!true").unwrap(), Value::Boolean(false));
    assert_eq!(eval_string_expr("!false").unwrap(), Value::Boolean(true));

    assert_eq!(
        eval_string_expr("2 + 3 * 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(14))
    );
    assert_eq!(
        eval_string_expr("2 * 3 + 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(10))
    );
    assert_eq!(
        eval_string_expr("10 - 2 * 3").unwrap(),
        Value::Number(DecimalNumber::from_i64(4))
    );
    assert_eq!(
        eval_string_expr("20 / 4 + 1").unwrap(),
        Value::Number(DecimalNumber::from_i64(6))
    );

    assert_eq!(
        eval_string_expr("2 ^ 3 ^ 2").unwrap(),
        Value::Number(DecimalNumber::from_i64(512))
    );
    assert_eq!(
        eval_string_expr("3 ^ 2 ^ 2").unwrap(),
        Value::Number(DecimalNumber::from_i64(81))
    );

    assert_eq!(
        eval_string_expr("(2 + 3) * 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(20))
    );
    assert_eq!(
        eval_string_expr("2 * (3 + 4)").unwrap(),
        Value::Number(DecimalNumber::from_i64(14))
    );
    assert_eq!(
        eval_string_expr("(2 ^ 3) ^ 2").unwrap(),
        Value::Number(DecimalNumber::from_i64(64))
    );

    assert_eq!(
        eval_string_expr("0 + 0").unwrap(),
        Value::Number(DecimalNumber::from_i64(0))
    );
    assert_eq!(
        eval_string_expr("1 * 0").unwrap(),
        Value::Number(DecimalNumber::from_i64(0))
    );
    assert_eq!(
        eval_string_expr("5 ^ 0").unwrap(),
        Value::Number(DecimalNumber::from_i64(1))
    );
    assert_eq!(
        eval_string_expr("0 ^ 5").unwrap(),
        Value::Number(DecimalNumber::from_i64(0))
    );
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
    assert_eq!(
        eval_string_expr("2 ^ 3 * 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(32))
    );
    assert_eq!(
        eval_string_expr("2 * 3 ^ 4").unwrap(),
        Value::Number(DecimalNumber::from_i64(162))
    );
    assert_eq!(
        eval_string_expr("-2 ^ 2").unwrap(),
        Value::Number(-DecimalNumber::from_i64(4))
    );
    assert_eq!(
        eval_string_expr("(-2) ^ 2").unwrap(),
        Value::Number(DecimalNumber::from_i64(4))
    );
    assert_eq!(
        eval_string_expr("2 + 3 * 4 - 1").unwrap(),
        Value::Number(DecimalNumber::from_i64(13))
    );

    let range_result = eval_string_expr("0..2+3").unwrap();
    if let Value::List(items) = range_result {
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(0)));
        assert_eq!(items[4], Value::Number(DecimalNumber::from_i64(4)));
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
