use nnlang::ast::*;
use nnlang::token::Span;

#[test]
fn test_expr_span_retrieval() {
    let span = Span::new(0, 5, 1, 0);
    let expr = Expr::Literal(Literal::Number(42.0, span.clone()));
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
        index: Box::new(Expr::Literal(Literal::Number(0.0, span.clone()))),
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
    let number = Expr::Literal(Literal::Number(42.0, span));
    assert!(!number.is_assignable());
}

#[test]
fn test_stmt_has_control_flow() {
    let span = Span::new(0, 5, 1, 0);

    // Return statement has control flow
    let return_stmt = Stmt::Return {
        value: None,
        span: span.clone(),
    };
    assert!(return_stmt.has_control_flow());

    // Break statement has control flow
    let break_stmt = Stmt::Break {
        label: None,
        span: span.clone(),
    };
    assert!(break_stmt.has_control_flow());

    // Continue statement has control flow
    let continue_stmt = Stmt::Continue {
        label: None,
        span: span.clone(),
    };
    assert!(continue_stmt.has_control_flow());

    // Expression statement without control flow
    let expr_stmt = Stmt::Expr(Expr::Literal(Literal::Number(42.0, span.clone())));
    assert!(!expr_stmt.has_control_flow());

    // Block with control flow inside
    let block_with_control = Stmt::Block {
        statements: vec![return_stmt],
        span: span.clone(),
    };
    assert!(block_with_control.has_control_flow());
}
