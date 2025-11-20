use super::common::eval_program;
use suji_values::Value;

#[test]
fn test_os_stat_basic() {
    // Create a temp file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("suji_test_stat.txt");
    std::fs::write(&test_file, "test content").unwrap();

    let path_str = test_file.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        stat = os:stat("{}")
        stat:size
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Number(_)));

    // Cleanup
    std::fs::remove_file(test_file).ok();
}

#[test]
fn test_os_stat_directory() {
    let temp_dir = std::env::temp_dir();
    let path_str = temp_dir.to_str().unwrap();

    let code = format!(
        r#"
        import std:os
        stat = os:stat("{}")
        stat:is_directory
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
#[cfg(unix)]
fn test_os_stat_follow_symlinks() {
    use std::os::unix::fs as unix_fs;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("suji_test_target.txt");
    let test_link = temp_dir.join("suji_test_link.txt");

    std::fs::write(&test_file, "content").unwrap();
    unix_fs::symlink(&test_file, &test_link).unwrap();

    let link_str = test_link.to_str().unwrap();

    // Without following
    let code1 = format!(
        r#"
        import std:os
        stat = os:stat("{}", false)
        stat:is_symlink
    "#,
        link_str
    );
    let result1 = eval_program(&code1);
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), Value::Boolean(true));

    // With following
    let code2 = format!(
        r#"
        import std:os
        stat = os:stat("{}", true)
        stat:is_symlink
    "#,
        link_str
    );
    let result2 = eval_program(&code2);
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Value::Boolean(false));

    // Cleanup
    std::fs::remove_file(test_link).ok();
    std::fs::remove_file(test_file).ok();
}

#[test]
fn test_os_stat_nonexistent() {
    let code = r#"
        import std:os
        os:stat("/nonexistent/file/path")
    "#;

    let result = eval_program(code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to stat") || error_msg.contains("InvalidOperation"),
        "Error message should mention stat failure, got: {}",
        error_msg
    );
}

#[test]
fn test_os_stat_arity_errors() {
    // Too few arguments
    let code1 = r#"
        import std:os
        os:stat()
    "#;
    let result1 = eval_program(code1);
    assert!(result1.is_err());
    let error_msg1 = result1.unwrap_err().to_string();
    assert!(
        error_msg1.contains("ArityMismatch") || error_msg1.contains("expects"),
        "Should have arity error, got: {}",
        error_msg1
    );

    // Too many arguments
    let code2 = r#"
        import std:os
        os:stat("/tmp", false, "extra")
    "#;
    let result2 = eval_program(code2);
    assert!(result2.is_err());
    let error_msg2 = result2.unwrap_err().to_string();
    assert!(
        error_msg2.contains("ArityMismatch") || error_msg2.contains("expects"),
        "Should have arity error, got: {}",
        error_msg2
    );
}

#[test]
fn test_os_stat_type_errors() {
    // Path not a string
    let code1 = r#"
        import std:os
        os:stat(42)
    "#;
    let result1 = eval_program(code1);
    assert!(result1.is_err());
    let error_msg1 = result1.unwrap_err().to_string();
    assert!(
        error_msg1.contains("TypeError") || error_msg1.contains("string"),
        "Should have type error, got: {}",
        error_msg1
    );

    // follow_symlinks not a boolean
    let code2 = r#"
        import std:os
        os:stat("/tmp", "not a boolean")
    "#;
    let result2 = eval_program(code2);
    assert!(result2.is_err());
    let error_msg2 = result2.unwrap_err().to_string();
    assert!(
        error_msg2.contains("TypeError") || error_msg2.contains("boolean"),
        "Should have type error, got: {}",
        error_msg2
    );
}

#[test]
fn test_os_stat_returns_map() {
    let temp_dir = std::env::temp_dir();
    let path_str = temp_dir.to_str().unwrap();

    let code = format!(
        r#"
        import std:os
        os:stat("{}")
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Map(_)));
}

#[test]
fn test_os_stat_map_fields() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("suji_test_stat_fields.txt");
    std::fs::write(&test_file, "hello").unwrap();

    let path_str = test_file.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        stat = os:stat("{}")
        has_size = stat::contains("size")
        has_is_dir = stat::contains("is_directory")
        has_is_symlink = stat::contains("is_symlink")
        has_size && has_is_dir && has_is_symlink
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));

    // Cleanup
    std::fs::remove_file(test_file).ok();
}
