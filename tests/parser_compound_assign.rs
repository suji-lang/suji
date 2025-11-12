use suji_ast::{CompoundOp, Expr, Literal};
mod common;
use common::parse_expression;

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
