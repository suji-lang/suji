use super::common::eval_program;
use suji_values::Value;

#[test]
fn test_os_rm_file() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("suji_test_rm.txt");
    std::fs::write(&test_file, "content").unwrap();

    let path_str = test_file.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:rm("{}")
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
    assert!(!test_file.exists());
}

#[test]
fn test_os_rm_directory_error() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("suji_test_rm_dir");
    std::fs::create_dir(&test_dir).unwrap();

    let path_str = test_dir.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:rm("{}")
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Cannot remove directory") || error_msg.contains("InvalidOperation"),
        "Should have directory error, got: {}",
        error_msg
    );

    // Cleanup
    std::fs::remove_dir(test_dir).ok();
}

#[test]
fn test_os_rm_nonexistent_error() {
    let code = r#"
        import std:os
        os:rm("/nonexistent/file/path.txt")
    "#;

    let result = eval_program(code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to remove file") || error_msg.contains("InvalidOperation"),
        "Should have file not found error, got: {}",
        error_msg
    );
}

#[test]
fn test_os_mkdir_create_all() {
    let temp_dir = std::env::temp_dir();
    let nested_path = temp_dir.join("suji_test_a").join("b").join("c");

    let path_str = nested_path.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:mkdir("{}", true)
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
    assert!(nested_path.exists());

    // Cleanup
    std::fs::remove_dir_all(temp_dir.join("suji_test_a")).ok();
}

#[test]
fn test_os_mkdir_no_create_all() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("suji_test_mkdir_single");
    // Ensure parent exists
    std::fs::create_dir_all(&temp_dir).ok();

    let path_str = test_dir.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:mkdir("{}", false)
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
    assert!(test_dir.exists());

    // Cleanup
    std::fs::remove_dir(test_dir).ok();
}

#[test]
fn test_os_mkdir_no_create_all_error() {
    let temp_dir = std::env::temp_dir();
    let nested_path = temp_dir.join("suji_test_x").join("y");

    let path_str = nested_path.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:mkdir("{}", false)
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to create directory") || error_msg.contains("InvalidOperation"),
        "Should have parent missing error, got: {}",
        error_msg
    );
}

#[test]
fn test_os_mkdir_default_create_all() {
    let temp_dir = std::env::temp_dir();
    let nested_path = temp_dir.join("suji_test_default").join("nested");

    let path_str = nested_path.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:mkdir("{}")
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
    assert!(nested_path.exists());

    // Cleanup
    std::fs::remove_dir_all(temp_dir.join("suji_test_default")).ok();
}

#[test]
fn test_os_rmdir_empty() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("suji_test_rmdir");
    std::fs::create_dir(&test_dir).unwrap();

    let path_str = test_dir.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:rmdir("{}")
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
    assert!(!test_dir.exists());
}

#[test]
fn test_os_rmdir_not_empty_error() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("suji_test_rmdir_full");
    std::fs::create_dir(&test_dir).unwrap();
    std::fs::write(test_dir.join("file.txt"), "content").unwrap();

    let path_str = test_dir.to_str().unwrap();
    let code = format!(
        r#"
        import std:os
        os:rmdir("{}")
    "#,
        path_str
    );

    let result = eval_program(&code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to remove directory") || error_msg.contains("InvalidOperation"),
        "Should have not empty error, got: {}",
        error_msg
    );

    // Cleanup
    std::fs::remove_dir_all(test_dir).ok();
}

#[test]
fn test_os_rmdir_nonexistent_error() {
    let code = r#"
        import std:os
        os:rmdir("/nonexistent/directory/path")
    "#;

    let result = eval_program(code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to remove directory") || error_msg.contains("InvalidOperation"),
        "Should have directory not found error, got: {}",
        error_msg
    );
}

#[test]
fn test_os_rm_arity_errors() {
    // Too few arguments
    let code1 = r#"
        import std:os
        os:rm()
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
        os:rm("/tmp/file.txt", "extra")
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
fn test_os_mkdir_arity_errors() {
    // Too few arguments
    let code1 = r#"
        import std:os
        os:mkdir()
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
        os:mkdir("/tmp/dir", true, "extra")
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
fn test_os_rmdir_arity_errors() {
    // Too few arguments
    let code1 = r#"
        import std:os
        os:rmdir()
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
        os:rmdir("/tmp/dir", "extra")
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
fn test_os_rm_type_errors() {
    // Path not a string
    let code = r#"
        import std:os
        os:rm(42)
    "#;
    let result = eval_program(code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("TypeError") || error_msg.contains("string"),
        "Should have type error, got: {}",
        error_msg
    );
}

#[test]
fn test_os_mkdir_type_errors() {
    // Path not a string
    let code1 = r#"
        import std:os
        os:mkdir(42)
    "#;
    let result1 = eval_program(code1);
    assert!(result1.is_err());
    let error_msg1 = result1.unwrap_err().to_string();
    assert!(
        error_msg1.contains("TypeError") || error_msg1.contains("string"),
        "Should have type error, got: {}",
        error_msg1
    );

    // create_all not a boolean
    let code2 = r#"
        import std:os
        os:mkdir("/tmp/dir", "not a boolean")
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
fn test_os_rmdir_type_errors() {
    // Path not a string
    let code = r#"
        import std:os
        os:rmdir(42)
    "#;
    let result = eval_program(code);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("TypeError") || error_msg.contains("string"),
        "Should have type error, got: {}",
        error_msg
    );
}
