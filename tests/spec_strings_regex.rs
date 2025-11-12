mod common;

use common::{eval_program, eval_string_expr};
use suji_values::{DecimalNumber, Value};

#[test]
fn test_string_interpolation() {
    let result = eval_program("name = \"Ada\"\nresult = \"Hello, ${name}\"").unwrap();
    assert_eq!(result, Value::String("Hello, Ada".to_string()));

    let result = eval_program("a = 3\nb = 4\nresult = \"${a} + ${b} = ${a + b}\"").unwrap();
    assert_eq!(result, Value::String("3 + 4 = 7".to_string()));

    let result = eval_program("x = 5\nresult = \"Square of ${x} is ${x ^ 2}\"").unwrap();
    assert_eq!(result, Value::String("Square of 5 is 25".to_string()));

    let result = eval_program(
        "first = \"Ada\"\nlast = \"Lovelace\"\nresult = \"${first} ${last} was a mathematician\"",
    )
    .unwrap();
    assert_eq!(
        result,
        Value::String("Ada Lovelace was a mathematician".to_string())
    );

    let result =
        eval_program("x = 10\ny = 20\nresult = \"Sum is ${x + y}, product is ${x * y}\"").unwrap();
    assert_eq!(
        result,
        Value::String("Sum is 30, product is 200".to_string())
    );

    let result =
        eval_program("name = \"world\"\ngreeting = \"Hello, ${name}\" + \"!\"\nresult = greeting")
            .unwrap();
    assert_eq!(result, Value::String("Hello, world!".to_string()));

    let result = eval_program("flag = true\nresult = \"The flag is ${flag}\"").unwrap();
    assert_eq!(result, Value::String("The flag is true".to_string()));

    let result = eval_program("items = [1, 2, 3]\nresult = \"First item: ${items[0]}\"").unwrap();
    assert_eq!(result, Value::String("First item: 1".to_string()));

    let result =
        eval_program("obj = { name: \"test\", value: 42 }\nresult = \"Object: ${obj}\"").unwrap();
    assert!(result.to_string().contains("Object:"));

    let result = eval_program("result = \"Result: ${(5 + 3) * 2}\"").unwrap();
    assert_eq!(result, Value::String("Result: 16".to_string()));
}

#[test]
fn test_regex_operations_and_precedence() {
    assert_eq!(
        eval_string_expr(r#""hello" ~ /hello/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""world" ~ /hello/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""hello" !~ /world/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""hello" !~ /hello/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""Hello" ~ /hello/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""Hello" ~ /Hello/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abc123" ~ /[0-9]+/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abcdef" ~ /[0-9]+/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""hello world" ~ /^hello/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""say hello" ~ /^hello/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""hello world" ~ /world$/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""world hello" ~ /world$/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""cat" ~ /c.t/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""cut" ~ /c.t/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""ct" ~ /c.t/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""hello" ~ /l+/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""helo" ~ /l+/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""heo" ~ /l+/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""color" ~ /colou?r/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""colour" ~ /colou?r/"#).unwrap(),
        Value::Boolean(true)
    );

    let result = eval_program("pattern = \"hello\"\nresult = \"hello world\" ~ /hello/").unwrap();
    assert_eq!(result, Value::Boolean(true));

    assert_eq!(
        eval_string_expr(r#""hello" ~ /hello/ && true"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""hello" ~ /world/ && true"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""abc" ~ /abc/ || "def" ~ /xyz/"#).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr(r#""abc" ~ /xyz/ || "def" ~ /xyz/"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#""hello" == ("hello" ~ /hello/)"#).unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        eval_string_expr(r#"("hello" ~ /hello/) == true"#).unwrap(),
        Value::Boolean(true)
    );
}

#[test]
fn test_string_slicing_and_single_quotes() {
    // String slicing
    assert_eq!(
        eval_string_expr("\"hello\"[1:3]").unwrap(),
        Value::String("el".to_string())
    );
    assert_eq!(
        eval_string_expr("\"hello\"[:2]").unwrap(),
        Value::String("he".to_string())
    );
    assert_eq!(
        eval_string_expr("\"hello\"[2:]").unwrap(),
        Value::String("llo".to_string())
    );
    assert_eq!(
        eval_string_expr("\"hello\"[-2:]").unwrap(),
        Value::String("lo".to_string())
    );

    // Single-quoted strings behave like double-quoted
    assert_eq!(
        eval_string_expr("'hello'::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_program("name = 'Alice'\nresult = 'Hello, ${name}!' ").unwrap(),
        Value::String("Hello, Alice!".to_string())
    );
}

#[test]
fn test_multiline_strings_and_escaped_dollar_brace() {
    // Multiline triple-quoted strings
    let program = r#"
poem = """
Roses are red,
Violets are blue
"""
result = poem::contains("Roses")
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Boolean(true));

    // Multiline with interpolation
    let program2 = r#"
name = "Ada"
profile = """
Name: ${name}
"""
result = profile::contains("Ada")
"#;
    let result2 = eval_program(program2).unwrap();
    assert_eq!(result2, Value::Boolean(true));

    // Escape ${ with \${ to produce literal sequence
    let program3 = "s = \"Price: \\${amount}\"\nresult = s";
    let result3 = eval_program(program3).unwrap();
    assert_eq!(result3, Value::String("Price: ${amount}".to_string()));
}
