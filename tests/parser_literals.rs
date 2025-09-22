use nnlang::ast::{Expr, Literal};
use nnlang::parser::parse_expression;

#[test]
fn test_parse_number_literal() {
    let result = parse_expression("42");
    assert!(result.is_ok());

    if let Ok(Expr::Literal(Literal::Number(n, _))) = result {
        assert_eq!(n, 42.0);
    } else {
        panic!("Expected number literal");
    }
}

#[test]
fn test_parse_boolean_literals() {
    let result_true = parse_expression("true");
    assert!(result_true.is_ok());
    if let Ok(Expr::Literal(Literal::Boolean(b, _))) = result_true {
        assert!(b);
    } else {
        panic!("Expected true literal");
    }

    let result_false = parse_expression("false");
    assert!(result_false.is_ok());
    if let Ok(Expr::Literal(Literal::Boolean(b, _))) = result_false {
        assert!(!b);
    } else {
        panic!("Expected false literal");
    }
}

#[test]
fn test_parse_identifier() {
    let result = parse_expression("variable_name");
    assert!(result.is_ok());

    if let Ok(Expr::Literal(Literal::Identifier(name, _))) = result {
        assert_eq!(name, "variable_name");
    } else {
        panic!("Expected identifier");
    }
}

#[test]
fn test_parse_nil_literal() {
    let result = parse_expression("nil");
    assert!(result.is_ok());

    if let Ok(Expr::Literal(Literal::Nil(_))) = result {
        // Expected
    } else {
        panic!("Expected nil literal");
    }
}
