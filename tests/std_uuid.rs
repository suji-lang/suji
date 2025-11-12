mod common;

use common::eval_program;
use suji_values::Value;

#[test]
fn test_uuid_v4_format() {
    let result = eval_program(
        r#"
        import std:uuid
        id = uuid:v4()
        uuid:is_valid(id)
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_uuid_v4_is_string() {
    let result = eval_program(
        r#"
        import std:uuid
        id = uuid:v4()
        id::length() == 36
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_uuid_v4_version_bit() {
    let result = eval_program(
        r#"
        import std:uuid
        id = uuid:v4()
        id[14:15] == "4"
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_uuid_v5_deterministic() {
    let result = eval_program(
        r#"
        import std:uuid
        dns_namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
        uuid1 = uuid:v5(dns_namespace, "example.com")
        uuid2 = uuid:v5(dns_namespace, "example.com")
        uuid1 == uuid2
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_uuid_is_valid_with_valid() {
    let result = eval_program(
        r#"
        import std:uuid
        uuid:is_valid("550e8400-e29b-41d4-a716-446655440000")
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_uuid_is_valid_with_invalid() {
    let result = eval_program(
        r#"
        import std:uuid
        uuid:is_valid("not-a-uuid")
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_uuid_is_valid_no_hyphens() {
    let result = eval_program(
        r#"
        import std:uuid
        uuid:is_valid("550e8400e29b41d4a716446655440000")
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}
