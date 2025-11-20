use suji_lexer::Lexer;
use suji_lexer::Token;

#[test]
fn test_basic_tokens() {
    let input = "( ) [ ] { } , : :: | ;";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBracket,
        Token::RightBracket,
        Token::LeftBrace,
        Token::RightBrace,
        Token::Comma,
        Token::Colon,
        Token::DoubleColon,
        Token::Pipe,
        Token::Semicolon,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_keywords() {
    let input = "return loop as through with continue break match import export true false nil";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Return,
        Token::Loop,
        Token::As,
        Token::Through,
        Token::With,
        Token::Continue,
        Token::Break,
        Token::Match,
        Token::Import,
        Token::Export,
        Token::True,
        Token::False,
        Token::Nil,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_identifiers() {
    let input = "foo bar_baz _private x1 hello123";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("foo".to_string()),
        Token::Identifier("bar_baz".to_string()),
        Token::Identifier("_private".to_string()),
        Token::Identifier("x1".to_string()),
        Token::Identifier("hello123".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_numbers() {
    let input = "42 2.5 0 999.999 0.5";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Number("42".to_string()),
        Token::Number("2.5".to_string()),
        Token::Number("0".to_string()),
        Token::Number("999.999".to_string()),
        Token::Number("0.5".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

// ============================================================================
// Nil and Semicolon Tests
// ============================================================================

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
    let input = "match x { nil => \"empty\", _ => \"something\", }";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Match,
        Token::Identifier("x".to_string()),
        Token::LeftBrace,
        Token::Nil,
        Token::FatArrow,
        Token::StringStart,
        Token::StringText("empty".to_string()),
        Token::StringEnd,
        Token::Comma,
        Token::Underscore,
        Token::FatArrow,
        Token::StringStart,
        Token::StringText("something".to_string()),
        Token::StringEnd,
        Token::Comma,
        Token::RightBrace,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
