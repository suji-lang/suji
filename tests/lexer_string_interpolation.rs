use nnlang::lexer::Lexer;
use nnlang::token::Token;

#[test]
fn test_string_interpolation_simple() {
    let input = r#""Hello ${name}!""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("Hello ".to_string()),
        Token::InterpStart,
        Token::Identifier("name".to_string()),
        Token::InterpEnd,
        Token::StringText("!".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_string_interpolation_expression() {
    let input = r#""Result: ${a + b * 2}""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("Result: ".to_string()),
        Token::InterpStart,
        Token::Identifier("a".to_string()),
        Token::Plus,
        Token::Identifier("b".to_string()),
        Token::Multiply,
        Token::Number("2".to_string()),
        Token::InterpEnd,
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_string_interpolation_nested_braces() {
    let input = r#""Map: ${{ a: 1, b: 2 }}""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("Map: ".to_string()),
        Token::InterpStart,
        Token::LeftBrace,
        Token::Identifier("a".to_string()),
        Token::Colon,
        Token::Number("1".to_string()),
        Token::Comma,
        Token::Identifier("b".to_string()),
        Token::Colon,
        Token::Number("2".to_string()),
        Token::RightBrace,
        Token::InterpEnd,
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
