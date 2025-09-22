mod common;

use common::{eval_program, eval_string_expr};
use nnlang::runtime::value::Value;

#[test]
fn test_number_methods() {
    assert_eq!(eval_string_expr("(-5)::abs()").unwrap(), Value::Number(5.0));
    assert_eq!(
        eval_string_expr("3.14159::ceil()").unwrap(),
        Value::Number(4.0)
    );
    assert_eq!(
        eval_string_expr("3.14159::floor()").unwrap(),
        Value::Number(3.0)
    );
    assert_eq!(
        eval_string_expr("3.14159::round()").unwrap(),
        Value::Number(3.0)
    );
    assert_eq!(eval_string_expr("16::sqrt()").unwrap(), Value::Number(4.0));
    assert_eq!(eval_string_expr("2::pow(3)").unwrap(), Value::Number(8.0));
    assert_eq!(
        eval_string_expr("10::min(20)").unwrap(),
        Value::Number(10.0)
    );
    assert_eq!(
        eval_string_expr("10::max(20)").unwrap(),
        Value::Number(20.0)
    );
}

#[test]
fn test_number_method_expressions() {
    assert_eq!(
        eval_program("x = -5\ny = x::abs()\ny").unwrap(),
        Value::Number(5.0)
    );
    assert_eq!(
        eval_program("x = -3.7\ny = x::abs()::ceil()\ny").unwrap(),
        Value::Number(4.0)
    );
    assert_eq!(
        eval_program("x = 2\ny = 3\nz = x::pow(y) + 5::sqrt()\nz").unwrap(),
        Value::Number(8.0 + 5.0_f64.sqrt())
    );

    assert!(eval_string_expr("(-4)::sqrt()").is_err());
    assert!(eval_string_expr("2::pow(\"invalid\")").is_err());
    assert!(eval_string_expr("10::min(\"invalid\")").is_err());
}
