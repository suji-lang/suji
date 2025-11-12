use suji_ast::Span;
use suji_ast::*;

#[test]
fn test_expr_span_retrieval() {
    let span = Span::new(0, 5, 1, 0);
    let expr = Expr::Literal(Literal::Number("42".to_string(), span.clone()));
    assert_eq!(expr.span(), &span);
}

#[test]
fn test_expr_is_assignable() {
    let span = Span::new(0, 5, 1, 0);

    // Identifier should be assignable
    let identifier = Expr::Literal(Literal::Identifier("x".to_string(), span.clone()));
    assert!(identifier.is_assignable());

    // Index should be assignable
    let index = Expr::Index {
        target: Box::new(identifier.clone()),
        index: Box::new(Expr::Literal(Literal::Number(
            "0".to_string(),
            span.clone(),
        ))),
        span: span.clone(),
    };
    assert!(index.is_assignable());

    // Map access should be assignable
    let map_access = Expr::MapAccessByName {
        target: Box::new(identifier),
        key: "key".to_string(),
        span: span.clone(),
    };
    assert!(map_access.is_assignable());

    // Number literal should not be assignable
    let number = Expr::Literal(Literal::Number("42".to_string(), span));
    assert!(!number.is_assignable());
}

#[test]
fn test_stmt_has_control_flow() {
    let span = Span::new(0, 5, 1, 0);

    // Return expression has control flow
    let return_expr = Expr::Return {
        values: Vec::new(),
        span: span.clone(),
    };
    let return_stmt = Stmt::Expr(return_expr);
    assert!(return_stmt.has_control_flow());

    // Break expression has control flow
    let break_expr = Expr::Break {
        label: None,
        span: span.clone(),
    };
    let break_stmt = Stmt::Expr(break_expr);
    assert!(break_stmt.has_control_flow());

    // Continue expression has control flow
    let continue_expr = Expr::Continue {
        label: None,
        span: span.clone(),
    };
    let continue_stmt = Stmt::Expr(continue_expr);
    assert!(continue_stmt.has_control_flow());

    // Expression statement without control flow
    let expr_stmt = Stmt::Expr(Expr::Literal(Literal::Number(
        "42".to_string(),
        span.clone(),
    )));
    assert!(!expr_stmt.has_control_flow());

    // Block with control flow inside
    let return_expr_for_block = Expr::Return {
        values: Vec::new(),
        span: span.clone(),
    };
    let block_with_control = Stmt::Block {
        statements: vec![Stmt::Expr(return_expr_for_block)],
        span: span.clone(),
    };
    assert!(block_with_control.has_control_flow());
}
