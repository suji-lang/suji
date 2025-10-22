mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

#[test]
fn test_math_constants() {
    let result = eval_program(
        r#"
        import std:math
        pi_ok = math:PI > 3.14 && math:PI < 3.15
        e_ok = math:E > 2.71 && math:E < 2.72
        pi_ok && e_ok
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_math_sin() {
    let result = eval_program(
        r#"
        import std:math
        result = math:sin(0)
        result > -0.01 && result < 0.01
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_math_cos() {
    let result = eval_program(
        r#"
        import std:math
        result = math:cos(0)
        result > 0.99 && result < 1.01
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_math_log_exp_identity() {
    let result = eval_program(
        r#"
        import std:math
        x = 5
        result = math:log(math:exp(x))
        diff = result - x
        diff::abs() < 0.0001
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_math_log10() {
    let result = eval_program(
        r#"
        import std:math
        result = math:log10(100)
        result > 1.99 && result < 2.01
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_math_asin_domain() {
    let result = eval_program(
        r#"
        import std:math
        math:asin(2)
    "#,
    );
    assert!(result.is_err());
}

#[test]
fn test_math_log_negative() {
    let result = eval_program(
        r#"
        import std:math
        math:log(-1)
    "#,
    );
    assert!(result.is_err());
}
