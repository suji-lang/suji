use nnlang::ast::*;
use nnlang::token::Span;

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
        Literal::Nil(span.clone()),
    ];

    for literal in literals {
        assert_eq!(literal.span(), &span);
    }
}

#[test]
fn test_nil_literal() {
    let span = Span::new(0, 3, 1, 0);
    let nil_literal = Literal::Nil(span.clone());

    // Test span retrieval
    assert_eq!(nil_literal.span(), &span);

    // Test that it can be used in expressions
    let expr = Expr::Literal(nil_literal);
    assert_eq!(expr.span(), &span);
}
