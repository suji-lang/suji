use suji_lexer::Lexer;
use suji_lexer::token::Token;

#[test]
fn test_lex_range_exclusive() {
    let input = "0..10";
    let tokens = Lexer::lex(input).unwrap();
    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();

    assert_eq!(
        actual,
        vec![
            Token::Number("0".to_string()),
            Token::Range,
            Token::Number("10".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_lex_range_inclusive() {
    let input = "0..=10";
    let tokens = Lexer::lex(input).unwrap();
    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();

    assert_eq!(
        actual,
        vec![
            Token::Number("0".to_string()),
            Token::RangeInclusive,
            Token::Number("10".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_lex_multiple_ranges() {
    let input = "a..b x..=y";
    let tokens = Lexer::lex(input).unwrap();
    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();

    assert_eq!(
        actual,
        vec![
            Token::Identifier("a".to_string()),
            Token::Range,
            Token::Identifier("b".to_string()),
            Token::Identifier("x".to_string()),
            Token::RangeInclusive,
            Token::Identifier("y".to_string()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_lex_range_with_negative() {
    let input = "-5..=5";
    let tokens = Lexer::lex(input).unwrap();
    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();

    assert!(matches!(actual[0], Token::Minus));
    assert!(matches!(actual[2], Token::RangeInclusive));
}

#[test]
fn test_lex_range_exclusive_vs_inclusive() {
    let input = "0..5 10..=15";
    let tokens = Lexer::lex(input).unwrap();
    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();

    // First range is exclusive
    assert!(matches!(actual[1], Token::Range));
    // Second range is inclusive
    assert!(matches!(actual[4], Token::RangeInclusive));
}

#[test]
fn test_lex_range_in_list() {
    let input = "[1..5, 10..=15]";
    let tokens = Lexer::lex(input).unwrap();
    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();

    assert!(matches!(actual[2], Token::Range));
    assert!(matches!(actual[6], Token::RangeInclusive));
}
