use suji_lang::lexer::Lexer;
use suji_lang::token::Token;

#[test]
fn test_simple_string() {
    let input = r#""hello world""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("hello world".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_empty_string() {
    let input = r#""""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![Token::StringStart, Token::StringEnd, Token::Eof];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_string_with_escapes() {
    let input = "\"hello\\nworld\\t\\\"quoted\\\"\"";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("hello\nworld\t\"quoted\"".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
