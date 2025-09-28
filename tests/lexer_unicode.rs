use nnlang::lexer::Lexer;
use nnlang::token::Token;

#[test]
fn test_unicode_string_lexing() {
    // Test basic Unicode characters
    let input = r#""Hello 🌍 World""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("Hello 🌍 World".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_unicode_triple_quote_strings() {
    // Test triple quotes with Unicode content
    let input = r#""""
Multi-line string with 🙂 emoji
and café characters
""""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("\nMulti-line string with 🙂 emoji\nand café characters\n".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_unicode_identifiers() {
    // Test that Unicode characters in identifiers produce appropriate errors
    // The lexer currently only supports ASCII identifiers
    let input = "café = 42";
    let result = Lexer::lex(input);

    // Should fail with unexpected character error for 'é'
    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Unexpected character"));
        assert!(error.to_string().contains("é"));
    }
}

#[test]
fn test_unicode_comments() {
    // Test comments with Unicode characters
    let input = "# This is a comment with 🚀 emoji and café text\n42";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Comment("# This is a comment with 🚀 emoji and café text".to_string()),
        Token::Newline,
        Token::Number("42".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_mixed_ascii_unicode_positioning() {
    // Test that Unicode characters in identifiers produce appropriate errors
    // This tests that our Unicode-safe positioning doesn't crash
    let input = "a🙂b = \"test\"";
    let result = Lexer::lex(input);

    // Should fail with unexpected character error for the emoji
    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Unexpected character"));
        assert!(error.to_string().contains("🙂"));
    }
}

#[test]
fn test_unicode_string_interpolation() {
    // Test string interpolation with Unicode
    let input = r#""Hello ${name} 🌍""#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::StringStart,
        Token::StringText("Hello ".to_string()),
        Token::InterpStart,
        Token::Identifier("name".to_string()),
        Token::InterpEnd,
        Token::StringText(" 🌍".to_string()),
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
