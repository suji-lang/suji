use suji_lang::ast::{BinaryOp, Expr, Literal, Stmt};
mod common;
use common::{eval_program, parse_expression};

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
                    assert_eq!(*n, "2".to_string());
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
            if let Stmt::Return { values, .. } = &statements[0] {
                if let Some(Expr::Binary {
                    op, left, right, ..
                }) = values.first()
                {
                    assert_eq!(*op, BinaryOp::Multiply);
                    if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
                        assert_eq!(name, "x");
                    } else {
                        panic!("Expected x as left operand");
                    }
                    if let Expr::Literal(Literal::Number(n, _)) = right.as_ref() {
                        assert_eq!(*n, "2".to_string());
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
fn test_parse_function_multi_return_destructure() {
    let result = parse_expression("|x| { return x, x + 1 }");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { body, .. }) = result {
        if let Stmt::Block { statements, .. } = body.as_ref() {
            assert_eq!(statements.len(), 1);
            if let Stmt::Return { values, .. } = &statements[0] {
                assert_eq!(values.len(), 2);
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
fn test_parse_destructuring_assignment() {
    let result = parse_expression("(a, _, b) = f()");
    assert!(result.is_ok());

    if let Ok(Expr::Assign { target, .. }) = result {
        if let Expr::Destructure { elements, .. } = target.as_ref() {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected destructure target");
        }
    } else {
        panic!("Expected assignment expression");
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
                assert_eq!(*n, "42".to_string());
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
            assert_eq!(*n, "0".to_string());
        } else {
            panic!("Expected default value 0 for x");
        }

        if let Some(Expr::Literal(Literal::Number(n, _))) = &params[1].default {
            assert_eq!(*n, "1".to_string());
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

#[test]
fn function_defaults_with_pipe_terminator() {
    let src = r#"
f = |x, y = 10| x + y
f(3)
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "13");
}
