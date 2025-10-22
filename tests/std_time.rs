mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

#[test]
fn test_time_now_shape() {
    let result = eval_program(
        r#"
        import std:time
        now = time:now()
        has_epoch = now::contains("epoch_ms")
        has_iso = now::contains("iso")
        has_tz = now::contains("tz")
        has_epoch && has_iso && has_tz
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_time_sleep_returns_nil() {
    let result = eval_program(
        r#"
        import std:time
        result = time:sleep(10)
        result
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
}

#[test]
fn test_time_parse_iso() {
    let result = eval_program(
        r#"
        import std:time
        parsed = time:parse_iso("2024-03-15T14:30:00Z")
        parsed::contains("epoch_ms")
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_time_roundtrip() {
    let result = eval_program(
        r#"
        import std:time
        iso_string = "2024-03-15T14:30:00Z"
        parsed = time:parse_iso(iso_string)
        formatted = time:format_iso(parsed:epoch_ms, "Z")
        formatted::contains("2024-03-15") && formatted::contains("14:30:00")
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}
