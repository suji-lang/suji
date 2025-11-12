use suji_ast::ExportBody;
use suji_ast::{Expr, Literal, Stmt};
use suji_parser::parse_program;

mod common;
use common::parse_statement;

#[test]
fn test_parse_return_statement() {
    let result = parse_statement("return 42");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Return { values, .. })) = result {
        assert_eq!(values.len(), 1);
        if let Expr::Literal(Literal::Number(n, _)) = &values[0] {
            assert_eq!(n, "42");
        } else {
            panic!("Expected number in return");
        }
    } else {
        panic!("Expected return expression statement");
    }
}

#[test]
fn test_parse_export_expression() {
    let result = parse_statement("export 42");
    assert!(result.is_ok());

    if let Ok(Stmt::Export { body, .. }) = result {
        match body {
            ExportBody::Expr(Expr::Literal(Literal::Number(n, _))) => {
                assert_eq!(n, "42");
            }
            _ => panic!("Expected export expression body with number"),
        }
    } else {
        panic!("Expected export statement");
    }
}

#[test]
fn test_parse_export_map() {
    let result = parse_statement("export { x: 1, y: 2 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Export { body, .. }) = result {
        match body {
            ExportBody::Map(spec) => {
                assert_eq!(spec.items.len(), 2);
            }
            _ => panic!("Expected map export body"),
        }
    } else {
        panic!("Expected export statement");
    }
}

#[test]
fn test_parse_return_without_value() {
    let result = parse_statement("return");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Return { values, .. })) = result {
        assert!(values.is_empty());
        // Expected
    } else {
        panic!("Expected return expression statement without value");
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

#[test]
fn test_parse_multiple_exports_error() {
    let src = "export { x: 1 }\nexport 2";
    common::assert_parse_fails(
        src,
        "Multiple export statements||Only one export statement is allowed",
    );
}

#[test]
fn test_parse_export_error_no_form() {
    // "export" followed by EOF should fail
    common::assert_parse_fails(
        "export",
        "Expected '{' or expression after export||Expected '{' after export",
    );
}
