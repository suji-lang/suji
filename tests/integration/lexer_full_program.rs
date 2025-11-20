use suji_lexer::Lexer;
use suji_lexer::Token;

#[test]
fn test_full_program_tokenization() {
    let input = r#"
a = 1
b = 2

add = |x, y| {
    return x + y
}

c = add(a, b)

loop {
    match c {
        10: { break }
    }
    c++
}

plus = "${a} plus ${b} does indeed equal ${c}"
"#;

    let tokens = Lexer::lex(input).unwrap();

    // Just verify it tokenizes without error and contains expected tokens
    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();

    // Check for presence of key tokens
    assert!(actual.contains(&Token::Identifier("a".to_string())));
    assert!(actual.contains(&Token::Assign));
    assert!(actual.contains(&Token::Number("1".to_string())));
    assert!(actual.contains(&Token::Pipe));
    assert!(actual.contains(&Token::Return));
    assert!(actual.contains(&Token::Loop));
    assert!(actual.contains(&Token::Match));
    assert!(actual.contains(&Token::Break));
    assert!(actual.contains(&Token::Increment));
    assert!(actual.contains(&Token::StringStart));
    assert!(actual.contains(&Token::InterpStart));
}
