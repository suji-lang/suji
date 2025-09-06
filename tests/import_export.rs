use nnlang::parser::parse_program;
use nnlang::runtime::builtins::setup_global_env;
use nnlang::runtime::env::Env;
use nnlang::runtime::eval::eval_program_with_modules;
use nnlang::runtime::module::ModuleRegistry;
use nnlang::runtime::value::Value;
use std::rc::Rc;

/// Helper to create a test environment with built-ins
fn create_test_env() -> Rc<Env> {
    let env = Rc::new(Env::new());
    setup_global_env(&env);
    env
}

/// Helper to evaluate a program with module support
fn eval_program_str(input: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let statements = parse_program(input)?;
    let env = create_test_env();
    let module_registry = ModuleRegistry::new();
    Ok(eval_program_with_modules(
        &statements,
        env,
        &module_registry,
    )?)
}

/// Test that import evaluation succeeds
fn assert_import_works(input: &str) {
    match eval_program_str(input) {
        Ok(_) => {}
        Err(e) => panic!("Import test failed: {}", e),
    }
}

/// Test that parsing fails with expected error
fn assert_parse_fails(input: &str, expected_error_fragment: &str) {
    match parse_program(input) {
        Ok(_) => panic!("Expected parsing to fail for: {}", input),
        Err(e) => assert!(
            e.to_string().contains(expected_error_fragment),
            "Error '{}' should contain '{}'",
            e,
            expected_error_fragment
        ),
    }
}

/// Test that evaluation fails with expected error
fn assert_eval_fails(input: &str, expected_error_fragment: &str) {
    match eval_program_str(input) {
        Ok(_) => panic!("Expected evaluation to fail for: {}", input),
        Err(e) => assert!(
            e.to_string().contains(expected_error_fragment),
            "Error '{}' should contain '{}'",
            e,
            expected_error_fragment
        ),
    }
}

#[test]
fn test_import_whole_module() {
    assert_import_works("import std");

    // Test that std module is available
    let result = eval_program_str("import std\nstd:println")
        .unwrap()
        .unwrap();
    assert!(matches!(result, Value::Function(_)));
}

#[test]
fn test_import_specific_item() {
    assert_import_works("import std:println");

    // Test that println is directly available
    assert_import_works("import std:println\nprintln(\"test\")");
}

#[test]
fn test_import_with_alias() {
    assert_import_works("import std:println as say");

    // Test that alias works
    assert_import_works("import std:println as say\nsay(\"test\")");

    // Test that original name is not available
    assert_eval_fails(
        "import std:println as say\nprintln(\"test\")",
        "Undefined variable",
    );
}

#[test]
fn test_export_single_per_file() {
    // Single export should work
    assert_import_works("export { value: 42 }");

    // Multiple exports should fail at parse time
    assert_parse_fails(
        "export { a: 1 }\nexport { b: 2 }",
        "Multiple export statements",
    );
}

#[test]
fn test_export_evaluation() {
    let result = eval_program_str("x = 10\nexport { value: x, doubled: x * 2 }")
        .unwrap()
        .unwrap();

    if let Value::Map(map) = result {
        assert_eq!(
            map.get(&nnlang::runtime::value::MapKey::String("value".to_string())),
            Some(&Value::Number(10.0))
        );
        assert_eq!(
            map.get(&nnlang::runtime::value::MapKey::String(
                "doubled".to_string()
            )),
            Some(&Value::Number(20.0))
        );
    } else {
        panic!("Export should return a map, got: {:?}", result);
    }
}

#[test]
fn test_export_with_functions() {
    let result = eval_program_str("export { add: |x, y| { return x + y } }")
        .unwrap()
        .unwrap();

    if let Value::Map(map) = result {
        let add_func = map.get(&nnlang::runtime::value::MapKey::String("add".to_string()));
        assert!(matches!(add_func, Some(Value::Function(_))));
    } else {
        panic!("Export should return a map");
    }
}

#[test]
fn test_import_nonexistent_module() {
    assert_eval_fails("import nonexistent", "Module 'nonexistent' not found");
}

#[test]
fn test_import_nonexistent_item() {
    assert_eval_fails(
        "import std:nonexistent",
        "Item 'nonexistent' not found in module 'std'",
    );
}

#[test]
fn test_import_from_nonexistent_module() {
    assert_eval_fails("import nonexistent:item", "Module 'nonexistent' not found");
}

#[test]
fn test_export_undefined_variable() {
    assert_eval_fails("export { value: undefined_var }", "Undefined variable");
}

#[test]
fn test_builtin_function_through_import() {
    // Test that imported println works correctly
    let result = eval_program_str("import std:println\nprintln(\"Hello\")");
    assert!(result.is_ok());

    // Test with alias
    let result = eval_program_str("import std:println as say\nsay(\"Hello\")");
    assert!(result.is_ok());
}

#[test]
fn test_complex_import_export_flow() {
    // Test a complex program with both imports and exports
    let program = r#"
        import std:println
        
        x = 42
        message = "The answer is ${x}"
        
        println(message)
        
        calculate = |n| {
            return n * 2
        }
        
        export {
            answer: x,
            calculator: calculate,
            message: message
        }
    "#;

    let result = eval_program_str(program).unwrap().unwrap();

    if let Value::Map(map) = result {
        assert_eq!(
            map.get(&nnlang::runtime::value::MapKey::String(
                "answer".to_string()
            )),
            Some(&Value::Number(42.0))
        );
        assert!(matches!(
            map.get(&nnlang::runtime::value::MapKey::String(
                "calculator".to_string()
            )),
            Some(Value::Function(_))
        ));
        assert_eq!(
            map.get(&nnlang::runtime::value::MapKey::String(
                "message".to_string()
            )),
            Some(&Value::String("The answer is 42".to_string()))
        );
    } else {
        panic!("Export should return a map");
    }
}

#[test]
fn test_import_parsing_variations() {
    // Test various valid import syntax forms
    assert!(parse_program("import std").is_ok());
    assert!(parse_program("import std:println").is_ok());
    assert!(parse_program("import std:println as say").is_ok());
    assert!(parse_program("import module_name").is_ok());
    assert!(parse_program("import long_module_name:item_name").is_ok());
    assert!(parse_program("import module:item as alias_name").is_ok());
}

#[test]
fn test_export_parsing_variations() {
    // Test various valid export syntax forms
    assert!(parse_program("export { }").is_ok());
    assert!(parse_program("export { a: 1 }").is_ok());
    assert!(parse_program("export { a: 1, b: 2 }").is_ok());
    assert!(parse_program("export { func: |x| { return x } }").is_ok());
    assert!(parse_program("export { value: x + y }").is_ok());
}

#[test]
fn test_error_recovery_and_messages() {
    // Test that we get helpful error messages

    // Missing item name after colon
    assert_parse_fails("import std:", "Expected item name after ':'");

    // Missing alias after 'as'
    assert_parse_fails("import std:println as", "Expected alias name after 'as'");

    // Missing module name
    assert_parse_fails("import", "Expected module name after import");

    // Multiple exports
    let multi_export = "export { a: 1 }\nexport { b: 2 }";
    assert_parse_fails(multi_export, "Multiple export statements");
}

#[test]
fn test_empty_export() {
    let result = eval_program_str("export { }").unwrap().unwrap();

    if let Value::Map(map) = result {
        assert!(map.is_empty());
    } else {
        panic!("Export should return a map");
    }
}

#[test]
fn test_std_module_completeness() {
    // Test that std module has expected items
    let result = eval_program_str("import std\nstd:println")
        .unwrap()
        .unwrap();
    assert!(matches!(result, Value::Function(_)));
}

#[test]
fn test_integration_with_existing_features() {
    // Test that import/export works with other language features

    // With loops
    assert_import_works(
        r#"
        import std:println
        loop through [1, 2, 3] with x {
            println(x)
        }
    "#,
    );

    // With functions
    assert_import_works(
        r#"
        import std:println
        greet = |name| {
            println("Hello, ${name}!")
        }
        greet("World")
    "#,
    );

    // With match statements
    assert_import_works(
        r#"
        import std:println
        x = 42
        match x {
            42: { println("Found the answer!") }
            _: { println("Not the answer") }
        }
    "#,
    );
}
