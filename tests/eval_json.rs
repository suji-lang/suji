mod common;

use common::eval_program;
use suji_values::Value;

#[test]
fn test_json_module_integration() {
    let result = eval_program(
        r#"
        import std:json
        data = json:parse("{\"name\": \"Alice\", \"age\": 30}")
        json:generate(data)
    "#,
    );
    assert!(result.is_ok());
    let output = result.unwrap();
    if let Value::String(json_str) = output {
        assert!(json_str.contains("\"name\":\"Alice\""));
        assert!(json_str.contains("\"age\":30"));
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_json_direct_imports() {
    let result = eval_program(
        r#"
        import std:json:parse
        import std:json:generate
        data = parse("{\"value\": 42}")
        generate(data)
    "#,
    );
    assert!(result.is_ok());
    let output = result.unwrap();
    if let Value::String(json_str) = output {
        assert!(json_str.contains("\"value\":42"));
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_json_roundtrip_integration() {
    let result = eval_program(
        r#"
        import std:json
        original = { name: "Bob", hobbies: ["reading", "coding"] }
        json_str = json:generate(original)
        parsed = json:parse(json_str)
        parsed:name
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Bob".to_string()));
}

#[test]
fn test_json_nested_structures() {
    let result = eval_program(
        r#"
        import std:json
        complex = {
            users: [
                { name: "Alice", age: 30 },
                { name: "Bob", age: 25 }
            ],
            metadata: { version: "1.0", count: 2 }
        }
        json_str = json:generate(complex)
        parsed = json:parse(json_str)
        parsed:metadata:version
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("1.0".to_string()));
}

#[test]
fn test_json_with_boolean_values() {
    let result = eval_program(
        r#"
        import std:json
        data = { active: true, disabled: false, count: 0 }
        json_str = json:generate(data)
        parsed = json:parse(json_str)
        parsed:active
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_json_with_nil_values() {
    let result = eval_program(
        r#"
        import std:json
        data = { value: nil, name: "test" }
        json_str = json:generate(data)
        parsed = json:parse(json_str)
        parsed:value
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
}
