use suji_lexer::Lexer;
use suji_lexer::token::Token;

#[test]
fn test_regex_simple() {
    let input = r#"/hello.*world/"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::RegexStart,
        Token::RegexContent("hello.*world".to_string()),
        Token::RegexEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_regex_vs_division_after_identifier() {
    let input = "x / 2"; // Should be division
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Divide,
        Token::Number("2".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_regex_vs_division_after_operator() {
    let input = "= /hello/"; // Should be regex
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Assign,
        Token::RegexStart,
        Token::RegexContent("hello".to_string()),
        Token::RegexEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_regex_vs_division_after_number() {
    let input = "42 / 3"; // Should be division
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Number("42".to_string()),
        Token::Divide,
        Token::Number("3".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_error_unterminated_regex() {
    let input = r#"x = /unterminated regex"#;
    let result = Lexer::lex(input);

    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Unterminated regex"));
    }
}
