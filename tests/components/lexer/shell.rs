use suji_lexer::Lexer;
use suji_lexer::Token;

#[test]
fn test_shell_command_simple() {
    let input = r#"`echo hello`"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::ShellStart,
        Token::StringText("echo hello".to_string()),
        Token::ShellEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_shell_command_with_interpolation() {
    let input = r#"`echo Hello ${name}`"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::ShellStart,
        Token::StringText("echo Hello ".to_string()),
        Token::InterpStart,
        Token::Identifier("name".to_string()),
        Token::InterpEnd,
        Token::ShellEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_error_unterminated_shell_command() {
    let input = r#"`unterminated command"#;
    let result = Lexer::lex(input);

    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Unterminated shell command"));
    }
}
