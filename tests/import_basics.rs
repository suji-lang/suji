mod common;

use common::{assert_eval_fails, assert_import_works, eval_program_with_modules};
use suji_values::Value;

#[test]
fn test_import_whole_module() {
    assert_import_works("import std");

    let result = eval_program_with_modules("import std\nstd:println")
        .unwrap()
        .unwrap();
    assert!(matches!(result, Value::Function(_)));
}

#[test]
fn test_import_specific_item() {
    assert_import_works("import std:println");
    assert_import_works("import std:println\nprintln(\"test\")");
}

#[test]
fn test_import_with_alias() {
    assert_import_works("import std:println as say");
    assert_import_works("import std:println as say\nsay(\"test\")");
    assert_eval_fails(
        "import std:println as say\nprintln(\"test\")",
        "Undefined variable",
    );
}

#[test]
fn test_import_nonexistent_module_and_item() {
    assert_eval_fails("import nonexistent", "Module 'nonexistent' not found");
    assert_eval_fails(
        "import std:nonexistent",
        "Item 'nonexistent' not found in module 'std'",
    );
    assert_eval_fails("import nonexistent:item", "Module 'nonexistent' not found");
}

#[test]
fn test_import_parsing_variations() {
    use suji_parser::parse_program;
    assert!(parse_program("import std").is_ok());
    assert!(parse_program("import std:println").is_ok());
    assert!(parse_program("import std:println as say").is_ok());
    assert!(parse_program("import module_name").is_ok());
    assert!(parse_program("import long_module_name:item_name").is_ok());
    assert!(parse_program("import module:item as alias_name").is_ok());
}

#[test]
fn test_error_recovery_and_messages_for_imports() {
    use common::assert_parse_fails;
    assert_parse_fails("import std:", "Expected item name after ':'");
    assert_parse_fails("import std:println as", "Expected alias name after 'as'");
    assert_parse_fails(
        "import",
        "Expected module name after import || Expected token",
    );
}
