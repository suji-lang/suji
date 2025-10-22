mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

#[test]
fn test_encoding_base64_roundtrip() {
    let result = eval_program(
        r#"
        import std:encoding
        text = "Hello, World!"
        encoded = encoding:base64_encode(text)
        decoded = encoding:base64_decode(encoded)
        decoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello, World!".to_string()));
}

#[test]
fn test_encoding_base64_known_value() {
    let result = eval_program(
        r#"
        import std:encoding
        encoded = encoding:base64_encode("Hello")
        encoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("SGVsbG8=".to_string()));
}

#[test]
fn test_encoding_hex_roundtrip() {
    let result = eval_program(
        r#"
        import std:encoding
        text = "Hello"
        encoded = encoding:hex_encode(text)
        decoded = encoding:hex_decode(encoded)
        decoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello".to_string()));
}

#[test]
fn test_encoding_hex_known_value() {
    let result = eval_program(
        r#"
        import std:encoding
        encoded = encoding:hex_encode("ABC")
        encoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("414243".to_string()));
}

#[test]
fn test_encoding_hex_case_insensitive() {
    let result = eval_program(
        r#"
        import std:encoding
        decoded = encoding:hex_decode("48656C6C6F")
        decoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Hello".to_string()));
}

#[test]
fn test_encoding_percent_encode() {
    let result = eval_program(
        r#"
        import std:encoding
        encoded = encoding:percent_encode("hello world & stuff")
        encoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String("hello%20world%20%26%20stuff".to_string())
    );
}

#[test]
fn test_encoding_percent_decode() {
    let result = eval_program(
        r#"
        import std:encoding
        decoded = encoding:percent_decode("hello%20world")
        decoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
}

#[test]
fn test_encoding_percent_roundtrip() {
    let result = eval_program(
        r#"
        import std:encoding
        text = "foo bar & baz"
        encoded = encoding:percent_encode(text)
        decoded = encoding:percent_decode(encoded)
        decoded
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("foo bar & baz".to_string()));
}

#[test]
fn test_encoding_direct_imports() {
    let result = eval_program(
        r#"
        import std:encoding:base64_encode
        import std:encoding:base64_decode
        encoded = base64_encode("test")
        base64_decode(encoded)
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("test".to_string()));
}

#[test]
fn test_encoding_base64_decode_invalid() {
    let result = eval_program(
        r#"
        import std:encoding
        encoding:base64_decode("not-valid-base64!!!")
    "#,
    );
    assert!(result.is_err());
}

#[test]
fn test_encoding_hex_decode_invalid() {
    let result = eval_program(
        r#"
        import std:encoding
        encoding:hex_decode("not-hex-zzz")
    "#,
    );
    assert!(result.is_err());
}
