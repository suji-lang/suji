use nnlang::ast::*;
use nnlang::token::Span;

#[test]
fn test_compound_assign_expression() {
    let span = Span::new(0, 10, 1, 0);
    let target = Expr::Literal(Literal::Identifier("x".to_string(), span.clone()));
    let value = Expr::Literal(Literal::Number(5.0, span.clone()));

    // Test PlusAssign
    let plus_assign = Expr::CompoundAssign {
        target: Box::new(target.clone()),
        op: CompoundOp::PlusAssign,
        value: Box::new(value.clone()),
        span: span.clone(),
    };

    assert_eq!(plus_assign.span(), &span);

    // Test MinusAssign
    let minus_assign = Expr::CompoundAssign {
        target: Box::new(target.clone()),
        op: CompoundOp::MinusAssign,
        value: Box::new(value.clone()),
        span: span.clone(),
    };

    assert_eq!(minus_assign.span(), &span);

    // Test MultiplyAssign
    let multiply_assign = Expr::CompoundAssign {
        target: Box::new(target.clone()),
        op: CompoundOp::MultiplyAssign,
        value: Box::new(value.clone()),
        span: span.clone(),
    };

    assert_eq!(multiply_assign.span(), &span);

    // Test DivideAssign
    let divide_assign = Expr::CompoundAssign {
        target: Box::new(target.clone()),
        op: CompoundOp::DivideAssign,
        value: Box::new(value.clone()),
        span: span.clone(),
    };

    assert_eq!(divide_assign.span(), &span);

    // Test ModuloAssign
    let modulo_assign = Expr::CompoundAssign {
        target: Box::new(target),
        op: CompoundOp::ModuloAssign,
        value: Box::new(value),
        span: span.clone(),
    };

    assert_eq!(modulo_assign.span(), &span);
}

#[test]
fn test_compound_assign_is_assignable() {
    let span = Span::new(0, 10, 1, 0);
    let target = Expr::Literal(Literal::Identifier("x".to_string(), span.clone()));
    let value = Expr::Literal(Literal::Number(5.0, span.clone()));

    let compound_assign = Expr::CompoundAssign {
        target: Box::new(target),
        op: CompoundOp::PlusAssign,
        value: Box::new(value),
        span,
    };

    // Compound assignment expressions themselves are not assignable targets
    // (they are assignments, not targets for assignment)
    assert!(!compound_assign.is_assignable());
}

#[test]
fn test_compound_assign_with_complex_targets() {
    let span = Span::new(0, 15, 1, 0);

    // Test compound assignment with index target
    let list_target = Expr::Literal(Literal::Identifier("list".to_string(), span.clone()));
    let index_target = Expr::Index {
        target: Box::new(list_target),
        index: Box::new(Expr::Literal(Literal::Number(0.0, span.clone()))),
        span: span.clone(),
    };
    let value = Expr::Literal(Literal::Number(10.0, span.clone()));

    let compound_assign = Expr::CompoundAssign {
        target: Box::new(index_target),
        op: CompoundOp::PlusAssign,
        value: Box::new(value),
        span: span.clone(),
    };

    assert_eq!(compound_assign.span(), &span);
}

#[test]
fn test_compound_assign_with_map_targets() {
    let span = Span::new(0, 15, 1, 0);

    // Test compound assignment with map access target
    let map_target = Expr::Literal(Literal::Identifier("map".to_string(), span.clone()));
    let map_access_target = Expr::MapAccessByName {
        target: Box::new(map_target),
        key: "count".to_string(),
        span: span.clone(),
    };
    let value = Expr::Literal(Literal::Number(1.0, span.clone()));

    let compound_assign = Expr::CompoundAssign {
        target: Box::new(map_access_target),
        op: CompoundOp::PlusAssign,
        value: Box::new(value),
        span: span.clone(),
    };

    assert_eq!(compound_assign.span(), &span);
}
