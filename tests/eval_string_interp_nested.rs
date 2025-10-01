mod common;

use common::eval_program_with_modules;

#[test]
fn test_nested_string_double_quotes() {
    let input = r#"
import std:println
msg = "Hello ${"world"}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_single_quotes() {
    let input = r#"
import std:println
msg = 'Hello ${"world"}'
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_concat() {
    let input = r#"
import std:println
msg = "Result: ${"Hello" + " " + "World"}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_multiple_nested_strings() {
    let input = r#"
import std:println
msg = "First: ${"one"}, Second: ${'two'}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_empty_nested_string() {
    let input = r#"
import std:println
msg = "Test ${""}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_escape_sequences() {
    let input = r#"
import std:println
msg = "Test ${"with\nnewline"}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}
