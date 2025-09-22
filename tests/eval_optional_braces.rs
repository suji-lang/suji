mod common;

use common::eval_program;
use nnlang::runtime::value::Value;

// Phase 6 Tests - Optional Braces for Single Expressions

#[test]
fn test_optional_braces_function_evaluation() {
    let result = eval_program("multiply = |x, y| x * y\nresult = multiply(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(12.0));

    let result = eval_program("square = |x| x * x\nresult = square(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(25.0));

    let result = eval_program("greet = |name| \"Hello, \" + name\nresult = greet(\"world\")");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello, world".to_string()));
}

#[test]
fn test_optional_braces_match_evaluation() {
    let result = eval_program(
        "x = 3\nmatch x { 1: result = \"one\" 2: result = \"two\" 3: result = \"three\" _: result = \"other\" }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("three".to_string()));

    let result = eval_program(
        "x = 5\nmatch x { 1: result = 10 2: result = 20 3: result = 30 _: result = 0 }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_optional_braces_mixed_evaluation() {
    let result = eval_program(
        "x = 2\nmatch x { 1: result = x * 2 2: { doubled = x * 2; result = doubled + 1 } _: result = 0 }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(5.0));
}

#[test]
fn test_optional_braces_implicit_return() {
    let result = eval_program("add = |x, y| x + y\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(7.0));

    let result =
        eval_program("complex = |x| { doubled = x * 2; doubled + 1 }\nresult = complex(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(11.0));
}

#[test]
fn test_optional_braces_empty_function() {
    let result = eval_program("constant = || 42\nresult = constant()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(42.0));
}

#[test]
fn test_optional_braces_function_with_default_params() {
    let result = eval_program("add = |x = 0, y = 1| x + y\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(7.0));

    let result = eval_program("add = |x = 0, y = 1| x + y\nresult = add()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[test]
fn test_optional_braces_nested_expressions() {
    let result =
        eval_program("x = 2\nmatch x { 1: result = x * 2 2: result = x + 10 _: result = 0 }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(12.0));
}

#[test]
fn test_optional_braces_backward_compatibility() {
    let result = eval_program("add = |x, y| { return x + y }\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(7.0));

    let result = eval_program("match 1 { 1: { result = \"one\" } 2: { result = \"two\" } }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("one".to_string()));
}

#[test]
fn test_optional_braces_boolean_match() {
    let result = eval_program("match true { true: result = 1 false: result = 0 }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));

    let result = eval_program("match false { true: result = 1 false: result = 0 }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_optional_braces_wildcard_match() {
    let result =
        eval_program("match 42 { 1: result = \"one\" 2: result = \"two\" _: result = \"other\" }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("other".to_string()));
}
