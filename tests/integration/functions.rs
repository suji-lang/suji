use suji_values::DecimalNumber;

use super::common::eval_program;
use suji_values::Value;

#[test]
fn test_function_evaluation() {
    let result = eval_program("func = |x| { return x * 2 }").unwrap();
    assert!(matches!(result, Value::Function(_)));

    assert_eq!(
        eval_program("add = |x, y| { x + y }; add(2, 3)").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
}

#[test]
fn test_function_calls_comprehensive() {
    // Test builtin function calls through import system
    let result = eval_program("import std:println\nprintln(1)").unwrap();
    // "1\n" => 2 bytes
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    // Test println with no arguments
    let result = eval_program("import std:println\nprintln()").unwrap();
    // "\n" => 1 byte
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    // With explicit output stream
    let result = eval_program("import std:println\nimport std:io\nprintln(1, io:stdout)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    // Test user-defined function calls
    let result = eval_program("add = |x, y| { x + y }\nadd(2, 3)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

    // Test function call with no arguments
    let result = eval_program("get_five = || { 5 }\nget_five()").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));

    // Test function call with default parameters
    let result = eval_program("greet = |name = \"World\"| { name }\ngreet()").unwrap();
    assert_eq!(result, Value::String("World".to_string()));

    let result = eval_program("greet = |name = \"World\"| { name }\ngreet(\"Alice\")").unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));
}

#[test]
fn test_function_multi_return_destructuring() {
    let program = r#"
        make_pair = || {
            return 1, 2
        }

        left, right = make_pair()
        right
    "#;

    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));
}

// ============================================================================
// Composition Tests
// ============================================================================

#[test]
fn test_compose_right_runtime() {
    let program = r#"
        add2 = |n| n + 2
        mul3 = |n| n * 3
        f = add2 >> mul3
        f(1)
    "#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(9)));
}

#[test]
fn test_compose_left_runtime() {
    let program = r#"
        add2 = |n| n + 2
        mul3 = |n| n * 3
        f = add2 << mul3
        f(1)
    "#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));
}

#[test]
fn test_composition_chaining() {
    let program = r#"
        add2 = |n| n + 2
        mul3 = |n| n * 3
        g = (add2 >> mul3) >> add2
        g(1)
    "#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(11)));
}

#[test]
fn test_composition_type_errors() {
    use super::common::assert_eval_fails;
    assert_eval_fails("f = 1 >> (|x| x)", "Cannot compose");
    assert_eval_fails("f = (|x| x) << \"s\"", "Cannot compose");
}

// Phase 6 Tests - Optional Braces for Single Expressions

#[test]
fn test_optional_braces_function_evaluation() {
    let result = eval_program("multiply = |x, y| x * y\nresult = multiply(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(12)));

    let result = eval_program("square = |x| x * x\nresult = square(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(25)));

    let result = eval_program("greet = |name| \"Hello, \" + name\nresult = greet(\"world\")");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello, world".to_string()));
}

#[test]
fn test_optional_braces_match_evaluation() {
    let result = eval_program(
        "x = 3\nmatch x { 1 => result = \"one\", 2 => result = \"two\", 3 => result = \"three\", _ => result = \"other\", }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("three".to_string()));

    let result = eval_program(
        "x = 5\nmatch x { 1 => result = 10, 2 => result = 20, 3 => result = 30, _ => result = 0, }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(0)));
}

#[test]
fn test_optional_braces_mixed_evaluation() {
    let result = eval_program(
        "x = 2\nmatch x { 1 => result = x * 2, 2 => { doubled = x * 2; result = doubled + 1 }, _ => result = 0, }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(5)));
}

#[test]
fn test_optional_braces_implicit_return() {
    let result = eval_program("add = |x, y| x + y\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(7)));

    let result =
        eval_program("complex = |x| { doubled = x * 2; doubled + 1 }\nresult = complex(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(11)));
}

#[test]
fn test_optional_braces_empty_function() {
    let result = eval_program("constant = || 42\nresult = constant()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(42)));
}

#[test]
fn test_optional_braces_function_with_default_params() {
    let result = eval_program("add = |x = 0, y = 1| x + y\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(7)));

    let result = eval_program("add = |x = 0, y = 1| x + y\nresult = add()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(1)));
}

#[test]
fn test_optional_braces_nested_expressions() {
    let result = eval_program(
        "x = 2\nmatch x { 1 => result = x * 2, 2 => result = x + 10, _ => result = 0, }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(12)));
}

#[test]
fn test_optional_braces_backward_compatibility() {
    let result = eval_program("add = |x, y| { return x + y }\nresult = add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(7)));

    let result = eval_program("match 1 { 1 => { result = \"one\" }, 2 => { result = \"two\" }, }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("one".to_string()));
}

#[test]
fn test_optional_braces_boolean_match() {
    let result = eval_program("match true { true => result = 1, false => result = 0, }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_program("match false { true => result = 1, false => result = 0, }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(0)));
}

#[test]
fn test_optional_braces_wildcard_match() {
    let result = eval_program(
        "match 42 { 1 => result = \"one\", 2 => result = \"two\", _ => result = \"other\", }",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("other".to_string()));
}
