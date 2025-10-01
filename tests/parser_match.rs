use nnlang::ast::{Expr, Literal, Pattern, Stmt, StringPart, ValueLike};
mod common;
use common::{parse_expression, parse_statement};

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
    let result = parse_statement("match x { 1 => { return \"one\" }, 2 => { return \"two\" }, }");
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
            if let Stmt::Return { values, .. } = &statements[0] {
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
            if let Stmt::Return { .. } = &arm.body {
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
    // Missing comma after first arm
    let result = parse_statement("match x { 1 => \"one\" 2 => \"two\" }");
    assert!(result.is_err());
}

#[test]
fn test_parse_match_error_missing_comma_last_arm() {
    // Missing comma after last arm
    let result = parse_statement("match x { 1 => \"one\", 2 => \"two\" }");
    assert!(result.is_err());
}

#[test]
fn test_parse_match_error_legacy_colon_syntax() {
    // Using old : syntax instead of =>
    let result = parse_statement("match x { 1: \"one\", 2: \"two\", }");
    assert!(result.is_err());
}
