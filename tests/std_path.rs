mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

fn eval_path(code: &str) -> Value {
    match eval_program(code) {
        Ok(value) => value,
        Err(e) => panic!("Evaluation error: {:?}", e),
    }
}

#[test]
fn test_path_is_abs_unix() {
    let code = r#"
        import std:path
        path:is_abs("/foo/bar")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(true));

    let code = r#"
        import std:path
        path:is_abs("foo/bar")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(false));

    let code = r#"
        import std:path
        path:is_abs("./foo")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(false));

    let code = r#"
        import std:path
        path:is_abs("")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(false));
}

#[test]
#[cfg(target_os = "windows")]
fn test_path_is_abs_windows() {
    let code = r#"
        import std:path
        path:is_abs("C:\\foo\\bar")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(true));

    let code = r#"
        import std:path
        path:is_abs("C:/foo/bar")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(true));

    let code = r#"
        import std:path
        path:is_abs("\\\\server\\share")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(true));

    let code = r#"
        import std:path
        path:is_abs("foo\\bar")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(false));
}

#[test]
fn test_path_join_empty() {
    let code = r#"
        import std:path
        path:join([])
    "#;
    assert_eq!(eval_path(code), Value::String("".to_string()));
}

#[test]
fn test_path_join_single() {
    let code = r#"
        import std:path
        path:join(["foo"])
    "#;
    assert_eq!(eval_path(code), Value::String("foo".to_string()));
}

#[test]
#[cfg(unix)]
fn test_path_join_multiple_unix() {
    let code = r#"
        import std:path
        path:join(["foo", "bar", "baz"])
    "#;
    assert_eq!(eval_path(code), Value::String("foo/bar/baz".to_string()));

    let code = r#"
        import std:path
        path:join(["foo", "/bar", "baz"])
    "#;
    assert_eq!(eval_path(code), Value::String("/bar/baz".to_string()));
}

#[test]
#[cfg(target_os = "windows")]
fn test_path_join_multiple_windows() {
    let code = r#"
        import std:path
        path:join(["foo", "bar", "baz"])
    "#;
    assert_eq!(eval_path(code), Value::String("foo\\bar\\baz".to_string()));

    let code = r#"
        import std:path
        path:join(["foo", "C:\\bar", "baz"])
    "#;
    assert_eq!(eval_path(code), Value::String("C:\\bar\\baz".to_string()));
}

#[test]
fn test_path_dirname_basic() {
    let code = r#"
        import std:path
        path:dirname("/foo/bar")
    "#;
    assert_eq!(eval_path(code), Value::String("/foo".to_string()));

    let code = r#"
        import std:path
        path:dirname("foo/bar")
    "#;
    assert_eq!(eval_path(code), Value::String("foo".to_string()));

    let code = r#"
        import std:path
        path:dirname("foo")
    "#;
    assert_eq!(eval_path(code), Value::String(".".to_string()));

    let code = r#"
        import std:path
        path:dirname("")
    "#;
    assert_eq!(eval_path(code), Value::String(".".to_string()));
}

#[test]
fn test_path_dirname_root() {
    let code = r#"
        import std:path
        path:dirname("/")
    "#;
    assert_eq!(eval_path(code), Value::String("/".to_string()));
}

#[test]
fn test_path_dirname_trailing_slash() {
    let code = r#"
        import std:path
        path:dirname("/foo/bar/")
    "#;
    assert_eq!(eval_path(code), Value::String("/foo".to_string()));
}

#[test]
fn test_path_basename_basic() {
    let code = r#"
        import std:path
        path:basename("/foo/bar")
    "#;
    assert_eq!(eval_path(code), Value::String("bar".to_string()));

    let code = r#"
        import std:path
        path:basename("foo/bar")
    "#;
    assert_eq!(eval_path(code), Value::String("bar".to_string()));

    let code = r#"
        import std:path
        path:basename("foo")
    "#;
    assert_eq!(eval_path(code), Value::String("foo".to_string()));

    let code = r#"
        import std:path
        path:basename("")
    "#;
    assert_eq!(eval_path(code), Value::String("".to_string()));
}

#[test]
fn test_path_basename_trailing_slash() {
    let code = r#"
        import std:path
        path:basename("/foo/bar/")
    "#;
    assert_eq!(eval_path(code), Value::String("bar".to_string()));
}

#[test]
fn test_path_extname_basic() {
    let code = r#"
        import std:path
        path:extname("file.txt")
    "#;
    assert_eq!(eval_path(code), Value::String(".txt".to_string()));

    let code = r#"
        import std:path
        path:extname("archive.tar.gz")
    "#;
    assert_eq!(eval_path(code), Value::String(".gz".to_string()));

    let code = r#"
        import std:path
        path:extname("no_extension")
    "#;
    assert_eq!(eval_path(code), Value::String("".to_string()));

    let code = r#"
        import std:path
        path:extname("")
    "#;
    assert_eq!(eval_path(code), Value::String("".to_string()));
}

#[test]
fn test_path_extname_hidden_files() {
    let code = r#"
        import std:path
        path:extname(".bashrc")
    "#;
    assert_eq!(eval_path(code), Value::String("".to_string()));

    let code = r#"
        import std:path
        path:extname(".config.json")
    "#;
    assert_eq!(eval_path(code), Value::String(".json".to_string()));
}

#[test]
fn test_path_extname_with_path() {
    let code = r#"
        import std:path
        path:extname("/foo/bar/file.txt")
    "#;
    assert_eq!(eval_path(code), Value::String(".txt".to_string()));
}

#[test]
fn test_path_normalize_basic() {
    let code = r#"
        import std:path
        path:normalize("a/./b")
    "#;
    assert_eq!(eval_path(code), Value::String("a/b".to_string()));

    let code = r#"
        import std:path
        path:normalize("a/b/../c")
    "#;
    assert_eq!(eval_path(code), Value::String("a/c".to_string()));

    let code = r#"
        import std:path
        path:normalize("a/b/c/../../d")
    "#;
    assert_eq!(eval_path(code), Value::String("a/d".to_string()));
}

#[test]
fn test_path_normalize_edge_cases() {
    let code = r#"
        import std:path
        path:normalize("")
    "#;
    assert_eq!(eval_path(code), Value::String(".".to_string()));

    let code = r#"
        import std:path
        path:normalize(".")
    "#;
    assert_eq!(eval_path(code), Value::String(".".to_string()));

    let code = r#"
        import std:path
        path:normalize("/")
    "#;
    assert_eq!(eval_path(code), Value::String("/".to_string()));
}

#[test]
fn test_path_normalize_leading_dotdot() {
    let code = r#"
        import std:path
        path:normalize("../a/b")
    "#;
    assert_eq!(eval_path(code), Value::String("../a/b".to_string()));

    let code = r#"
        import std:path
        path:normalize("../../a")
    "#;
    assert_eq!(eval_path(code), Value::String("../../a".to_string()));
}

#[test]
fn test_path_normalize_absolute() {
    let code = r#"
        import std:path
        path:normalize("/a/./b/../c")
    "#;
    assert_eq!(eval_path(code), Value::String("/a/c".to_string()));

    let code = r#"
        import std:path
        path:normalize("/a/b/../../c")
    "#;
    assert_eq!(eval_path(code), Value::String("/c".to_string()));
}

#[test]
fn test_path_normalize_too_many_dotdot_absolute() {
    // Going beyond root should stop at root
    let code = r#"
        import std:path
        path:normalize("/a/../..")
    "#;
    assert_eq!(eval_path(code), Value::String("/".to_string()));
}

#[test]
fn test_path_integration_dirname_basename() {
    let code = r#"
        import std:path
        p = "/foo/bar/baz.txt"
        dir = path:dirname(p)
        base = path:basename(p)
        ext = path:extname(p)
        result = dir + "/" + base
        result
    "#;
    assert_eq!(
        eval_path(code),
        Value::String("/foo/bar/baz.txt".to_string())
    );
}

#[test]
fn test_path_lazy_loading() {
    // Verify that std:path loads std:os automatically
    let code = r#"
        import std:path
        # path module should work without explicit os import
        path:is_abs("/foo")
    "#;
    assert_eq!(eval_path(code), Value::Boolean(true));
}

#[test]
fn test_path_platform_detection() {
    // Verify that path module correctly detects the platform
    let code = r#"
        import std:path
        import std:os

        # The separator should match the OS
        sep = match os:name() {
            "windows" => "\\",
            _ => "/",
        }

        # Join should use the correct separator
        joined = path:join(["a", "b"])
        joined::contains(sep)
    "#;
    assert_eq!(eval_path(code), Value::Boolean(true));
}
