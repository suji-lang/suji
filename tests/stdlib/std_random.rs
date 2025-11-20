use super::common::eval_program_with_modules;
use suji_values::{DecimalNumber, Value};

#[test]
fn random_import_and_seed() {
    let src = r#"
import std:random

random:seed(42)
val = random:random()
"#;
    let result = eval_program_with_modules(src).expect("eval ok");
    assert!(result.is_some());
}

#[test]
fn test_random_string() {
    let code = r#"
import std:random
random:seed(42)
s = random:string("abc", 10)
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(10)));
}

#[test]
fn test_random_hex_string() {
    let code = r#"
import std:random
random:seed(42)
s = random:hex_string(8)
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(8)));
}

#[test]
fn test_random_hex_string_default() {
    let code = r#"
import std:random
random:seed(42)
s = random:hex_string()
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(16)));
}

#[test]
fn test_random_alpha_string() {
    let code = r#"
import std:random
random:seed(42)
s = random:alpha_string(10, false)
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(10)));
}

#[test]
fn test_random_alpha_string_defaults() {
    let code = r#"
import std:random
random:seed(42)
s = random:alpha_string()
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(16)));
}

#[test]
fn test_random_numeric_string() {
    let code = r#"
import std:random
random:seed(42)
s = random:numeric_string(6)
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(6)));
}

#[test]
fn test_random_numeric_string_default() {
    let code = r#"
import std:random
random:seed(42)
s = random:numeric_string()
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(16)));
}

#[test]
fn test_random_alphanumeric_string() {
    let code = r#"
import std:random
random:seed(42)
s = random:alphanumeric_string(12, true)
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(12)));
}

#[test]
fn test_random_alphanumeric_string_defaults() {
    let code = r#"
import std:random
random:seed(42)
s = random:alphanumeric_string()
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(16)));
}

#[test]
fn test_random_string_only_allowed_chars() {
    let code = r#"
import std:random
random:seed(42)
s = random:hex_string(20)
allowed = "0123456789abcdef"
chars = s::to_list()
all_valid = true
loop through chars with c {
    match allowed::contains(c) {
        false => { all_valid = false; break },
        _ => {},
    }
}
all_valid
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_random_string_zero_length() {
    let code = r#"
import std:random
random:seed(42)
s = random:string("abc", 0)
s::length()
"#;
    let result = eval_program_with_modules(code).expect("eval ok");
    assert_eq!(result.unwrap(), Value::Number(DecimalNumber::from_i64(0)));
}
