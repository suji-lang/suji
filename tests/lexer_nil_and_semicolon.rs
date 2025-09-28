use nnlang::lexer::Lexer;
use nnlang::token::Token;

#[test]
fn test_nil_keyword() {
    let input = "x = nil; y = nil";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::Nil,
        Token::Semicolon,
        Token::Identifier("y".to_string()),
        Token::Assign,
        Token::Nil,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_semicolon_token() {
    let input = "x = 1; y = 2; z = 3";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::Number("1".to_string()),
        Token::Semicolon,
        Token::Identifier("y".to_string()),
        Token::Assign,
        Token::Number("2".to_string()),
        Token::Semicolon,
        Token::Identifier("z".to_string()),
        Token::Assign,
        Token::Number("3".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_nil_in_match_pattern() {
    let input = "match x { nil: \"empty\"; _: \"something\" }";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Match,
        Token::Identifier("x".to_string()),
        Token::LeftBrace,
        Token::Nil,
        Token::Colon,
        Token::StringStart,
        Token::StringText("empty".to_string()),
        Token::StringEnd,
        Token::Semicolon,
        Token::Underscore,
        Token::Colon,
        Token::StringStart,
        Token::StringText("something".to_string()),
        Token::StringEnd,
        Token::RightBrace,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
