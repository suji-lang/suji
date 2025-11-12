use suji_ast::*;
use suji_parser::parse_program;

mod common;

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
