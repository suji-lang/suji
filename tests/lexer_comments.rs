use suji_lexer::Lexer;
use suji_lexer::token::Token;

#[test]
fn test_comments() {
    let input = "# This is a comment\nx = 1 # Another comment";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Comment("# This is a comment".to_string()),
        Token::Newline,
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::Number("1".to_string()),
        Token::Comment("# Another comment".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
