mod common;

use common::eval_program;
use nnlang::runtime::value::Value;

#[test]
fn test_function_evaluation() {
    let result = eval_program("func = |x| { return x * 2 }").unwrap();
    assert!(matches!(result, Value::Function(_)));

    assert_eq!(
        eval_program("add = |x, y| { x + y }; add(2, 3)").unwrap(),
        Value::Number(5.0)
    );
}

#[test]
fn test_function_calls_comprehensive() {
    // Test builtin function calls through import system
    let result = eval_program("import std:println\nprintln(1)").unwrap();
    // "1\n" => 2 bytes
    assert_eq!(result, Value::Number(2.0));

    // Test println with no arguments
    let result = eval_program("import std:println\nprintln()").unwrap();
    // "\n" => 1 byte
    assert_eq!(result, Value::Number(1.0));

    // With explicit output stream
    let result = eval_program("import std:println\nimport std:io\nprintln(1, io:stdout)").unwrap();
    assert_eq!(result, Value::Number(2.0));

    // Test user-defined function calls
    let result = eval_program("add = |x, y| { x + y }\nadd(2, 3)").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Test function call with no arguments
    let result = eval_program("get_five = || { 5 }\nget_five()").unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Test function call with default parameters
    let result = eval_program("greet = |name = \"World\"| { name }\ngreet()").unwrap();
    assert_eq!(result, Value::String("World".to_string()));

    let result = eval_program("greet = |name = \"World\"| { name }\ngreet(\"Alice\")").unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));
}
