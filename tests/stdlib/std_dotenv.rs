use std::fs;
use std::io::Write;
use tempfile::TempDir;

use super::common::eval_program;
use suji_values::Value;

/// Helper to create a temporary .env file with given content
fn create_temp_env_file(content: &str) -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join(".env.test");
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    (dir, file_path.to_string_lossy().to_string())
}

#[test]
fn test_dotenv_basic_load() {
    let (_dir, path) = create_temp_env_file("BASIC_LOAD_KEY=test_value\n");

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["BASIC_LOAD_KEY"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("test_value".to_string()));
}

#[test]
fn test_dotenv_multiple_keys() {
    let content = "MULTI_KEY1=value1\nMULTI_KEY2=value2\nMULTI_KEY3=value3\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["MULTI_KEY1"] + "," + loaded["MULTI_KEY2"] + "," + loaded["MULTI_KEY3"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("value1,value2,value3".to_string()));
}

#[test]
fn test_dotenv_skip_comments() {
    let content =
        "# This is a comment\nCOMMENT_KEY1=value1\n# Another comment\nCOMMENT_KEY2=value2\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["COMMENT_KEY1"] + "," + loaded["COMMENT_KEY2"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("value1,value2".to_string()));
}

#[test]
fn test_dotenv_skip_blank_lines() {
    let content = "BLANK_KEY1=value1\n\n\nBLANK_KEY2=value2\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["BLANK_KEY1"] + "," + loaded["BLANK_KEY2"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("value1,value2".to_string()));
}

#[test]
fn test_dotenv_skip_malformed_lines() {
    let content = "MALFORM_KEY1=value1\nMALFORMED LINE WITHOUT EQUALS\nMALFORM_KEY2=value2\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["MALFORM_KEY1"] + "," + loaded["MALFORM_KEY2"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("value1,value2".to_string()));
}

#[test]
fn test_dotenv_skip_empty_keys() {
    let content = "EMPTYKEY_KEY1=value1\n=value_without_key\nEMPTYKEY_KEY2=value2\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["EMPTYKEY_KEY1"] + "," + loaded["EMPTYKEY_KEY2"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("value1,value2".to_string()));
}

#[test]
fn test_dotenv_trim_whitespace() {
    let content = "  TRIM_KEY1  =  value1  \n  TRIM_KEY2=value2\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["TRIM_KEY1"] + "," + loaded["TRIM_KEY2"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("value1,value2".to_string()));
}

#[test]
fn test_dotenv_override_false_preserves_existing() {
    let (_dir, path) = create_temp_env_file("OVERRIDE_FALSE_KEY=new_value\n");

    let code = format!(
        r#"
        import std:dotenv
        import std:env

        # Set existing value
        env:var["OVERRIDE_FALSE_KEY"] = "original"

        # Load without override (false)
        dotenv:load("{}", false)

        # Should preserve original value
        env:var["OVERRIDE_FALSE_KEY"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("original".to_string()));
}

#[test]
fn test_dotenv_override_true_replaces_existing() {
    let (_dir, path) = create_temp_env_file("OVERRIDE_TRUE_KEY=new_value\n");

    let code = format!(
        r#"
        import std:dotenv
        import std:env

        # Set existing value
        env:var["OVERRIDE_TRUE_KEY"] = "original"

        # Load with override (true)
        dotenv:load("{}", true)

        # Should have new value
        env:var["OVERRIDE_TRUE_KEY"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("new_value".to_string()));
}

#[test]
fn test_dotenv_default_override_is_false() {
    let (_dir, path) = create_temp_env_file("DEFAULT_OVERRIDE_KEY=new_value\n");

    let code = format!(
        r#"
        import std:dotenv
        import std:env

        # Set existing value
        env:var["DEFAULT_OVERRIDE_KEY"] = "original"

        # Load with just path (override defaults to false)
        dotenv:load("{}")

        # Should preserve original value
        env:var["DEFAULT_OVERRIDE_KEY"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("original".to_string()));
}

#[test]
fn test_dotenv_return_value_contains_loaded_keys() {
    let (_dir, path) = create_temp_env_file("RETVAL_KEY1=value1\nRETVAL_KEY2=value2\n");

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        # Return number of keys loaded
        loaded::length()
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert!(matches!(result, Value::Number(_)));
    if let Value::Number(n) = result {
        assert_eq!(n.to_string(), "2");
    }
}

#[test]
fn test_dotenv_return_value_excludes_skipped_keys() {
    let (_dir, path) = create_temp_env_file("KEY1=value1\n");

    let code = format!(
        r#"
        import std:dotenv
        import std:env

        # Pre-set a key
        env:var["KEY1"] = "existing"

        # Load without override - KEY1 should not be in return map
        loaded = dotenv:load("{}", false)

        # Return should be empty map since KEY1 was not loaded
        loaded::length()
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    if let Value::Number(n) = result {
        assert_eq!(n.to_string(), "0");
    }
}

#[test]
fn test_dotenv_empty_file() {
    let (_dir, path) = create_temp_env_file("");

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded::length()
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    if let Value::Number(n) = result {
        assert_eq!(n.to_string(), "0");
    }
}

#[test]
fn test_dotenv_values_with_equals_sign() {
    let content = "EQUALS_KEY1=value=with=equals\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        loaded["EQUALS_KEY1"]
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::String("value=with=equals".to_string()));
}

#[test]
fn test_dotenv_empty_values() {
    let content = "KEY1=\nKEY2=value2\n";
    let (_dir, path) = create_temp_env_file(content);

    let code = format!(
        r#"
        import std:dotenv

        loaded = dotenv:load("{}", false)
        # Empty values should be skipped, so KEY1 should not be in loaded
        # KEY2 should be in loaded
        !loaded::contains("KEY1") && loaded::contains("KEY2")
        "#,
        path
    );

    let result = eval_program(&code).unwrap();
    assert_eq!(result, Value::Boolean(true));
}
