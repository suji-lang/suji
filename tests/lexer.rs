use nnlang::lexer::Lexer;
use nnlang::token::Token;

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
        Token::Number(42.0),
        Token::Number(2.5),
        Token::Number(0.0),
        Token::Number(999.999),
        Token::Number(0.5),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

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
        Token::Number(2.0),
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
        Token::Number(1.0),
        Token::Comma,
        Token::Identifier("b".to_string()),
        Token::Colon,
        Token::Number(2.0),
        Token::RightBrace,
        Token::InterpEnd,
        Token::StringEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

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
fn test_regex_simple() {
    let input = r#"/hello.*world/"#;
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::RegexStart,
        Token::RegexContent("hello.*world".to_string()),
        Token::RegexEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_regex_vs_division_after_identifier() {
    let input = "x / 2"; // Should be division
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::Divide,
        Token::Number(2.0),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_regex_vs_division_after_operator() {
    let input = "= /hello/"; // Should be regex
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Assign,
        Token::RegexStart,
        Token::RegexContent("hello".to_string()),
        Token::RegexEnd,
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_regex_vs_division_after_number() {
    let input = "42 / 3"; // Should be division
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Number(42.0),
        Token::Divide,
        Token::Number(3.0),
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
fn test_method_calls() {
    let input = "obj::method string::length";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("obj".to_string()),
        Token::DoubleColon,
        Token::Identifier("method".to_string()),
        Token::Identifier("string".to_string()),
        Token::DoubleColon,
        Token::Identifier("length".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_comments() {
    let input = "# This is a comment\nx = 1 # Another comment";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Comment("# This is a comment".to_string()),
        Token::Newline,
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::Number(1.0),
        Token::Comment("# Another comment".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_language_spec_example() {
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
    assert!(actual.contains(&Token::Number(1.0)));
    assert!(actual.contains(&Token::Pipe));
    assert!(actual.contains(&Token::Return));
    assert!(actual.contains(&Token::Loop));
    assert!(actual.contains(&Token::Match));
    assert!(actual.contains(&Token::Break));
    assert!(actual.contains(&Token::Increment));
    assert!(actual.contains(&Token::StringStart));
    assert!(actual.contains(&Token::InterpStart));
}

#[test]
fn test_range_operator() {
    let input = "0..10 a..b";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Number(0.0),
        Token::Range,
        Token::Number(10.0),
        Token::Identifier("a".to_string()),
        Token::Range,
        Token::Identifier("b".to_string()),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

#[test]
fn test_escaped_dollar_in_string() {
    let input = r#""Price: \$${amount}""#;
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
fn test_error_unterminated_string() {
    let input = r#""unterminated string"#;
    let result = Lexer::lex(input);

    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Unterminated string"));
    }
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

#[test]
fn test_error_unterminated_regex() {
    let input = r#"x = /unterminated regex"#;
    let result = Lexer::lex(input);

    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Unterminated regex"));
    }
}

#[test]
fn test_compound_assignment_operators() {
    let input = "x += 5; y -= 3; z *= 2; w /= 4; v %= 7";
    let tokens = Lexer::lex(input).unwrap();

    let expected = vec![
        Token::Identifier("x".to_string()),
        Token::PlusAssign,
        Token::Number(5.0),
        Token::Semicolon,
        Token::Identifier("y".to_string()),
        Token::MinusAssign,
        Token::Number(3.0),
        Token::Semicolon,
        Token::Identifier("z".to_string()),
        Token::MultiplyAssign,
        Token::Number(2.0),
        Token::Semicolon,
        Token::Identifier("w".to_string()),
        Token::DivideAssign,
        Token::Number(4.0),
        Token::Semicolon,
        Token::Identifier("v".to_string()),
        Token::ModuloAssign,
        Token::Number(7.0),
        Token::Eof,
    ];

    let actual: Vec<Token> = tokens.into_iter().map(|t| t.token).collect();
    assert_eq!(actual, expected);
}

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
        Token::Number(1.0),
        Token::Semicolon,
        Token::Identifier("y".to_string()),
        Token::Assign,
        Token::Number(2.0),
        Token::Semicolon,
        Token::Identifier("z".to_string()),
        Token::Assign,
        Token::Number(3.0),
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
        Token::Number(1.0),
        Token::Semicolon,
        Token::Identifier("y".to_string()),
        Token::Assign,
        Token::Identifier("y".to_string()),
        Token::Plus,
        Token::Number(1.0),
        Token::Semicolon,
        Token::Identifier("z".to_string()),
        Token::MultiplyAssign,
        Token::Number(2.0),
        Token::Semicolon,
        Token::Identifier("w".to_string()),
        Token::Assign,
        Token::Identifier("w".to_string()),
        Token::Multiply,
        Token::Number(2.0),
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
        Token::Number(2.0),
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
        Token::Number(1.0),
        Token::Comma,
        Token::Identifier("b".to_string()),
        Token::Colon,
        Token::Number(2.0),
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
fn test_single_quote_error_unterminated() {
    let input = r#"'unterminated string"#;
    let result = Lexer::lex(input);

    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Unterminated string"));
    }
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
