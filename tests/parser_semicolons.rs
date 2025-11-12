use suji_ast::{CompoundOp, Expr, Literal, Stmt};
mod common;
use common::parse_statement;

#[test]
fn test_parse_semicolon_statement_separators() {
    let result = parse_statement("{ x = 1; y = 2; z = 3 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Block { statements, .. }) = result {
        assert_eq!(statements.len(), 3);

        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[0] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "x");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, "1".to_string());
            }
        } else {
            panic!("Expected assignment statement");
        }

        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[1] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "y");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, "2".to_string());
            }
        } else {
            panic!("Expected assignment statement");
        }

        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[2] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "z");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, "3".to_string());
            }
        } else {
            panic!("Expected assignment statement");
        }
    } else {
        panic!("Expected block statement");
    }
}

#[test]
fn test_parse_mixed_semicolon_newline_separators() {
    let result = parse_statement("{ x = 1; y = 2\nz = 3 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Block { statements, .. }) = result {
        assert_eq!(statements.len(), 3);
    } else {
        panic!("Expected block statement");
    }
}

#[test]
fn test_parse_semicolon_with_compound_assignment() {
    let result = parse_statement("{ x += 5; y -= 3; z *= 2 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Block { statements, .. }) = result {
        assert_eq!(statements.len(), 3);

        if let Stmt::Expr(Expr::CompoundAssign {
            op, target, value, ..
        }) = &statements[0]
        {
            assert_eq!(*op, CompoundOp::PlusAssign);
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "x");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, "5".to_string());
            }
        } else {
            panic!("Expected compound assignment");
        }

        if let Stmt::Expr(Expr::CompoundAssign {
            op, target, value, ..
        }) = &statements[1]
        {
            assert_eq!(*op, CompoundOp::MinusAssign);
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "y");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, "3".to_string());
            }
        } else {
            panic!("Expected compound assignment");
        }

        if let Stmt::Expr(Expr::CompoundAssign {
            op, target, value, ..
        }) = &statements[2]
        {
            assert_eq!(*op, CompoundOp::MultiplyAssign);
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "z");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, "2".to_string());
            }
        } else {
            panic!("Expected compound assignment");
        }
    } else {
        panic!("Expected block statement");
    }
}
