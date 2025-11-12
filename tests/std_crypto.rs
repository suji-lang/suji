mod common;

use common::eval_program;
use suji_values::Value;

#[test]
fn test_crypto_sha256_known_value() {
    let result = eval_program(
        r#"
        import std:crypto
        hash = crypto:sha256("Hello, World!")
        hash
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String(
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f".to_string()
        )
    );
}

#[test]
fn test_crypto_sha256_empty_string() {
    let result = eval_program(
        r#"
        import std:crypto
        hash = crypto:sha256("")
        hash
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()
        )
    );
}

#[test]
fn test_crypto_md5_known_value() {
    let result = eval_program(
        r#"
        import std:crypto
        hash = crypto:md5("Hello, World!")
        hash
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String("65a8e27d8879283831b664bd8b7f0ad4".to_string())
    );
}

#[test]
fn test_crypto_sha1_known_value() {
    let result = eval_program(
        r#"
        import std:crypto
        hash = crypto:sha1("Hello, World!")
        hash
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String("0a0a9f2a6772942557ab5355d76af442f8f65e01".to_string())
    );
}

#[test]
fn test_crypto_sha512_produces_string() {
    let result = eval_program(
        r#"
        import std:crypto
        hash = crypto:sha512("test")
        hash::length() == 128
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_crypto_hmac_sha256_known_value() {
    let result = eval_program(
        r#"
        import std:crypto
        hmac = crypto:hmac_sha256("secret", "test message")
        hmac
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String(
            "3bcebf43c85d20bba6e3b6ba278af1d2ba3ab0d57de271b0ad30b833e851c5a6".to_string()
        )
    );
}

#[test]
fn test_crypto_hmac_sha256_different_keys() {
    let result = eval_program(
        r#"
        import std:crypto
        hmac1 = crypto:hmac_sha256("secret1", "message")
        hmac2 = crypto:hmac_sha256("secret2", "message")
        hmac1 != hmac2
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_crypto_hmac_sha256_different_messages() {
    let result = eval_program(
        r#"
        import std:crypto
        hmac1 = crypto:hmac_sha256("secret", "message1")
        hmac2 = crypto:hmac_sha256("secret", "message2")
        hmac1 != hmac2
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_crypto_hash_consistency() {
    let result = eval_program(
        r#"
        import std:crypto
        text = "consistent data"
        hash1 = crypto:sha256(text)
        hash2 = crypto:sha256(text)
        hash1 == hash2
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_crypto_direct_imports() {
    let result = eval_program(
        r#"
        import std:crypto:sha256
        import std:crypto:md5
        sha_hash = sha256("test")
        md5_hash = md5("test")
        sha_hash::length() > md5_hash::length()
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}
