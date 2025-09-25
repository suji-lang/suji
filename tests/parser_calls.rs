mod common;
use common::parse_expression;

#[test]
fn test_function_and_method_calls() {
    assert!(parse_expression("func() ").is_ok());
    assert!(parse_expression("obj::method() ").is_ok());
    assert!(parse_expression("func(42) ").is_ok());
    assert!(parse_expression("obj::method(\"test\") ").is_ok());
    assert!(parse_expression("func(1, 2, 3) ").is_ok());
    assert!(parse_expression("obj::method(a, b) ").is_ok());
    assert!(parse_expression("func(1, 2,) ").is_ok());
    assert!(parse_expression("obj::method(a,) ").is_ok());
    assert!(parse_expression("func(a + b, c * d) ").is_ok());
    assert!(parse_expression("func(other(x), obj::method(y)) ").is_ok());
}

#[test]
fn test_parse_errors() {
    assert!(parse_expression("func(1 2)").is_err());
    assert!(parse_expression("func(1, 2").is_err());
}
