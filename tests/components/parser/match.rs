use suji_ast::{Expr, Literal, Pattern, Stmt, StringPart, ValueLike};

use super::common::{eval_program, parse_expression, parse_statement};
use suji_values::Value;

#[test]
fn match_alternation_expands_to_multiple_arms() {
    let src = r#"
match x {
    1 | 2 | 3 => "ok",
    _ => "no",
}
"#;

    let expr = parse_expression(src).unwrap();
    if let Expr::Match {
        scrutinee, arms, ..
    } = expr
    {
        assert!(scrutinee.is_some());
        // Expect 4 arms total: 3 alternates + 1 wildcard
        assert_eq!(arms.len(), 4);

        // First three arms should be literals 1, 2, 3 with identical bodies
        for (i, expected) in ["1", "2", "3"].iter().enumerate() {
            let arm = &arms[i];
            match &arm.pattern {
                Pattern::Literal { value, .. } => match value {
                    suji_ast::ValueLike::Number(n) => assert_eq!(n, *expected),
                    _ => panic!("Expected numeric literal pattern"),
                },
                _ => panic!("Expected literal pattern"),
            }
            // Body should be Expr("ok")
            match &arm.body {
                Stmt::Expr(Expr::Literal(suji_ast::Literal::StringTemplate(parts, _))) => {
                    if let [suji_ast::StringPart::Text(s)] = parts.as_slice() {
                        assert_eq!(s, "ok");
                    } else {
                        panic!("Expected string template with 'ok'");
                    }
                }
                _ => panic!("Expected literal body 'ok'"),
            }
        }
    } else {
        panic!("Expected match expression");
    }
}

#[test]
fn alternation_missing_following_pattern_reports_error() {
    let src = r#"match x { 1 | => "x", _ => "y", }"#;
    // Parsing should fail with our custom message
    let res = parse_expression(src);
    assert!(res.is_err());
}

#[test]
fn test_parse_match_single_expression() {
    let result = parse_statement("match x { 1 => \"one\", 2 => \"two\", _ => \"other\", }");
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

        if let (
            Pattern::Literal {
                value: ValueLike::Number(n),
                ..
            },
            Stmt::Expr(Expr::Literal(Literal::StringTemplate(parts, _))),
        ) = (&arms[0].pattern, &arms[0].body)
        {
            assert_eq!(*n, "1".to_string());
            if let [StringPart::Text(s)] = parts.as_slice() {
                assert_eq!(s, "one");
            } else {
                panic!("Expected string template with 'one'");
            }
        } else {
            panic!("Expected first arm to be 1: \"one\"");
        }

        if let (
            Pattern::Literal {
                value: ValueLike::Number(n),
                ..
            },
            Stmt::Expr(Expr::Literal(Literal::StringTemplate(parts, _))),
        ) = (&arms[1].pattern, &arms[1].body)
        {
            assert_eq!(*n, "2".to_string());
            if let [StringPart::Text(s)] = parts.as_slice() {
                assert_eq!(s, "two");
            } else {
                panic!("Expected string template with 'two'");
            }
        } else {
            panic!("Expected second arm to be 2: \"two\"");
        }

        if let (
            Pattern::Wildcard { .. },
            Stmt::Expr(Expr::Literal(Literal::StringTemplate(parts, _))),
        ) = (&arms[2].pattern, &arms[2].body)
        {
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
    let result = parse_statement("match x { 1 => { return \"one\" } 2 => { return \"two\" } }");
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

        assert_eq!(arms.len(), 2);

        if let (
            Pattern::Literal {
                value: ValueLike::Number(n),
                ..
            },
            Stmt::Block { statements, .. },
        ) = (&arms[0].pattern, &arms[0].body)
        {
            assert_eq!(*n, "1".to_string());
            assert_eq!(statements.len(), 1);
            if let Stmt::Expr(Expr::Return { values, .. }) = &statements[0] {
                if let [Expr::Literal(Literal::StringTemplate(parts, _))] = values.as_slice() {
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
    let result =
        parse_statement("match x { 1 => \"one\", 2 => { return \"two\" }, _ => \"other\", }");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 3);
        if let Stmt::Expr(_) = &arms[0].body {
        } else {
            panic!("Expected first arm expr");
        }
        if let Stmt::Block { .. } = &arms[1].body {
        } else {
            panic!("Expected second arm block");
        }
        if let Stmt::Expr(_) = &arms[2].body {
        } else {
            panic!("Expected third arm expr");
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_nil_pattern() {
    let result = parse_statement("match x { nil => \"empty\", _ => \"something\", }");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 2);
        if let Pattern::Literal {
            value: ValueLike::Nil,
            ..
        } = &arms[0].pattern
        {
        } else {
            panic!("Expected nil pattern");
        }
        if let Pattern::Wildcard { .. } = &arms[1].pattern {
        } else {
            panic!("Expected wildcard pattern");
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_nil_pattern_in_expression() {
    let result = parse_expression("match x { nil => \"empty\", _ => \"something\", }");
    assert!(result.is_ok());

    if let Ok(Expr::Match { arms, .. }) = result {
        assert_eq!(arms.len(), 2);
        if let Pattern::Literal {
            value: ValueLike::Nil,
            ..
        } = &arms[0].pattern
        {
        } else {
            panic!("Expected nil pattern");
        }
    } else {
        panic!("Expected match expression");
    }
}

#[test]
fn test_parse_return_statements_in_non_braced_match_arms() {
    let result = parse_statement("match x { 1 => return 1, 2 => return 2, _ => return 0, }");
    if result.is_err() {
        println!("Parse error: {:?}", result);
    }
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 3);
        for arm in arms {
            if let Stmt::Expr(Expr::Return { .. }) = &arm.body {
            } else {
                panic!("Expected return expression in match arm");
            }
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_map_literals_in_match_arms() {
    let result = parse_statement(
        "match x { 1 => { \"status\": \"success\" }, 2 => { \"status\": \"error\" }, }",
    );
    if result.is_err() {
        println!("Parse error: {:?}", result);
    }
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 2);
        for arm in arms {
            if let Stmt::Expr(Expr::Literal(Literal::Map(..))) = &arm.body {
            } else {
                panic!("Expected map literal in match arm");
            }
        }
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_match_error_missing_comma() {
    // Missing comma after first arm (single-expression arm still requires comma)
    let result = parse_statement("match x { 1 => \"one\" 2 => \"two\" }");
    assert!(result.is_err());
}

#[test]
fn test_parse_match_error_missing_comma_last_arm() {
    // Missing comma after last arm (single-expression)
    let result = parse_statement("match x { 1 => \"one\", 2 => \"two\" }");
    assert!(result.is_err());
}

#[test]
fn test_parse_match_conditional_braced_no_commas() {
    let result = parse_statement(
        "match { x > 10 => { return \"big\" } x > 0 => { return \"pos\" } _ => { return \"other\" } }",
    );
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match { arms, .. })) = result {
        assert_eq!(arms.len(), 3);
        assert!(matches!(arms[0].body, Stmt::Block { .. }));
        assert!(matches!(arms[1].body, Stmt::Block { .. }));
        assert!(matches!(arms[2].body, Stmt::Block { .. }));
    } else {
        panic!("Expected match statement");
    }
}

#[test]
fn test_parse_match_mixed_braced_optional_and_expr_requires_comma() {
    let result = parse_statement(
        "match x { 1 => { return \"one\" } , 2 => \"two\", _ => { return \"other\" } }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_parse_match_error_legacy_colon_syntax() {
    // Using old : syntax instead of =>
    let result = parse_statement("match x { 1: \"one\", 2: \"two\", }");
    assert!(result.is_err());
}

#[test]
fn test_match_alternation_expands_to_multiple_arms() {
    let src = "match x { 1 => \"ok\", 2 | 3 | 4 => \"ok\", _ => \"no\", }";
    let result = parse_statement(src);
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Match {
        scrutinee, arms, ..
    })) = result
    {
        assert!(scrutinee.is_some());
        // 1 arm for 1, three for 2|3|4, one fallback => total 5 arms
        assert_eq!(arms.len(), 5);
    } else {
        panic!("Expected match expression statement");
    }
}

#[test]
fn test_regex_not_split_by_alternation_bar() {
    // Ensure /a|b/ remains a single regex pattern, not alternation split
    let src = "match s { /a|b/ => 1, _ => 0, }";
    let result = parse_statement(src);
    assert!(result.is_ok());
}
#[test]
fn test_match_single_negative_literal() {
    let source = r#"
        x = -5
        result = match x {
            -5 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_negative_with_alternation() {
    let source = r#"
        x = -10
        result = match x {
            -10 | -5 | 0 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_negative_alternation_middle() {
    let source = r#"
        x = -5
        result = match x {
            -10 | -5 | 0 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_negative_alternation_last() {
    let source = r#"
        x = 0
        result = match x {
            -10 | -5 | 0 => "hit",
            _ => "miss",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("hit".to_string()));
}

#[test]
fn test_match_mixed_positive_and_negative() {
    let source = r#"
        x = -3
        result = match x {
            -5 => "negative five",
            -3 => "negative three",
            0 => "zero",
            3 => "positive three",
            5 => "positive five",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("negative three".to_string()));
}

#[test]
fn test_match_negative_zero() {
    let source = r#"
        x = -0
        result = match x {
            -0 => "negative zero",
            0 => "zero",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    // -0 and 0 should be treated as the same value
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_match_large_negative_number() {
    let source = r#"
        x = -999999
        result = match x {
            -999999 => "large negative",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("large negative".to_string()));
}

#[test]
fn test_match_negative_does_not_match_positive() {
    let source = r#"
        x = 5
        result = match x {
            -5 => "negative",
            _ => "positive or other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("positive or other".to_string()));
}

#[test]
fn test_match_positive_does_not_match_negative() {
    let source = r#"
        x = -5
        result = match x {
            5 => "positive",
            _ => "negative or other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("negative or other".to_string()));
}

#[test]
fn test_match_negative_decimal() {
    let source = r#"
        x = -3.14
        result = match x {
            -3.14 => "pi",
            _ => "other",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("pi".to_string()));
}

#[test]
fn test_match_negative_in_tuple_pattern() {
    let source = r#"
        pair = (-1, 2)
        result = match pair {
            (-1, 2) => "match",
            _ => "no match",
        }
        result
    "#;

    let result = eval_program(source).expect("Evaluation failed");
    assert_eq!(result, Value::String("match".to_string()));
}
