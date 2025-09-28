use nnlang::lexer::Lexer;
use nnlang::token::Token;

// Single quote string tests
#[test]
fn test_single_quote_simple_string() {
    let input = r#"'hello world'"#;
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
fn test_single_quote_empty_string() {
    let input = r#"''"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![Token::StringStart, Token::StringEnd, Token::Eof];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_single_quote_with_escapes() {
    let input = r#"'hello\nworld\t\'quoted\''"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("hello\nworld\t'quoted'".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_single_quote_string_interpolation_simple() {
    let input = r#"'Hello ${name}!'"#;
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
fn test_single_quote_string_interpolation_expression() {
    let input = r#"'Result: ${a + b * 2}'"#;
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
fn test_single_quote_string_interpolation_nested_braces() {
    let input = r#"'Map: ${{ a: 1, b: 2 }}'"#;
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

#[test]
fn test_single_quote_escaped_dollar() {
    let input = r#"'Price: \$${amount}'"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("Price: $".to_string()),
        Token::InterpStart,
        Token::Identifier("amount".to_string()),
        Token::InterpEnd,
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_mixed_quote_usage() {
    let input = r#"'He said, "Hello there!"' "She replied, 'Hi back!'""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("He said, \"Hello there!\"".to_string()),
        Token::StringEnd,
        Token::StringStart,
        Token::StringText("She replied, 'Hi back!'".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_single_quote_with_double_quote_inside() {
    let input = r#"'Path: "C:\\Users\\Alice"'"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("Path: \"C:\\Users\\Alice\"".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_double_quote_with_single_quote_inside() {
    let input = r#""It's a beautiful day""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("It's a beautiful day".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_single_quote_all_escape_sequences() {
    let input = r#"'\n\t\r\\\'\"\$'"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("\n\t\r\\'\"$".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
