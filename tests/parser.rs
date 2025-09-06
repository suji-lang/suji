use nnlang::ast::{BinaryOp, Expr, Literal, Stmt, UnaryOp};
use nnlang::parser::{parse_expression, parse_program, parse_statement};

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
            assert_eq!(*l, 3.0);
            assert_eq!(*r, 4.0);
            assert_eq!(op, BinaryOp::Add);
        } else {
            panic!("Expected number operands");
        }
    } else {
        panic!("Expected binary expression");
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
            assert_eq!(*l, 2.0);
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
                assert_eq!(*l, 3.0);
                assert_eq!(*r, 4.0);
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
            assert_eq!(*n, 42.0);
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
                    assert_eq!(*l, 3.0);
                    assert_eq!(*r, 4.0);
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
fn test_parse_return_statement() {
    let result = parse_statement("return 42");
    assert!(result.is_ok());

    if let Ok(Stmt::Return {
        value: Some(expr), ..
    }) = result
    {
        if let Expr::Literal(Literal::Number(n, _)) = expr {
            assert_eq!(n, 42.0);
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

    if let Ok(Stmt::Return { value: None, .. }) = result {
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

        // Check first statement (42)
        if let Stmt::Expr(Expr::Literal(Literal::Number(n, _))) = &statements[0] {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected first statement to be number");
        }

        // Check second statement (true)
        if let Stmt::Expr(Expr::Literal(Literal::Boolean(b, _))) = &statements[1] {
            assert!(*b);
        } else {
            panic!("Expected second statement to be boolean");
        }

        // Check third statement (variable)
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
fn test_error_handling() {
    // Test invalid expression
    let result = parse_expression("+ 42");
    assert!(result.is_err());

    // Test unclosed parenthesis
    let result = parse_expression("(42");
    assert!(result.is_err());
}
