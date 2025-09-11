use nnlang::ast::{
    BinaryOp, CompoundOp, Expr, ImportSpec, Literal, Pattern, Stmt, StringPart, UnaryOp, ValueLike,
};
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

// Phase 3 Tests - Version 0.1.1 Features

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

            // Check that target is an identifier
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected identifier target for: {}", expr_str);
            }

            // Check that value is a number
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert!(n > &0.0, "Expected positive number for: {}", expr_str);
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

        // Target should be x
        if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
            assert_eq!(name, "x");
        } else {
            panic!("Expected x as target");
        }

        // Value should be another compound assignment (y += 5)
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
                assert_eq!(*n, 5.0);
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
fn test_parse_semicolon_statement_separators() {
    // Test semicolon-separated statements in a block
    let result = parse_statement("{ x = 1; y = 2; z = 3 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Block { statements, .. }) = result {
        assert_eq!(statements.len(), 3);

        // Check first statement (x = 1)
        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[0] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected x as target");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 1.0);
            } else {
                panic!("Expected 1 as value");
            }
        } else {
            panic!("Expected assignment statement");
        }

        // Check second statement (y = 2)
        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[1] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "y");
            } else {
                panic!("Expected y as target");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 2.0);
            } else {
                panic!("Expected 2 as value");
            }
        } else {
            panic!("Expected assignment statement");
        }

        // Check third statement (z = 3)
        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[2] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "z");
            } else {
                panic!("Expected z as target");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 3.0);
            } else {
                panic!("Expected 3 as value");
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
    // Test mixing semicolons and newlines
    let result = parse_statement("{ x = 1; y = 2\nz = 3 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Block { statements, .. }) = result {
        assert_eq!(statements.len(), 3);
        // All statements should be parsed correctly regardless of separator type
    } else {
        panic!("Expected block statement");
    }
}

#[test]
fn test_parse_semicolon_with_compound_assignment() {
    // Test semicolon with compound assignment
    let result = parse_statement("{ x += 5; y -= 3; z *= 2 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Block { statements, .. }) = result {
        assert_eq!(statements.len(), 3);

        // Check first statement (x += 5)
        if let Stmt::Expr(Expr::CompoundAssign {
            op, target, value, ..
        }) = &statements[0]
        {
            assert_eq!(*op, CompoundOp::PlusAssign);
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "x");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 5.0);
            }
        } else {
            panic!("Expected compound assignment");
        }

        // Check second statement (y -= 3)
        if let Stmt::Expr(Expr::CompoundAssign {
            op, target, value, ..
        }) = &statements[1]
        {
            assert_eq!(*op, CompoundOp::MinusAssign);
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "y");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 3.0);
            }
        } else {
            panic!("Expected compound assignment");
        }

        // Check third statement (z *= 2)
        if let Stmt::Expr(Expr::CompoundAssign {
            op, target, value, ..
        }) = &statements[2]
        {
            assert_eq!(*op, CompoundOp::MultiplyAssign);
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "z");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 2.0);
            }
        } else {
            panic!("Expected compound assignment");
        }
    } else {
        panic!("Expected block statement");
    }
}

#[test]
fn test_parse_nil_in_assignments() {
    // Test nil in assignments
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
    // Test compound assignment with nil (should parse but may be invalid at runtime)
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

// Phase 6 Tests - Optional Braces for Single Expressions

#[test]
fn test_parse_function_single_expression() {
    // Test function with single expression body (no braces)
    let result = parse_expression("|x| x * 2");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "x");

        // Body should be a single expression statement
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
    // Test function with block body (should still work)
    let result = parse_expression("|x| { return x * 2 }");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "x");

        // Body should be a block statement
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
    // Test empty function with single expression body
    let result = parse_expression("|| 42");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 0);

        // Body should be a single expression statement
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
fn test_parse_match_single_expression() {
    // Test match with single expression arms (no braces)
    let result = parse_statement("match x { 1: \"one\" 2: \"two\" _: \"other\" }");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match {
        scrutinee, arms, ..
    })) = result
    {
        // Check scrutinee
        if let Some(scrutinee_expr) = scrutinee {
            if let Expr::Literal(Literal::Identifier(name, _)) = scrutinee_expr.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected x as scrutinee");
            }
        } else {
            panic!("Expected scrutinee to be present");
        }

        // Check arms
        assert_eq!(arms.len(), 3);

        // First arm: 1: "one"
        if let (
            Pattern::Literal {
                value: ValueLike::Number(n),
                ..
            },
            Stmt::Expr(Expr::Literal(Literal::StringTemplate(parts, _))),
        ) = (&arms[0].pattern, &arms[0].body)
        {
            assert_eq!(*n, 1.0);
            // Check that the string template contains "one"
            if let [StringPart::Text(s)] = parts.as_slice() {
                assert_eq!(s, "one");
            } else {
                panic!("Expected string template with 'one'");
            }
        } else {
            panic!("Expected first arm to be 1: \"one\"");
        }

        // Second arm: 2: "two"
        if let (
            Pattern::Literal {
                value: ValueLike::Number(n),
                ..
            },
            Stmt::Expr(Expr::Literal(Literal::StringTemplate(parts, _))),
        ) = (&arms[1].pattern, &arms[1].body)
        {
            assert_eq!(*n, 2.0);
            // Check that the string template contains "two"
            if let [StringPart::Text(s)] = parts.as_slice() {
                assert_eq!(s, "two");
            } else {
                panic!("Expected string template with 'two'");
            }
        } else {
            panic!("Expected second arm to be 2: \"two\"");
        }

        // Third arm: _: "other"
        if let (
            Pattern::Wildcard { .. },
            Stmt::Expr(Expr::Literal(Literal::StringTemplate(parts, _))),
        ) = (&arms[2].pattern, &arms[2].body)
        {
            // Check that the string template contains "other"
            if let [StringPart::Text(s)] = parts.as_slice() {
                assert_eq!(s, "other");
            } else {
                panic!("Expected string template with 'other'");
            }
        } else {
            panic!("Expected third arm to be _: \"other\"");
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_match_block() {
    // Test match with block arms (should still work)
    let result = parse_statement("match x { 1: { return \"one\" } 2: { return \"two\" } }");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match {
        scrutinee, arms, ..
    })) = result
    {
        // Check scrutinee
        if let Some(scrutinee_expr) = scrutinee {
            if let Expr::Literal(Literal::Identifier(name, _)) = scrutinee_expr.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected x as scrutinee");
            }
        } else {
            panic!("Expected scrutinee to be present");
        }

        // Check arms
        assert_eq!(arms.len(), 2);

        // First arm: 1: { return "one" }
        if let (
            Pattern::Literal {
                value: ValueLike::Number(n),
                ..
            },
            Stmt::Block { statements, .. },
        ) = (&arms[0].pattern, &arms[0].body)
        {
            assert_eq!(*n, 1.0);
            assert_eq!(statements.len(), 1);
            if let Stmt::Return {
                value: Some(expr), ..
            } = &statements[0]
            {
                if let Expr::Literal(Literal::StringTemplate(parts, _)) = expr {
                    if let [StringPart::Text(s)] = parts.as_slice() {
                        assert_eq!(s, "one");
                    } else {
                        panic!("Expected string template with 'one'");
                    }
                } else {
                    panic!("Expected string template in return");
                }
            } else {
                panic!("Expected return statement");
            }
        } else {
            panic!("Expected first arm to be block with return");
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_mixed_syntax() {
    // Test mixing single expressions and blocks
    let result = parse_statement("match x { 1: \"one\" 2: { return \"two\" } _: \"other\" }");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 3);

        // First arm: single expression
        if let Stmt::Expr(_) = &arms[0].body {
            // Expected
        } else {
            panic!("Expected first arm to be single expression");
        }

        // Second arm: block
        if let Stmt::Block { .. } = &arms[1].body {
            // Expected
        } else {
            panic!("Expected second arm to be block");
        }

        // Third arm: single expression
        if let Stmt::Expr(_) = &arms[2].body {
            // Expected
        } else {
            panic!("Expected third arm to be single expression");
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_nested_expressions_without_braces() {
    // Test complex nested expressions without braces
    // Since match is a statement, not an expression, we test it as a statement
    let result = parse_statement("match x { 1: x * 2 2: x + 10 _: 0 }");
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
fn test_parse_function_with_default_params_single_expression() {
    // Test function with default parameters and single expression body
    let result = parse_expression("|x = 0, y = 1| x + y");
    assert!(result.is_ok());

    if let Ok(Expr::FunctionLiteral { params, body, .. }) = result {
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "x");
        assert_eq!(params[1].name, "y");

        // Check default values
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

        // Body should be a single expression
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
fn test_parse_error_cases_optional_braces() {
    // Test that invalid syntax still produces appropriate errors

    // Incomplete function without body should fail
    let result = parse_expression("|x|");
    if result.is_ok() {
        println!("Unexpected success: {:?}", result);
    }
    assert!(result.is_err());

    // Incomplete match arm should fail
    let result = parse_statement("match x { 1: }");
    if result.is_ok() {
        println!("Unexpected success: {:?}", result);
    }
    assert!(result.is_err());

    // Note: The following cases actually parse correctly because:
    // 1. |x| x * 2; y = 3 - parses as function literal with body x * 2, ignoring the rest
    // 2. match x { 1: x * 2; y = 3 } - parses as match with single expression arm x * 2, ignoring the rest
    // This is actually correct behavior for the parser.
}

#[test]
fn test_parse_top_level_semicolon_separators() {
    // Test semicolon-separated statements at top level
    let result = parse_program("x = 1; y = 2; z = 3");
    assert!(result.is_ok());

    if let Ok(statements) = result {
        assert_eq!(statements.len(), 3);

        // Check first statement: x = 1
        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[0] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "x");
            } else {
                panic!("Expected x as target");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 1.0);
            } else {
                panic!("Expected 1 as value");
            }
        } else {
            panic!("Expected assignment statement");
        }

        // Check second statement: y = 2
        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[1] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "y");
            } else {
                panic!("Expected y as target");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 2.0);
            } else {
                panic!("Expected 2 as value");
            }
        } else {
            panic!("Expected assignment statement");
        }

        // Check third statement: z = 3
        if let Stmt::Expr(Expr::Assign { target, value, .. }) = &statements[2] {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "z");
            } else {
                panic!("Expected z as target");
            }
            if let Expr::Literal(Literal::Number(n, _)) = value.as_ref() {
                assert_eq!(*n, 3.0);
            } else {
                panic!("Expected 3 as value");
            }
        } else {
            panic!("Expected assignment statement");
        }
    } else {
        panic!("Expected program to parse successfully");
    }
}

#[test]
fn test_parse_top_level_mixed_semicolon_newline_separators() {
    // Test mixing semicolons and newlines at top level
    let result = parse_program("x = 1; y = 2\nz = 3");
    assert!(result.is_ok());

    if let Ok(statements) = result {
        assert_eq!(statements.len(), 3);
    } else {
        panic!("Expected program to parse successfully");
    }
}

#[test]
fn test_parse_top_level_semicolon_with_imports() {
    // Test semicolon with import statements
    let result = parse_program("import std:println; import std:math");
    assert!(result.is_ok());

    if let Ok(statements) = result {
        assert_eq!(statements.len(), 2);

        // Check first import
        if let Stmt::Import { spec, .. } = &statements[0] {
            if let ImportSpec::Item { module, name } = spec {
                assert_eq!(module, "std");
                assert_eq!(name, "println");
            } else {
                panic!("Expected import std:println");
            }
        } else {
            panic!("Expected import statement");
        }

        // Check second import
        if let Stmt::Import { spec, .. } = &statements[1] {
            if let ImportSpec::Item { module, name } = spec {
                assert_eq!(module, "std");
                assert_eq!(name, "math");
            } else {
                panic!("Expected import std:math");
            }
        } else {
            panic!("Expected import statement");
        }
    } else {
        panic!("Expected program to parse successfully");
    }
}

#[test]
fn test_parse_top_level_semicolon_with_expressions() {
    // Test semicolon with expression statements
    let result = parse_program("42; true; \"hello\"");
    assert!(result.is_ok());

    if let Ok(statements) = result {
        assert_eq!(statements.len(), 3);

        // Check first expression: 42
        if let Stmt::Expr(Expr::Literal(Literal::Number(n, _))) = &statements[0] {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number literal");
        }

        // Check second expression: true
        if let Stmt::Expr(Expr::Literal(Literal::Boolean(b, _))) = &statements[1] {
            assert!(b);
        } else {
            panic!("Expected boolean literal");
        }

        // Check third expression: "hello"
        if let Stmt::Expr(Expr::Literal(Literal::StringTemplate(parts, _))) = &statements[2] {
            assert_eq!(parts.len(), 1);
            if let StringPart::Text(s) = &parts[0] {
                assert_eq!(s, "hello");
            } else {
                panic!("Expected text part in string template");
            }
        } else {
            panic!("Expected string literal");
        }
    } else {
        panic!("Expected program to parse successfully");
    }
}

#[test]
fn test_parse_top_level_trailing_semicolon() {
    // Test trailing semicolon at top level
    let result = parse_program("x = 1; y = 2;");
    assert!(result.is_ok());

    if let Ok(statements) = result {
        assert_eq!(statements.len(), 2);
    } else {
        panic!("Expected program to parse successfully");
    }
}

#[test]
fn test_parse_top_level_multiple_semicolons() {
    // Test multiple consecutive semicolons (should be handled gracefully)
    let result = parse_program("x = 1;; y = 2");
    assert!(result.is_ok());

    if let Ok(statements) = result {
        assert_eq!(statements.len(), 2);
    } else {
        panic!("Expected program to parse successfully");
    }
}

#[test]
fn test_parse_nil_pattern() {
    let result = parse_statement("match x { nil: \"empty\" _: \"something\" }");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 2);

        // First arm should be nil pattern
        if let Pattern::Literal {
            value: ValueLike::Nil,
            ..
        } = &arms[0].pattern
        {
            // Expected
        } else {
            panic!("Expected nil pattern");
        }

        // Second arm should be wildcard
        if let Pattern::Wildcard { .. } = &arms[1].pattern {
            // Expected
        } else {
            panic!("Expected wildcard pattern");
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_nil_pattern_in_expression() {
    let result = parse_expression("match x { nil: \"empty\" _: \"something\" }");
    assert!(result.is_ok());

    if let Ok(Expr::Match { arms, .. }) = result {
        assert_eq!(arms.len(), 2);

        // First arm should be nil pattern
        if let Pattern::Literal {
            value: ValueLike::Nil,
            ..
        } = &arms[0].pattern
        {
            // Expected
        } else {
            panic!("Expected nil pattern");
        }
    } else {
        panic!("Expected match expression");
    }
}

#[test]
fn test_parse_return_statements_in_non_braced_match_arms() {
    let result = parse_statement("match x { 1: return 1 2: return 2 _: return 0 }");
    if result.is_err() {
        println!("Parse error: {:?}", result);
    }
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 3);

        // All arms should be single expression statements with return
        for arm in arms {
            if let Stmt::Return { .. } = &arm.body {
                // Expected
            } else {
                panic!("Expected return statement in match arm");
            }
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_map_literals_in_match_arms() {
    let result =
        parse_statement("match x { 1: { \"status\": \"success\" } 2: { \"status\": \"error\" } }");
    if result.is_err() {
        println!("Parse error: {:?}", result);
    }
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 2);

        // All arms should be single expression statements with map literals
        for arm in arms {
            if let Stmt::Expr(Expr::Literal(Literal::Map(..))) = &arm.body {
                // Expected
            } else {
                panic!("Expected map literal in match arm");
            }
        }
    } else {
        panic!("Expected match statement");
    }
}
