use super::common::eval_program_with_modules;

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
fn test_nested_string_with_expression() {
    let input = r#"
import std:println
x = 5
y = 10
msg = "Sum: ${x + y}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_function_call() {
    let input = r#"
import std:println
add = |a, b| a + b
msg = "Result: ${add(3, 4)}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_method_call() {
    let input = r#"
import std:println
list = [1, 2, 3]
msg = "Length: ${list::length()}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_map_access() {
    let input = r#"
import std:println
map = { name: "Alice", age: 30 }
msg = "Name: ${map:name}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_nested_interpolation() {
    let input = r#"
import std:println
x = "world"
msg = "Hello ${"${x}"}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_list_access() {
    let input = r#"
import std:println
list = ["a", "b", "c"]
msg = "First: ${list[0]}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_conditional() {
    let input = r#"
import std:println
x = 5
msg = "Value: ${match x > 0 { true => "positive", false => "negative", }}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_with_match() {
    let input = r#"
import std:println
x = 2
msg = "Result: ${match x { 1 => "one", 2 => "two", _ => "other", }}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[test]
fn test_nested_string_complex_expression() {
    let input = r#"
import std:println
a = 3
b = 4
c = 5
msg = "Sum: ${a + b * c}"
println(msg)
"#;
    let result = eval_program_with_modules(input);
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}
