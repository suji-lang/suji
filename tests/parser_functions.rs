use nnlang::ast::{BinaryOp, Expr, Literal, Stmt};
mod common;
use common::parse_expression;

#[test]
fn test_parse_function_single_expression() {
    let result = parse_expression("|x| x * 2");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "x");

        if let Stmt::Expr(expr) = body.as_ref() {
            if let Expr::Binary {
                op, left, right, ..
            } = expr
            {
                assert_eq!(*op, BinaryOp::Multiply);
                if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected x as left operand");
                }
                if let Expr::Literal(Literal::Number(n, _)) = right.as_ref() {
                    assert_eq!(*n, 2.0);
                } else {
                    panic!("Expected 2 as right operand");
                }
            } else {
                panic!("Expected multiplication expression");
            }
        } else {
            panic!("Expected expression statement");
        }
    } else {
        panic!("Expected function literal");
    }
}

#[test]
fn test_parse_function_block() {
    let result = parse_expression("|x| { return x * 2 }");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "x");

        if let Stmt::Block { statements, .. } = body.as_ref() {
            assert_eq!(statements.len(), 1);
            if let Stmt::Return {
                value: Some(expr), ..
            } = &statements[0]
            {
                if let Expr::Binary {
                    op, left, right, ..
                } = expr
                {
                    assert_eq!(*op, BinaryOp::Multiply);
                    if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
                        assert_eq!(name, "x");
                    } else {
                        panic!("Expected x as left operand");
                    }
                    if let Expr::Literal(Literal::Number(n, _)) = right.as_ref() {
                        assert_eq!(*n, 2.0);
                    } else {
                        panic!("Expected 2 as right operand");
                    }
                } else {
                    panic!("Expected multiplication expression");
                }
            } else {
                panic!("Expected return statement");
            }
        } else {
            panic!("Expected block statement");
        }
    } else {
        panic!("Expected function literal");
    }
}

#[test]
fn test_parse_empty_function_single_expression() {
    let result = parse_expression("|| 42");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 0);

        if let Stmt::Expr(expr) = body.as_ref() {
            if let Expr::Literal(Literal::Number(n, _)) = expr {
                assert_eq!(*n, 42.0);
            } else {
                panic!("Expected number literal");
            }
        } else {
            panic!("Expected expression statement");
        }
    } else {
        panic!("Expected function literal");
    }
}

#[test]
fn test_parse_function_with_default_params_single_expression() {
    let result = parse_expression("|x = 0, y = 1| x + y");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "x");
        assert_eq!(params[1].name, "y");

        if let Some(Expr::Literal(Literal::Number(n, _))) = &params[0].default {
            assert_eq!(*n, 0.0);
        } else {
            panic!("Expected default value 0 for x");
        }

        if let Some(Expr::Literal(Literal::Number(n, _))) = &params[1].default {
            assert_eq!(*n, 1.0);
        } else {
            panic!("Expected default value 1 for y");
        }

        if let Stmt::Expr(Expr::Binary {
            op, left, right, ..
        }) = body.as_ref()
        {
            assert_eq!(*op, BinaryOp::Add);
            if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected x as left operand");
            }
            if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
                assert_eq!(name, "y");
            } else {
                panic!("Expected y as right operand");
            }
        } else {
            panic!("Expected addition expression");
        }
    } else {
        panic!("Expected function literal");
    }
}
