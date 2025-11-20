use super::common::{parse_expression, parse_statement};

#[test]
fn test_error_handling() {
    let result = parse_expression("+ 42");
    assert!(result.is_err());

    let result = parse_expression("(42");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_cases_optional_braces() {
    let result = parse_expression("|x|");
    if result.is_ok() {
        println!("Unexpected success: {:?}", result);
    }
    assert!(result.is_err());

    let result = parse_statement("match x { 1: }");
    if result.is_ok() {
        println!("Unexpected success: {:?}", result);
    }
    assert!(result.is_err());
}
