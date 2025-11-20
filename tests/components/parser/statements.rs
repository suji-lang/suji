use suji_ast::ExportBody;
use suji_ast::{BinaryOp, CompoundOp, Expr, Literal, Stmt};

use super::common::{parse_program, parse_statement};

#[test]
fn test_parse_return_statement() {
    let result = parse_statement("return 42");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Return { values, .. })) = result {
        assert_eq!(values.len(), 1);
        if let Expr::Literal(Literal::Number(n, _)) = &values[0] {
            assert_eq!(n, "42");
        } else {
            panic!("Expected number in return");
        }
    } else {
        panic!("Expected return expression statement");
    }
}

#[test]
fn test_parse_export_expression() {
    let result = parse_statement("export 42");
    assert!(result.is_ok());

    if let Ok(Stmt::Export { body, .. }) = result {
        match body {
            ExportBody::Expr(Expr::Literal(Literal::Number(n, _))) => {
                assert_eq!(n, "42");
            }
            _ => panic!("Expected export expression body with number"),
        }
    } else {
        panic!("Expected export statement");
    }
}

#[test]
fn test_parse_export_map() {
    let result = parse_statement("export { x: 1, y: 2 }");
    assert!(result.is_ok());

    if let Ok(Stmt::Export { body, .. }) = result {
        match body {
            ExportBody::Map(spec) => {
                assert_eq!(spec.items.len(), 2);
            }
            _ => panic!("Expected map export body"),
        }
    } else {
        panic!("Expected export statement");
    }
}

#[test]
fn test_parse_return_without_value() {
    let result = parse_statement("return");
    assert!(result.is_ok());

    if let Ok(Stmt::Expr(Expr::Return { values, .. })) = result {
        assert!(values.is_empty());
        // Expected
    } else {
        panic!("Expected return expression statement without value");
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

        if let Stmt::Expr(Expr::Literal(Literal::Number(n, _))) = &statements[0] {
            assert_eq!(*n, "42".to_string());
        } else {
            panic!("Expected first statement to be number");
        }

        if let Stmt::Expr(Expr::Literal(Literal::Boolean(b, _))) = &statements[1] {
            assert!(*b);
        } else {
            panic!("Expected second statement to be boolean");
        }

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
fn test_parse_multiple_exports_error() {
    let src = "export { x: 1 }\nexport 2";
    super::common::assert_parse_fails(
        src,
        "Multiple export statements||Only one export statement is allowed",
    );
}

#[test]
fn test_parse_export_error_no_form() {
    // "export" followed by EOF should fail
    super::common::assert_parse_fails(
        "export",
        "Expected '{' or expression after export||Expected '{' after export",
    );
}
#[test]
fn test_parse_index_with_function_call() {
    let ast = parse_program("list[get_idx()]").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Index { target, index, .. }) = &ast[0] {
        // Target should be identifier "list"
        if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
            assert_eq!(name, "list");
        } else {
            panic!("Expected identifier as target");
        }

        // Index should be a function call
        if let Expr::Call { callee, args, .. } = index.as_ref() {
            if let Expr::Literal(Literal::Identifier(name, _)) = callee.as_ref() {
                assert_eq!(name, "get_idx");
            } else {
                panic!("Expected identifier as callee");
            }
            assert_eq!(args.len(), 0);
        } else {
            panic!("Expected function call as index");
        }
    } else {
        panic!("Expected index expression statement");
    }
}

#[test]
fn test_parse_index_with_method_call() {
    let ast = parse_program("list[nums::length()]").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Index { index, .. }) = &ast[0] {
        // Index should be a method call
        if let Expr::MethodCall { target, method, .. } = index.as_ref() {
            if let Expr::Literal(Literal::Identifier(name, _)) = target.as_ref() {
                assert_eq!(name, "nums");
            } else {
                panic!("Expected identifier as method target");
            }
            assert_eq!(method, "length");
        } else {
            panic!("Expected method call as index");
        }
    } else {
        panic!("Expected index expression statement");
    }
}

#[test]
fn test_parse_index_with_arithmetic() {
    let ast = parse_program("list[i + 1]").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Index { index, .. }) = &ast[0] {
        // Index should be a binary operation
        if let Expr::Binary {
            op: BinaryOp::Add, ..
        } = index.as_ref()
        {
            // Success
        } else {
            panic!("Expected Add binary operation as index");
        }
    } else {
        panic!("Expected index expression statement");
    }
}

#[test]
fn test_parse_index_with_nested_index() {
    let ast = parse_program("list[indices[0]]").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Index {
        target: outer_target,
        index: outer_index,
        ..
    }) = &ast[0]
    {
        // Outer target should be "list"
        if let Expr::Literal(Literal::Identifier(name, _)) = outer_target.as_ref() {
            assert_eq!(name, "list");
        } else {
            panic!("Expected identifier as outer target");
        }

        // Outer index should be another index expression
        if let Expr::Index {
            target: inner_target,
            index: inner_index,
            ..
        } = outer_index.as_ref()
        {
            if let Expr::Literal(Literal::Identifier(name, _)) = inner_target.as_ref() {
                assert_eq!(name, "indices");
            } else {
                panic!("Expected identifier as inner target");
            }
            if let Expr::Literal(Literal::Number(n, _)) = inner_index.as_ref() {
                assert_eq!(n, "0");
            } else {
                panic!("Expected number as inner index");
            }
        } else {
            panic!("Expected nested index expression");
        }
    } else {
        panic!("Expected index expression statement");
    }
}

#[test]
fn test_parse_index_with_pipeline_apply() {
    let ast = parse_program("list[x |> transform]").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Index { index, .. }) = &ast[0] {
        // Index should be a pipeline operation
        if let Expr::Binary {
            op: BinaryOp::PipeApplyFwd,
            ..
        } = index.as_ref()
        {
            // Success
        } else {
            panic!("Expected PipeApplyFwd binary operation as index");
        }
    } else {
        panic!("Expected index expression statement");
    }
}

#[test]
fn test_parse_index_with_method_and_arithmetic() {
    let ast = parse_program("list[nums::length() - 1]").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Index { index, .. }) = &ast[0] {
        // Index should be a subtraction with method call on left
        if let Expr::Binary {
            op: BinaryOp::Subtract,
            left,
            right,
            ..
        } = index.as_ref()
        {
            // Left side should be method call
            if let Expr::MethodCall { method, .. } = left.as_ref() {
                assert_eq!(method, "length");
            } else {
                panic!("Expected method call on left side");
            }
            // Right side should be 1
            if let Expr::Literal(Literal::Number(n, _)) = right.as_ref() {
                assert_eq!(n, "1");
            } else {
                panic!("Expected number on right side");
            }
        } else {
            panic!("Expected Subtract binary operation as index");
        }
    } else {
        panic!("Expected index expression statement");
    }
}

#[test]
fn test_parse_slice_still_works() {
    // Test that slice syntax is unaffected by our changes
    let test_cases = vec![
        ("list[1:3]", "simple slice"),
        ("list[:3]", "slice from start"),
        ("list[1:]", "slice to end"),
        ("list[:]", "full slice"),
    ];

    for (code, description) in test_cases {
        let ast =
            parse_program(code).unwrap_or_else(|_| panic!("Parsing failed for {}", description));

        assert_eq!(ast.len(), 1);
        if let Stmt::Expr(Expr::Slice { .. }) = &ast[0] {
            // Success - we got a slice expression
        } else {
            panic!("Expected slice expression for {}", description);
        }
    }
}

#[test]
fn test_parse_index_assignment_with_complex_expression() {
    let ast = parse_program("list[get_idx()] = 42").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Assign { target, .. }) = &ast[0] {
        if let Expr::Index { index, .. } = target.as_ref() {
            // Index should be a function call
            if let Expr::Call { .. } = index.as_ref() {
                // Success
            } else {
                panic!("Expected function call as index");
            }
        } else {
            panic!("Expected index expression as assignment target");
        }
    } else {
        panic!("Expected assignment expression statement");
    }
}

#[test]
fn test_parse_chain_multiple_complex_indices() {
    let ast = parse_program("data[func()][method()]").expect("Parsing failed");

    assert_eq!(ast.len(), 1);
    if let Stmt::Expr(Expr::Index {
        target: outer_target,
        index: outer_index,
        ..
    }) = &ast[0]
    {
        // Outer target should be another index expression
        if let Expr::Index { index, .. } = outer_target.as_ref() {
            // Inner index should be a function call
            if let Expr::Call { .. } = index.as_ref() {
                // Success
            } else {
                panic!("Expected function call as inner index");
            }
        } else {
            panic!("Expected index expression as outer target");
        }

        // Outer index should be a function call
        if let Expr::Call { .. } = outer_index.as_ref() {
            // Success
        } else {
            panic!("Expected function call as outer index");
        }
    } else {
        panic!("Expected index expression statement");
    }
}

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
