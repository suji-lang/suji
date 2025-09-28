use nnlang::ast::{Expr, Literal, Stmt};
use nnlang::parser::parse_program;

mod common;
use common::parse_statement;

#[test]
fn test_parse_return_statement() {
    let result = parse_statement("return 42");
    assert!(result.is_ok());

    if let Ok(Stmt::Return { values, .. }) = result {
        assert_eq!(values.len(), 1);
        if let Expr::Literal(Literal::Number(n, _)) = &values[0] {
            assert_eq!(n, "42");
        } else {
            panic!("Expected number in return");
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_parse_return_without_value() {
    let result = parse_statement("return");
    assert!(result.is_ok());

    if let Ok(Stmt::Return { values, .. }) = result {
        assert!(values.is_empty());
        // Expected
    } else {
        panic!("Expected return statement without value");
    }
}

#[test]
fn test_parse_expression_statement() {
    let result = parse_statement("variable_name");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Literal(Literal::Identifier(name, _)))) = result {
        assert_eq!(name, "variable_name");
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_parse_program() {
    let result = parse_program("42\ntrue\nvariable");
    assert!(result.is_ok());

    if let Ok(statements) = result {
        assert_eq!(statements.len(), 3);

        if let Stmt::Expr(Expr::Literal(Literal::Number(n, _))) = &statements[0] {
            assert_eq!(*n, "42".to_string());
        } else {
            panic!("Expected first statement to be number");
        }

        if let Stmt::Expr(Expr::Literal(Literal::Boolean(b, _))) = &statements[1] {
            assert!(*b);
        } else {
            panic!("Expected second statement to be boolean");
        }

        if let Stmt::Expr(Expr::Literal(Literal::Identifier(name, _))) = &statements[2] {
            assert_eq!(name, "variable");
        } else {
            panic!("Expected third statement to be identifier");
        }
    } else {
        panic!("Expected program to parse successfully");
    }
}
