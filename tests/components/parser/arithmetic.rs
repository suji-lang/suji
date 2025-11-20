use suji_ast::{BinaryOp, CompoundOp, Expr, Literal, Stmt, UnaryOp};

use super::common::{parse_expression, parse_statement};

#[test]
fn test_parse_arithmetic_expression() {
    let result = parse_expression("3 + 4");
    assert!(result.is_ok());

    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = result
    {
        if let (Expr::Literal(Literal::Number(l, _)), Expr::Literal(Literal::Number(r, _))) =
            (left.as_ref(), right.as_ref())
        {
            assert_eq!(*l, "3".to_string());
            assert_eq!(*r, "4".to_string());
            assert_eq!(op, BinaryOp::Add);
        } else {
            panic!("Expected number operands");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_parse_pipe_associativity() {
    let result = parse_expression("a | b | c");
    assert!(result.is_ok());
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = result
    {
        assert_eq!(op, BinaryOp::Pipe);
        // Left should itself be a pipe: (a | b)
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::Pipe);
        } else {
            panic!("Expected left to be a pipe expression");
        }
        // Right should be a primary (identifier c)
        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected binary pipe expression");
    }
}

#[test]
fn test_parse_multiplication_precedence() {
    let result = parse_expression("2 + 3 * 4");
    assert!(result.is_ok());

    // Should parse as 2 + (3 * 4)
    if let Ok(Expr::Binary {
        left, op, right, ..
    }) = result
    {
        assert_eq!(op, BinaryOp::Add);

        if let Expr::Literal(Literal::Number(l, _)) = left.as_ref() {
            assert_eq!(*l, "2".to_string());
        } else {
            panic!("Expected left operand to be 2");
        }

        if let Expr::Binary {
            left: inner_left,
            op: inner_op,
            right: inner_right,
            ..
        } = right.as_ref()
        {
            assert_eq!(*inner_op, BinaryOp::Multiply);
            if let (Expr::Literal(Literal::Number(l, _)), Expr::Literal(Literal::Number(r, _))) =
                (inner_left.as_ref(), inner_right.as_ref())
            {
                assert_eq!(*l, "3".to_string());
                assert_eq!(*r, "4".to_string());
            } else {
                panic!("Expected multiplication operands");
            }
        } else {
            panic!("Expected right operand to be multiplication");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_parse_unary_expression() {
    let result = parse_expression("-42");
    assert!(result.is_ok());

    if let Ok(Expr::Unary { op, expr, .. }) = result {
        assert_eq!(op, UnaryOp::Negate);
        if let Expr::Literal(Literal::Number(n, _)) = expr.as_ref() {
            assert_eq!(*n, "42".to_string());
        } else {
            panic!("Expected number operand");
        }
    } else {
        panic!("Expected unary expression");
    }
}

#[test]
fn test_parse_grouping() {
    let result = parse_expression("(3 + 4) * 2");
    assert!(result.is_ok());

    // Should parse as (3 + 4) * 2
    if let Ok(Expr::Binary {
        left, op, right: _, ..
    }) = result
    {
        assert_eq!(op, BinaryOp::Multiply);

        if let Expr::Grouping { expr, .. } = left.as_ref() {
            if let Expr::Binary {
                left: inner_left,
                op: inner_op,
                right: inner_right,
                ..
            } = expr.as_ref()
            {
                assert_eq!(*inner_op, BinaryOp::Add);
                if let (
                    Expr::Literal(Literal::Number(l, _)),
                    Expr::Literal(Literal::Number(r, _)),
                ) = (inner_left.as_ref(), inner_right.as_ref())
                {
                    assert_eq!(*l, "3".to_string());
                    assert_eq!(*r, "4".to_string());
                } else {
                    panic!("Expected addition operands");
                }
            } else {
                panic!("Expected addition in grouping");
            }
        } else {
            panic!("Expected left operand to be grouping");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_parse_comparison_operators() {
    let operators = vec![
        ("3 < 4", BinaryOp::Less),
        ("3 <= 4", BinaryOp::LessEqual),
        ("3 > 4", BinaryOp::Greater),
        ("3 >= 4", BinaryOp::GreaterEqual),
        ("3 == 4", BinaryOp::Equal),
        ("3 != 4", BinaryOp::NotEqual),
    ];

    for (expr_str, expected_op) in operators {
        let result = parse_expression(expr_str);
        assert!(result.is_ok(), "Failed to parse: {}", expr_str);

        if let Ok(Expr::Binary { op, .. }) = result {
            assert_eq!(op, expected_op, "Wrong operator for: {}", expr_str);
        } else {
            panic!("Expected binary expression for: {}", expr_str);
        }
    }
}

#[test]
fn test_parse_nested_expressions_without_braces() {
    // Since match is a statement, not an expression, we test it as a statement
    let result = parse_statement("match x { 1 => x * 2, 2 => x + 10, _ => 0, }");
    if result.is_err() {
        println!("Parse error: {:?}", result);
    }
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match {
        scrutinee, arms, ..
    })) = result
    {
        if let Some(scrutinee_expr) = scrutinee {
            if let Expr::Literal(Literal::Identifier(name, _)) = scrutinee_expr.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected x as scrutinee");
            }
        } else {
            panic!("Expected scrutinee to be present");
        }

        assert_eq!(arms.len(), 3);

        // All arms should be single expressions
        for arm in arms {
            if let Stmt::Expr(_) = &arm.body {
                // Expected
            } else {
                panic!("Expected all arms to be single expressions");
            }
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn power_is_right_associative() {
    let result = parse_expression("a ^ b ^ c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::Power);
        if let Expr::Binary { op: right_op, .. } = right.as_ref() {
            assert_eq!(*right_op, BinaryOp::Power);
        } else {
            panic!("Expected right to be a power expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
            assert_eq!(name, "a");
        } else {
            panic!("Expected left to be identifier a");
        }
    } else {
        panic!("Expected top-level power expression");
    }
}

#[test]
fn backward_apply_is_right_associative() {
    let result = parse_expression("a <| b <| c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::PipeApplyBwd);
        if let Expr::Binary { op: right_op, .. } = right.as_ref() {
            assert_eq!(right_op, &BinaryOp::PipeApplyBwd);
        } else {
            panic!("Expected right to be a backward-apply expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = left.as_ref() {
            assert_eq!(name, "a");
        } else {
            panic!("Expected left to be identifier a");
        }
    } else {
        panic!("Expected top-level backward-apply expression");
    }
}

#[test]
fn subtraction_is_left_associative() {
    let result = parse_expression("a - b - c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::Subtract);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::Subtract);
        } else {
            panic!("Expected left to be a subtraction expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected top-level subtraction expression");
    }
}

#[test]
fn stream_pipe_is_left_associative() {
    let result = parse_expression("a | b | c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::Pipe);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::Pipe);
        } else {
            panic!("Expected left to be a stream pipe expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected top-level stream pipe expression");
    }
}

#[test]
fn forward_apply_is_left_associative() {
    let result = parse_expression("a |> b |> c");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary {
        left, op, right, ..
    } = expr
    {
        assert_eq!(op, BinaryOp::PipeApplyFwd);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::PipeApplyFwd);
        } else {
            panic!("Expected left to be a forward-apply expression");
        }
        if let Expr::Literal(Literal::Identifier(name, _)) = right.as_ref() {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right to be identifier c");
        }
    } else {
        panic!("Expected top-level forward-apply expression");
    }
}

#[test]
fn composition_binds_tighter_than_pipe_apply() {
    let result = parse_expression("x |> f >> g");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary { op, right, .. } = expr {
        assert_eq!(op, BinaryOp::PipeApplyFwd);
        // Right should be a composition binary
        if let Expr::Binary { op: right_op, .. } = right.as_ref() {
            assert!(matches!(
                right_op,
                BinaryOp::ComposeRight | BinaryOp::ComposeLeft
            ));
        } else {
            panic!("Expected right to be a composition expression");
        }
    } else {
        panic!("Expected top-level forward-apply expression");
    }
}

#[test]
fn composition_left_assoc_chain() {
    let result = parse_expression("f >> g >> h");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary { left, op, .. } = expr {
        assert_eq!(op, BinaryOp::ComposeRight);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::ComposeRight);
        } else {
            panic!("Expected left to be a composition expression");
        }
    } else {
        panic!("Expected top-level composition expression");
    }
}

#[test]
fn composition_and_pipe_interaction() {
    let result = parse_expression("f << g |> h");
    assert!(result.is_ok());
    let expr = result.unwrap();

    if let Expr::Binary { left, op, .. } = expr {
        assert_eq!(op, BinaryOp::PipeApplyFwd);
        if let Expr::Binary { op: left_op, .. } = left.as_ref() {
            assert_eq!(*left_op, BinaryOp::ComposeLeft);
        } else {
            panic!("Expected left to be a left-composition expression");
        }
    } else {
        panic!("Expected top-level forward-apply expression");
    }
}

#[test]
fn test_parse_compound_assignment_operators() {
    let operators = vec![
        ("x += 5", CompoundOp::PlusAssign),
        ("x -= 3", CompoundOp::MinusAssign),
        ("x *= 2", CompoundOp::MultiplyAssign),
        ("x /= 4", CompoundOp::DivideAssign),
        ("x %= 7", CompoundOp::ModuloAssign),
    ];

    for (expr_str, expected_op) in operators {
        let result = parse_expression(expr_str);
        assert!(result.is_ok(), "Failed to parse: {}", expr_str);

        if let Ok(Expr::CompoundAssign {
            op, target, value, ..
        }) = result
        {
            assert_eq!(op, expected_op, "Wrong operator for: {}", expr_str);

            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected identifier target for: {}", expr_str);
            }

            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert!(
                    n.parse::<f64>().unwrap() > 0.0,
                    "Expected positive number for: {}",
                    expr_str
                );
            } else {
                panic!("Expected number value for: {}", expr_str);
            }
        } else {
            panic!("Expected compound assignment for: {}", expr_str);
        }
    }
}

#[test]
fn test_parse_compound_assignment_precedence() {
    // Test that compound assignment is right-associative
    let result = parse_expression("x += y += 5");
    assert!(result.is_ok());

    if let Ok(Expr::CompoundAssign {
        op, target, value, ..
    }) = result
    {
        assert_eq!(op, CompoundOp::PlusAssign);

        if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
            assert_eq!(name, "x");
        } else {
            panic!("Expected x as target");
        }

        if let Expr::CompoundAssign {
            op: inner_op,
            target: inner_target,
            value: inner_value,
            ..
        } = value.as_ref()
        {
            assert_eq!(*inner_op, CompoundOp::PlusAssign);

            if let Expr::Literal(Literal::Identifier(inner_name, _)) = inner_target.as_ref() {
                assert_eq!(inner_name, "y");
            } else {
                panic!("Expected y as inner target");
            }

            if let Expr::Literal(Literal::Number(n, _)) = inner_value.as_ref() {
                assert_eq!(*n, "5".to_string());
            } else {
                panic!("Expected 5 as inner value");
            }
        } else {
            panic!("Expected inner compound assignment");
        }
    } else {
        panic!("Expected compound assignment");
    }
}

#[test]
fn test_parse_nil_in_assignments() {
    let result = parse_expression("x = nil");
    assert!(result.is_ok());

    if let Ok(Expr::Assign { target, value, .. }) = result {
        if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
            assert_eq!(name, "x");
        } else {
            panic!("Expected x as target");
        }
        if let Expr::Literal(Literal::Nil(_)) = value.as_ref() {
            // Expected
        } else {
            panic!("Expected nil as value");
        }
    } else {
        panic!("Expected assignment");
    }
}

#[test]
fn test_parse_compound_assignment_with_nil() {
    let result = parse_expression("x += nil");
    assert!(result.is_ok());

    if let Ok(Expr::CompoundAssign {
        op, target, value, ..
    }) = result
    {
        assert_eq!(op, CompoundOp::PlusAssign);
        if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
            assert_eq!(name, "x");
        } else {
            panic!("Expected x as target");
        }
        if let Expr::Literal(Literal::Nil(_)) = value.as_ref() {
            // Expected
        } else {
            panic!("Expected nil as value");
        }
    } else {
        panic!("Expected compound assignment");
    }
}
