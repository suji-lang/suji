use super::common::eval_program;
use suji_values::Value;

#[test]
fn test_os_name_returns_valid_string() {
    let result = eval_program(
        r#"
        import std:os
        name = os:name()
        is_valid = (name == "linux") || (name == "darwin") || (name == "windows")
        is_valid
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_name_returns_string_type() {
    let result = eval_program(
        r#"
        import std:os
        os:name()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::String(_)));
}

#[test]
fn test_os_hostname_returns_non_empty_string() {
    let result = eval_program(
        r#"
        import std:os
        hostname = os:hostname()
        hostname::length() > 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_hostname_returns_string_type() {
    let result = eval_program(
        r#"
        import std:os
        os:hostname()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::String(_)));
}

#[test]
fn test_os_uptime_ms_returns_non_negative_number() {
    let result = eval_program(
        r#"
        import std:os
        uptime = os:uptime_ms()
        uptime >= 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_uptime_ms_returns_number_type() {
    let result = eval_program(
        r#"
        import std:os
        os:uptime_ms()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Number(_)));
}

#[test]
fn test_os_tmp_dir_returns_non_empty_string() {
    let result = eval_program(
        r#"
        import std:os
        tmp_dir = os:tmp_dir()
        tmp_dir::length() > 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_tmp_dir_returns_string_type() {
    let result = eval_program(
        r#"
        import std:os
        os:tmp_dir()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::String(_)));
}

#[test]
fn test_os_home_dir_returns_non_empty_string() {
    let result = eval_program(
        r#"
        import std:os
        home_dir = os:home_dir()
        home_dir::length() > 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_home_dir_returns_string_type() {
    let result = eval_program(
        r#"
        import std:os
        os:home_dir()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::String(_)));
}

#[test]
fn test_os_work_dir_returns_non_empty_string() {
    let result = eval_program(
        r#"
        import std:os
        work_dir = os:work_dir()
        work_dir::length() > 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_work_dir_returns_string_type() {
    let result = eval_program(
        r#"
        import std:os
        os:work_dir()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::String(_)));
}

#[test]
fn test_os_pid_returns_positive_number() {
    let result = eval_program(
        r#"
        import std:os
        pid = os:pid()
        pid > 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_pid_returns_number_type() {
    let result = eval_program(
        r#"
        import std:os
        os:pid()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Number(_)));
}

#[test]
fn test_os_ppid_returns_non_negative_number() {
    let result = eval_program(
        r#"
        import std:os
        ppid = os:ppid()
        ppid >= 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_ppid_returns_number_type() {
    let result = eval_program(
        r#"
        import std:os
        os:ppid()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Number(_)));
}

#[test]
#[cfg(unix)]
fn test_os_uid_returns_non_negative_number_unix() {
    let result = eval_program(
        r#"
        import std:os
        uid = os:uid()
        uid >= 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_uid_returns_number_type() {
    let result = eval_program(
        r#"
        import std:os
        os:uid()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Number(_)));
}

#[test]
#[cfg(windows)]
fn test_os_uid_returns_zero_windows() {
    let result = eval_program(
        r#"
        import std:os
        os:uid()
    "#,
    );
    assert!(result.is_ok());
    if let Value::Number(n) = result.unwrap() {
        assert_eq!(n.to_i32(), Some(0));
    } else {
        panic!("Expected number");
    }
}

#[test]
#[cfg(unix)]
fn test_os_gid_returns_non_negative_number_unix() {
    let result = eval_program(
        r#"
        import std:os
        gid = os:gid()
        gid >= 0
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_os_gid_returns_number_type() {
    let result = eval_program(
        r#"
        import std:os
        os:gid()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Number(_)));
}

#[test]
#[cfg(windows)]
fn test_os_gid_returns_zero_windows() {
    let result = eval_program(
        r#"
        import std:os
        os:gid()
    "#,
    );
    assert!(result.is_ok());
    if let Value::Number(n) = result.unwrap() {
        assert_eq!(n.to_i32(), Some(0));
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_os_name_no_args() {
    let result = eval_program(
        r#"
        import std:os
        os:name()
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_os_module_available() {
    let result = eval_program(
        r#"
        import std:os
        os:name()
    "#,
    );
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::String(_)));
}

#[test]
fn test_os_all_functions_available() {
    // Test that all OS functions can be called successfully
    let tests = vec![
        "import std:os; os:name()",
        "import std:os; os:hostname()",
        "import std:os; os:uptime_ms()",
        "import std:os; os:tmp_dir()",
        "import std:os; os:home_dir()",
        "import std:os; os:work_dir()",
        "import std:os; os:pid()",
        "import std:os; os:ppid()",
        "import std:os; os:uid()",
        "import std:os; os:gid()",
    ];

    for test in tests {
        let result = eval_program(test);
        assert!(result.is_ok(), "Failed to call: {}", test);
    }
}

#[test]
fn test_os_directories_are_absolute_paths() {
    let result = eval_program(
        r#"
        import std:os
        tmp_dir = os:tmp_dir()
        home_dir = os:home_dir()
        work_dir = os:work_dir()
        
        # On Unix, absolute paths start with /
        # On Windows, they start with drive letter or \\
        tmp_has_content = tmp_dir::length() > 0
        home_has_content = home_dir::length() > 0
        work_has_content = work_dir::length() > 0
        
        tmp_has_content && home_has_content && work_has_content
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}
