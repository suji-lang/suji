use super::common::{eval_program, parse_expression};
use suji_ast::{BinaryOp, Expr, Literal, Stmt};

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
            if let Stmt::Expr(Expr::Return { values, .. }) = &statements[0] {
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
                panic!("Expected return expression statement");
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
            if let Stmt::Expr(Expr::Return { values, .. }) = &statements[0] {
                assert_eq!(values.len(), 2);
            } else {
                panic!("Expected return expression statement");
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

#[test]
fn test_parse_compose_right_and_left() {
    // f >> g
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("f >> g")
    {
        assert_eq!(op, BinaryOp::ComposeRight);
        match (left.as_ref(), right.as_ref()) {
            (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) => {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            }
            _ => panic!("Expected identifier operands"),
        }
    } else {
        panic!("Expected composition expression");
    }

    // f << g
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("f << g")
    {
        assert_eq!(op, BinaryOp::ComposeLeft);
        match (left.as_ref(), right.as_ref()) {
            (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) => {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            }
            _ => panic!("Expected identifier operands"),
        }
    } else {
        panic!("Expected composition expression");
    }
}

#[test]
fn test_composition_precedence_with_pipe_apply() {
    // x |> f >> g  => top-level is |> with right side as (f >> g)
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("x |> f >> g")
    {
        assert_eq!(op, BinaryOp::PipeApplyFwd);

        if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
            assert_eq!(name, "x");
        } else {
            panic!("Expected left to be identifier x");
        }

        if let Expr::Binary {
            op: right_op,
            left: r_left,
            right: r_right,
            ..
        } = right.as_ref()
        {
            assert_eq!(*right_op, BinaryOp::ComposeRight);
            if let (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) = (r_left.as_ref(), r_right.as_ref())
            {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            } else {
                panic!("Expected identifiers f and g in composition");
            }
        } else {
            panic!("Expected right to be a composition expression");
        }
    } else {
        panic!("Expected pipe-apply expression");
    }

    // f << g |> h  => top-level |> with left side as (f << g)
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("f << g |> h")
    {
        assert_eq!(op, BinaryOp::PipeApplyFwd);

        if let Expr::Binary {
            op: left_op,
            left: l_left,
            right: l_right,
            ..
        } = left.as_ref()
        {
            assert_eq!(*left_op, BinaryOp::ComposeLeft);
            if let (
                Expr::Literal(Literal::Identifier(l, _)),
                Expr::Literal(Literal::Identifier(r, _)),
            ) = (l_left.as_ref(), l_right.as_ref())
            {
                assert_eq!(l, "f");
                assert_eq!(r, "g");
            } else {
                panic!("Expected identifiers f and g in composition");
            }
        } else {
            panic!("Expected left to be a composition expression");
        }

        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "h");
        } else {
            panic!("Expected right to be identifier h");
        }
    } else {
        panic!("Expected pipe-apply expression");
    }
}

#[test]
fn test_composition_associativity_left() {
    // a >> b >> c  => (a >> b) >> c
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("a >> b >> c")
    {
        assert_eq!(op, BinaryOp::ComposeRight);
        // left should be a composition
        if let Expr::Binary {
            op: left_op,
            left: ll,
            right: lr,
            ..
        } = left.as_ref()
        {
            assert_eq!(*left_op, BinaryOp::ComposeRight);
            if let (
                Expr::Literal(Literal::Identifier(a, _)),
                Expr::Literal(Literal::Identifier(b, _)),
            ) = (ll.as_ref(), lr.as_ref())
            {
                assert_eq!(a, "a");
                assert_eq!(b, "b");
            } else {
                panic!("Expected identifiers a and b");
            }
        } else {
            panic!("Expected left to be a composition expression");
        }
        if let Expr::Literal(Literal::Identifier(c, _)) = right.as_ref() {
            assert_eq!(c, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected composition expression");
    }

    // a << b << c  => (a << b) << c
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = parse_expression("a << b << c")
    {
        assert_eq!(op, BinaryOp::ComposeLeft);
        if let Expr::Binary {
            op: left_op,
            left: ll,
            right: lr,
            ..
        } = left.as_ref()
        {
            assert_eq!(*left_op, BinaryOp::ComposeLeft);
            if let (
                Expr::Literal(Literal::Identifier(a, _)),
                Expr::Literal(Literal::Identifier(b, _)),
            ) = (ll.as_ref(), lr.as_ref())
            {
                assert_eq!(a, "a");
                assert_eq!(b, "b");
            } else {
                panic!("Expected identifiers a and b");
            }
        } else {
            panic!("Expected left to be a composition expression");
        }
        if let Expr::Literal(Literal::Identifier(c, _)) = right.as_ref() {
            assert_eq!(c, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected composition expression");
    }
}
