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

#[test]
fn test_pattern_is_exhaustive() {
    let span = Span::new(0, 5, 1, 0);

    // Wildcard is exhaustive
    let wildcard = Pattern::Wildcard { span: span.clone() };
    assert!(wildcard.is_exhaustive());

    // Literal is not exhaustive
    let literal = Pattern::Literal {
        value: ValueLike::Number(42.0),
        span: span.clone(),
    };
    assert!(!literal.is_exhaustive());

    // Tuple with all wildcards is exhaustive
    let tuple_exhaustive = Pattern::Tuple {
        patterns: vec![
            Pattern::Wildcard { span: span.clone() },
            Pattern::Wildcard { span: span.clone() },
        ],
        span: span.clone(),
    };
    assert!(tuple_exhaustive.is_exhaustive());

    // Tuple with mixed patterns is not exhaustive
    let tuple_mixed = Pattern::Tuple {
        patterns: vec![Pattern::Wildcard { span: span.clone() }, literal],
        span,
    };
    assert!(!tuple_mixed.is_exhaustive());
}

#[test]
fn test_pattern_can_match_value() {
    let span = Span::new(0, 5, 1, 0);

    // Wildcard matches everything
    let wildcard = Pattern::Wildcard { span: span.clone() };
    assert!(wildcard.can_match_value(&ValueLike::Number(42.0)));
    assert!(wildcard.can_match_value(&ValueLike::Boolean(true)));
    assert!(wildcard.can_match_value(&ValueLike::String("test".to_string())));

    // Literal number pattern matches number values
    let number_pattern = Pattern::Literal {
        value: ValueLike::Number(42.0),
        span: span.clone(),
    };
    assert!(number_pattern.can_match_value(&ValueLike::Number(100.0)));
    assert!(!number_pattern.can_match_value(&ValueLike::Boolean(true)));

    // Tuple pattern matches tuple values with same length
    let tuple_pattern = Pattern::Tuple {
        patterns: vec![
            Pattern::Wildcard { span: span.clone() },
            Pattern::Literal {
                value: ValueLike::Number(0.0),
                span: span.clone(),
            },
        ],
        span: span.clone(),
    };
    let tuple_value = ValueLike::Tuple(vec![
        ValueLike::String("test".to_string()),
        ValueLike::Number(123.0),
    ]);
    assert!(tuple_pattern.can_match_value(&tuple_value));

    // Regex pattern matches string values
    let regex_pattern = Pattern::Regex {
        pattern: r"\d+".to_string(),
        span,
    };
    assert!(regex_pattern.can_match_value(&ValueLike::String("123".to_string())));
    assert!(!regex_pattern.can_match_value(&ValueLike::Number(123.0)));
}

#[test]
fn test_binary_op_variants() {
    // Test that all binary operators are represented
    let ops = vec![
        BinaryOp::Add,
        BinaryOp::Subtract,
        BinaryOp::Multiply,
        BinaryOp::Divide,
        BinaryOp::Modulo,
        BinaryOp::Power,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::LessEqual,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
        BinaryOp::And,
        BinaryOp::Or,
        BinaryOp::Range,
        BinaryOp::RegexMatch,
        BinaryOp::RegexNotMatch,
    ];

    // Ensure all operators implement required traits
    for op in ops {
        let _ = format!("{:?}", op);
        let _ = op.clone();
        let _ = op == BinaryOp::Add;
    }
}

#[test]
fn test_unary_op_variants() {
    let ops = vec![UnaryOp::Negate, UnaryOp::Not];

    for op in ops {
        let _ = format!("{:?}", op);
        let _ = op.clone();
        let _ = op == UnaryOp::Negate;
    }
}

#[test]
fn test_loop_bindings() {
    let none = LoopBindings::None;
    let one = LoopBindings::One("x".to_string());
    let two = LoopBindings::Two("k".to_string(), "v".to_string());

    // Test they can be created and compared
    assert_eq!(none, LoopBindings::None);
    assert_ne!(one, none);
    assert_ne!(two, one);
}

#[test]
fn test_import_spec_variants() {
    let module_import = ImportSpec::Module {
        name: "math".to_string(),
    };
    let item_import = ImportSpec::Item {
        module: "math".to_string(),
        name: "sin".to_string(),
    };
    let item_as_import = ImportSpec::ItemAs {
        module: "math".to_string(),
        name: "sin".to_string(),
        alias: "sine".to_string(),
    };

    // Test different import types are not equal
    assert_ne!(module_import, item_import);
    assert_ne!(item_import, item_as_import);
}

#[test]
fn test_literal_span() {
    let span = Span::new(0, 5, 1, 0);

    let literals = vec![
        Literal::Number(42.0, span.clone()),
        Literal::Boolean(true, span.clone()),
        Literal::Identifier("x".to_string(), span.clone()),
        Literal::StringTemplate(vec![], span.clone()),
        Literal::List(vec![], span.clone()),
        Literal::Map(vec![], span.clone()),
        Literal::Tuple(vec![], span.clone()),
        Literal::RegexLiteral("pattern".to_string(), span.clone()),
    ];

    for literal in literals {
        assert_eq!(literal.span(), &span);
    }
}
