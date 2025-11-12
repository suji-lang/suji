mod common;

use common::{eval_program, eval_string_expr};
use suji_values::Value;

#[test]
fn test_shell_commands() {
    let result = eval_string_expr(r#"`echo "hello world"`"#).unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));

    let result = eval_string_expr("`echo test`").unwrap();
    assert_eq!(result, Value::String("test".to_string()));

    let result = eval_string_expr("`echo 42`").unwrap();
    assert_eq!(result, Value::String("42".to_string()));

    let result = eval_string_expr("`echo one two three`").unwrap();
    assert_eq!(result, Value::String("one two three".to_string()));

    let result = eval_string_expr("`echo -n`").unwrap();
    assert_eq!(result, Value::String("".to_string()));
}

#[test]
fn test_shell_interpolation() {
    let result = eval_program("name = \"world\"\nresult = `echo Hello, ${name}`").unwrap();
    assert_eq!(result, Value::String("Hello, world".to_string()));

    let result = eval_program("num = 42\nresult = `echo ${num}`").unwrap();
    assert_eq!(result, Value::String("42".to_string()));

    let result = eval_program("x = 3\ny = 4\nresult = `echo ${x + y}`").unwrap();
    assert_eq!(result, Value::String("7".to_string()));

    let result = eval_program("a = \"foo\"\nb = \"bar\"\nresult = `echo ${a}-${b}`").unwrap();
    assert_eq!(result, Value::String("foo-bar".to_string()));

    let result = eval_program("base = 5\nresult = `echo Result: ${base * base}`").unwrap();
    assert_eq!(result, Value::String("Result: 25".to_string()));

    let result = eval_program("flag = true\nresult = `echo ${flag}`").unwrap();
    assert_eq!(result, Value::String("true".to_string()));
}

#[test]
fn test_shell_pipes_and_operations() {
    let result = eval_string_expr("`echo hello | tr a-z A-Z`").unwrap();
    assert_eq!(result, Value::String("HELLO".to_string()));

    let result =
        eval_string_expr("`echo 'one two three' | tr ' ' '\n' | wc -l | tr -d ' '`").unwrap();
    assert_eq!(result, Value::String("3".to_string()));

    let result = eval_string_expr("`printf '%s' 'test'`").unwrap();
    assert_eq!(result, Value::String("test".to_string()));

    let result = eval_program("cmd = \"date\"\nresult = `echo Current command: ${cmd}`").unwrap();
    assert!(result.to_string().contains("Current command: date"));
}

#[test]
fn test_shell_command_errors() {
    assert!(eval_string_expr("`false`").is_err());
    assert!(eval_string_expr("`nonexistentcommand123`").is_err());
    assert!(eval_string_expr("`echo 'unclosed quote`").is_err());
}

#[test]
fn test_shell_commands_in_context() {
    let result = eval_program("output = `echo assigned`\nresult = output").unwrap();
    assert_eq!(result, Value::String("assigned".to_string()));

    let result = eval_program("getdate = || { return `date +%Y` }\nresult = getdate()").unwrap();
    assert!(result.to_string().chars().all(|c| c.is_ascii_digit()));

    let result = eval_program("prefix = \"Output: \"\nresult = prefix + `echo test`").unwrap();
    assert_eq!(result, Value::String("Output: test".to_string()));

    let result = eval_program("output = `echo hello`\nresult = (output == \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(true));
}
