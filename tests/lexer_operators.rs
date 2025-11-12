use suji_lexer::Lexer;
use suji_lexer::token::Token;

#[test]
fn test_operators() {
    let input = "x = y + z - a * b / c % d ^ e";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::Identifier("y".to_string()),
        Token::Plus,
        Token::Identifier("z".to_string()),
        Token::Minus,
        Token::Identifier("a".to_string()),
        Token::Multiply,
        Token::Identifier("b".to_string()),
        Token::Divide,
        Token::Identifier("c".to_string()),
        Token::Modulo,
        Token::Identifier("d".to_string()),
        Token::Power,
        Token::Identifier("e".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_additional_operators() {
    let input =
        "x++ y-- x == y x != z x < y x <= z x > y x >= z !x x && y x || z x..y x ~ /re/ x !~ /pat/";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Increment,
        Token::Identifier("y".to_string()),
        Token::Decrement,
        Token::Identifier("x".to_string()),
        Token::Equal,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::NotEqual,
        Token::Identifier("z".to_string()),
        Token::Identifier("x".to_string()),
        Token::Less,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::LessEqual,
        Token::Identifier("z".to_string()),
        Token::Identifier("x".to_string()),
        Token::Greater,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::GreaterEqual,
        Token::Identifier("z".to_string()),
        Token::Not,
        Token::Identifier("x".to_string()),
        Token::Identifier("x".to_string()),
        Token::And,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::Or,
        Token::Identifier("z".to_string()),
        Token::Identifier("x".to_string()),
        Token::Range,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::RegexMatch,
        Token::RegexStart,
        Token::RegexContent("re".to_string()),
        Token::RegexEnd,
        Token::Identifier("x".to_string()),
        Token::RegexNotMatch,
        Token::RegexStart,
        Token::RegexContent("pat".to_string()),
        Token::RegexEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_range_operator() {
    let input = "0..10 a..b";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Number("0".to_string()),
        Token::Range,
        Token::Number("10".to_string()),
        Token::Identifier("a".to_string()),
        Token::Range,
        Token::Identifier("b".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_postfix_operators() {
    let input = "x++ y--";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Increment,
        Token::Identifier("y".to_string()),
        Token::Decrement,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_compound_assignment_operators() {
    let input = "x += 5; y -= 3; z *= 2; w /= 4; v %= 7";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::PlusAssign,
        Token::Number("5".to_string()),
        Token::Semicolon,
        Token::Identifier("y".to_string()),
        Token::MinusAssign,
        Token::Number("3".to_string()),
        Token::Semicolon,
        Token::Identifier("z".to_string()),
        Token::MultiplyAssign,
        Token::Number("2".to_string()),
        Token::Semicolon,
        Token::Identifier("w".to_string()),
        Token::DivideAssign,
        Token::Number("4".to_string()),
        Token::Semicolon,
        Token::Identifier("v".to_string()),
        Token::ModuloAssign,
        Token::Number("7".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_compound_assignment_vs_regular_operators() {
    let input = "x += 1; y = y + 1; z *= 2; w = w * 2";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::PlusAssign,
        Token::Number("1".to_string()),
        Token::Semicolon,
        Token::Identifier("y".to_string()),
        Token::Assign,
        Token::Identifier("y".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
        Token::Semicolon,
        Token::Identifier("z".to_string()),
        Token::MultiplyAssign,
        Token::Number("2".to_string()),
        Token::Semicolon,
        Token::Identifier("w".to_string()),
        Token::Assign,
        Token::Identifier("w".to_string()),
        Token::Multiply,
        Token::Number("2".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_fat_arrow() {
    let input = "x => y";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::FatArrow,
        Token::Identifier("y".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_fat_arrow_vs_assign_and_greater() {
    // Ensure => is distinct from = > and = >=
    let input = "x = y x => y x >= y x > y";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::FatArrow,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::GreaterEqual,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::Greater,
        Token::Identifier("y".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_fat_arrow_in_match_context() {
    // Realistic match arm token sequence
    let input = "true => 1,";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::True,
        Token::FatArrow,
        Token::Number("1".to_string()),
        Token::Comma,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_fat_arrow_multiple() {
    // Multiple fat arrows in sequence (like in a match expression)
    let input = "1 => a, 2 => b,";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Number("1".to_string()),
        Token::FatArrow,
        Token::Identifier("a".to_string()),
        Token::Comma,
        Token::Number("2".to_string()),
        Token::FatArrow,
        Token::Identifier("b".to_string()),
        Token::Comma,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_function_composition_tokens() {
    let input = "f >> g h<<i";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("f".to_string()),
        Token::ComposeRight,
        Token::Identifier("g".to_string()),
        Token::Identifier("h".to_string()),
        Token::ComposeLeft,
        Token::Identifier("i".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_composition_does_not_conflict_with_relations_and_pipes() {
    let input = "a >> b >= c a << b <| c x > y x < y";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("a".to_string()),
        Token::ComposeRight,
        Token::Identifier("b".to_string()),
        Token::GreaterEqual,
        Token::Identifier("c".to_string()),
        Token::Identifier("a".to_string()),
        Token::ComposeLeft,
        Token::Identifier("b".to_string()),
        Token::PipeBackward,
        Token::Identifier("c".to_string()),
        Token::Identifier("x".to_string()),
        Token::Greater,
        Token::Identifier("y".to_string()),
        Token::Identifier("x".to_string()),
        Token::Less,
        Token::Identifier("y".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_composition_greedy_matching() {
    let input = ">>> <<<<";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::ComposeRight,
        Token::Greater,
        Token::ComposeLeft,
        Token::ComposeLeft,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}
