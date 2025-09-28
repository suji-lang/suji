use std::fs;
use std::path::PathBuf;

mod common;

use common::eval_program;
use nnlang::runtime::value::{DecimalNumber, Value};

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nnlang_test_{}_{}", name, std::process::id()));
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_io_open_write_and_read_all() {
    let path = temp_path("write_read_all");
    cleanup(&path);

    let source = format!(
        r#"import std:io

f = io:open("{path}")
f::write("hello\nworld")
f::close()

f = io:open("{path}")
f::read_all()
"#,
        path = path.display()
    );

    let result = eval_program(&source).unwrap();
    assert_eq!(result, Value::String("hello\nworld".to_string()));

    cleanup(&path);
}

#[test]
fn test_io_open_read_lines_and_line() {
    let path = temp_path("read_lines");
    cleanup(&path);

    fs::write(&path, b"a\r\nb\n\nlast").unwrap();

    let program_lines = format!(
        r#"import std:io

f = io:open("{path}")
lines = f::read_lines()
lines
"#,
        path = path.display()
    );

    let result = eval_program(&program_lines).unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("".to_string()),
            Value::String("last".to_string()),
        ])
    );

    let program_line = format!(
        r#"import std:io

f = io:open("{path}")
first = f::read_line()
second = f::read_line()
third = f::read_line()
fourth = f::read_line()
result = []
result::push(first)
result::push(second)
result::push(third)
result::push(fourth)
result
"#,
        path = path.display()
    );

    let result = eval_program(&program_line).unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("".to_string()),
            Value::String("last".to_string()),
        ])
    );

    cleanup(&path);
}

#[test]
fn test_io_open_write_returns_bytes() {
    let path = temp_path("write_bytes");
    cleanup(&path);

    let program = format!(
        r#"import std:io

f = io:open("{path}")
f::write("abc")
"#,
        path = path.display()
    );

    let result = eval_program(&program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

    cleanup(&path);
}

#[test]
fn test_io_open_closed_stream_errors() {
    let path = temp_path("closed_error");
    cleanup(&path);

    let program = format!(
        r#"import std:io

f = io:open("{path}")
f::close()
f::read_all()
"#,
        path = path.display()
    );

    let result = eval_program(&program);
    assert!(result.is_err());

    cleanup(&path);
}
