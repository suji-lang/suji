use nnlang::ast::*;
use nnlang::token::Span;

#[test]
fn test_pattern_is_exhaustive() {
    let span = Span::new(0, 5, 1, 0);

    // Wildcard is exhaustive
    let wildcard = Pattern::Wildcard { span: span.clone() };
    assert!(wildcard.is_exhaustive());

    // Literal is not exhaustive
    let literal = Pattern::Literal {
        value: ValueLike::Number("42".to_string()),
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
    assert!(wildcard.can_match_value(&ValueLike::Number("42".to_string())));
    assert!(wildcard.can_match_value(&ValueLike::Boolean(true)));
    assert!(wildcard.can_match_value(&ValueLike::String("test".to_string())));

    // Literal number pattern matches number values
    let number_pattern = Pattern::Literal {
        value: ValueLike::Number("42".to_string()),
        span: span.clone(),
    };
    assert!(number_pattern.can_match_value(&ValueLike::Number("100".to_string())));
    assert!(!number_pattern.can_match_value(&ValueLike::Boolean(true)));

    // Tuple pattern matches tuple values with same length
    let tuple_pattern = Pattern::Tuple {
        patterns: vec![
            Pattern::Wildcard { span: span.clone() },
            Pattern::Literal {
                value: ValueLike::Number("0".to_string()),
                span: span.clone(),
            },
        ],
        span: span.clone(),
    };
    let tuple_value = ValueLike::Tuple(vec![
        ValueLike::String("test".to_string()),
        ValueLike::Number("123".to_string()),
    ]);
    assert!(tuple_pattern.can_match_value(&tuple_value));

    // Regex pattern matches string values
    let regex_pattern = Pattern::Regex {
        pattern: r"\d+".to_string(),
        span,
    };
    assert!(regex_pattern.can_match_value(&ValueLike::String("123".to_string())));
    assert!(!regex_pattern.can_match_value(&ValueLike::Number("123".to_string())));
}
