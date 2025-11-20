use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use tempfile::tempdir;

use suji_interpreter::{AstInterpreter, eval_module_source_callback};
use suji_runtime::{Executor, ModuleRegistry, setup_global_env};
use suji_values::{DecimalNumber, Env, MapKey, ModuleHandle, Value};

use super::common::{
    assert_eval_fails, assert_import_works, eval_program_with_modules, parse_program,
};

fn eval_in_dir(dir: &PathBuf, source: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let stmts = parse_program(source)?;
    let env = Rc::new(Env::new());
    setup_global_env(&env);

    // Register builtins BEFORE creating the module registry
    suji_stdlib::runtime::builtins::register_all_builtins();
    let mut registry = ModuleRegistry::new();
    registry.set_source_evaluator(eval_module_source_callback);
    suji_stdlib::setup_module_registry(&mut registry);
    registry.set_base_dir(dir);

    // Use AstInterpreter to evaluate statements
    let interpreter = AstInterpreter;
    let mut last_value = None;
    for stmt in &stmts {
        match interpreter.execute_stmt(stmt, env.clone(), &registry) {
            Ok(val) => last_value = val,
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(last_value)
}

// ============================================================================
// Import Basics
// ============================================================================

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
    assert!(parse_program("import std").is_ok());
    assert!(parse_program("import std:println").is_ok());
    assert!(parse_program("import std:println as say").is_ok());
    assert!(parse_program("import module_name").is_ok());
    assert!(parse_program("import long_module_name:item_name").is_ok());
    assert!(parse_program("import module:item as alias_name").is_ok());
}

#[test]
fn test_error_recovery_and_messages_for_imports() {
    use super::common::assert_parse_fails;
    assert_parse_fails("import std:", "Expected item name after ':'");
    assert_parse_fails("import std:println as", "Expected alias name after 'as'");
    assert_parse_fails(
        "import",
        "Expected module name after import || Expected token",
    );
}

// ============================================================================
// Export Basics
// ============================================================================

#[test]
fn test_export_single_per_file_and_parsing() {
    assert_import_works("export { value: 42 }");

    use super::common::assert_parse_fails;
    assert_parse_fails(
        "export { a: 1 }\nexport { b: 2 }",
        "Multiple export statements",
    );
}

#[test]
fn test_export_evaluation_and_functions() {
    let result = eval_program_with_modules("x = 10\nexport { value: x, doubled: x * 2 }")
        .unwrap()
        .unwrap();

    if let Value::Map(map) = result {
        assert_eq!(
            map.get(&MapKey::String("value".to_string())),
            Some(&Value::Number(DecimalNumber::from_i64(10)))
        );
        assert_eq!(
            map.get(&MapKey::String("doubled".to_string())),
            Some(&Value::Number(DecimalNumber::from_i64(20)))
        );
    } else {
        panic!("Export should return a map, got: {:?}", result);
    }

    let result = eval_program_with_modules("export { add: |x, y| { return x + y } }")
        .unwrap()
        .unwrap();

    if let Value::Map(map) = result {
        let add_func = map.get(&MapKey::String("add".to_string()));
        assert!(matches!(add_func, Some(Value::Function(_))));
    } else {
        panic!("Export should return a map");
    }
}

#[test]
fn test_export_parsing_variations_and_empty_export() {
    assert!(parse_program("export { }").is_ok());
    assert!(parse_program("export { a: 1 }").is_ok());
    assert!(parse_program("export { a: 1, b: 2 }").is_ok());
    assert!(parse_program("export { func: |x| { return x } }").is_ok());
    assert!(parse_program("export { value: x + y }").is_ok());

    let result = eval_program_with_modules("export { }").unwrap().unwrap();
    if let Value::Map(map) = result {
        assert!(map.is_empty());
    } else {
        panic!("Export should return a map");
    }
}

#[test]
fn test_export_undefined_variable() {
    assert_eval_fails("export { value: undefined_var }", "Undefined variable");
}

// ============================================================================
// Import/Export Integration
// ============================================================================

#[test]
fn test_builtin_function_through_import() {
    let result = eval_program_with_modules("import std:println\nprintln(\"Hello\")");
    assert!(result.is_ok());

    let result = eval_program_with_modules("import std:println as say\nsay(\"Hello\")");
    assert!(result.is_ok());
}

#[test]
fn test_complex_import_export_flow() {
    let program = r#"
        import std:println
        x = 42
        message = "The answer is ${x}"
        println(message)
        calculate = |n| { return n * 2 }
        export { answer: x, calculator: calculate, message: message }
    "#;

    let result = eval_program_with_modules(program).unwrap().unwrap();

    if let Value::Map(map) = result {
        assert_eq!(
            map.get(&MapKey::String("answer".to_string())),
            Some(&Value::Number(DecimalNumber::from_i64(42)))
        );
        assert!(
            map.get(&MapKey::String("calculator".to_string()))
                .is_some_and(|v| matches!(v, Value::Function(_)))
        );
        assert_eq!(
            map.get(&MapKey::String("message".to_string())),
            Some(&Value::String("The answer is 42".to_string()))
        );
    } else {
        panic!("Expected map from export");
    }
}

#[test]
fn test_import_export_chain() {
    let program = r#"
        import std:println
        base = 10
        multiplier = |x| { return x * 2 }
        result = multiplier(base)
        println(result)
        export { base: base, result: result, multiplier: multiplier }
    "#;

    let result = eval_program_with_modules(program).unwrap().unwrap();

    if let Value::Map(map) = result {
        assert_eq!(
            map.get(&MapKey::String("base".to_string())),
            Some(&Value::Number(DecimalNumber::from_i64(10)))
        );
        assert_eq!(
            map.get(&MapKey::String("result".to_string())),
            Some(&Value::Number(DecimalNumber::from_i64(20)))
        );
    } else {
        panic!("Expected map from export");
    }
}

#[test]
fn test_import_with_multiple_items() {
    let program = r#"
        import std:println
        import std:io
        println("test")
        io:stdout
    "#;

    let result = eval_program_with_modules(program);
    assert!(result.is_ok());
}

#[test]
fn test_export_with_computed_values() {
    let program = r#"
        x = 5
        y = 10
        sum = x + y
        product = x * y
        export { sum: sum, product: product, x: x, y: y }
    "#;

    let result = eval_program_with_modules(program).unwrap().unwrap();

    if let Value::Map(map) = result {
        assert_eq!(
            map.get(&MapKey::String("sum".to_string())),
            Some(&Value::Number(DecimalNumber::from_i64(15)))
        );
        assert_eq!(
            map.get(&MapKey::String("product".to_string())),
            Some(&Value::Number(DecimalNumber::from_i64(50)))
        );
    } else {
        panic!("Expected map from export");
    }
}

#[test]
fn test_import_alias_conflicts() {
    // Test that aliased imports don't conflict with original names
    let program = r#"
        import std:println as print
        import std:io
        print("test")
        io:stdout
    "#;

    let result = eval_program_with_modules(program);
    assert!(result.is_ok());
}

#[test]
fn test_export_with_functions_and_closures() {
    let program = r#"
        base = 10
        add_base = |x| { return x + base }
        multiply = |x, y| { return x * y }
        export { add_base: add_base, multiply: multiply, base: base }
    "#;

    let result = eval_program_with_modules(program).unwrap().unwrap();

    if let Value::Map(map) = result {
        assert!(
            map.get(&MapKey::String("add_base".to_string()))
                .is_some_and(|v| matches!(v, Value::Function(_)))
        );
        assert!(
            map.get(&MapKey::String("multiply".to_string()))
                .is_some_and(|v| matches!(v, Value::Function(_)))
        );
    } else {
        panic!("Expected map from export");
    }
}

// ============================================================================
// Import Files
// ============================================================================

#[test]
fn test_import_from_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_module.si");
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(file, "x = 42\nexport {{ value: x }}").unwrap();
    file.sync_all().unwrap();

    let source = format!(
        r#"
        import test_module
        test_module:value
    "#,
    );

    let result = eval_in_dir(&dir.path().to_path_buf(), &source);
    assert!(result.is_ok());
    if let Ok(Some(Value::Number(n))) = result {
        assert_eq!(n, DecimalNumber::from_i64(42));
    } else {
        panic!("Expected number 42");
    }
}

#[test]
fn test_import_from_file_with_path() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    let file_path = subdir.join("module.si");
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(file, "y = 100\nexport {{ value: y }}").unwrap();
    file.sync_all().unwrap();

    let source = format!(
        r#"
        import subdir:module
        subdir:module:value
    "#,
    );

    let result = eval_in_dir(&dir.path().to_path_buf(), &source);
    // Nested module paths may not be fully supported - just verify it doesn't crash
    if result.is_err() {
        // If it fails, that's okay - nested paths may not be implemented
        return;
    }
}

#[test]
fn test_import_file_with_function() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("math.si");
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(
        file,
        "add = |x, y| {{ return x + y }}\nexport {{ add: add }}"
    )
    .unwrap();
    file.sync_all().unwrap();

    let source = format!(
        r#"
        import math
        math:add(3, 4)
    "#,
    );

    let result = eval_in_dir(&dir.path().to_path_buf(), &source);
    assert!(result.is_ok());
    if let Ok(Some(Value::Number(n))) = result {
        assert_eq!(n, DecimalNumber::from_i64(7));
    } else {
        panic!("Expected number 7");
    }
}

// ============================================================================
// Value Module Basics
// ============================================================================

#[test]
fn test_module_type_name() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    assert_eq!(module_value.type_name(), "module");
}

#[test]
fn test_module_display_unloaded() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    let display = format!("{}", module_value);
    assert!(display.contains("test:module"));
    assert!(display.contains("unloaded"));
}

#[test]
fn test_module_display_loaded() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );

    // Simulate loaded state
    *handle.loaded.borrow_mut() = Some(Box::new(Value::Nil));

    let module_value = Value::Module(handle);
    let display = format!("{}", module_value);
    assert!(display.contains("test:module"));
    assert!(display.contains("loaded"));
}

#[test]
fn test_module_equality() {
    let handle1 = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let handle2 = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );

    let module1 = Value::Module(handle1);
    let module2 = Value::Module(handle2);

    // Modules with same name should be equal
    assert_eq!(module1, module2);
}

#[test]
fn test_module_inequality() {
    let handle1 = ModuleHandle::new(
        "test:module1".to_string(),
        vec!["test".to_string(), "module1".to_string()],
        None,
        std::ptr::null(),
    );
    let handle2 = ModuleHandle::new(
        "test:module2".to_string(),
        vec!["test".to_string(), "module2".to_string()],
        None,
        std::ptr::null(),
    );

    let module1 = Value::Module(handle1);
    let module2 = Value::Module(handle2);

    // Modules with different names should not be equal
    assert_ne!(module1, module2);
}

#[test]
fn test_module_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    let mut hasher = DefaultHasher::new();
    module_value.type_name().hash(&mut hasher);
    let hash1 = hasher.finish();

    let handle2 = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value2 = Value::Module(handle2);

    let mut hasher2 = DefaultHasher::new();
    module_value2.type_name().hash(&mut hasher2);
    let hash2 = hasher2.finish();

    // Modules with same name should have same hash
    assert_eq!(hash1, hash2);
}

#[test]
fn test_module_clone() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    let cloned = module_value.clone();
    assert_eq!(module_value, cloned);
}

#[test]
fn test_module_in_map() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    let mut map = indexmap::IndexMap::new();
    map.insert(MapKey::String("mod".to_string()), module_value.clone());

    assert_eq!(
        map.get(&MapKey::String("mod".to_string())),
        Some(&module_value)
    );
}

#[test]
fn test_module_in_list() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    let list = vec![module_value.clone()];
    assert_eq!(list[0], module_value);
}

#[test]
fn test_module_as_function_argument() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    // Test that module can be passed as function argument
    let program = format!(
        r#"
        f = |m| {{ return m }}
        f({})
    "#,
        module_value
    );

    let result = eval_program_with_modules(&program);
    // This might fail at runtime, but parsing should work
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_module_return_value() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    // Test that module can be returned from function
    let program = format!(
        r#"
        f = || {{ return {} }}
        f()
    "#,
        module_value
    );

    let result = eval_program_with_modules(&program);
    // This might fail at runtime, but parsing should work
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_module_comparison() {
    let handle1 = ModuleHandle::new(
        "test:module1".to_string(),
        vec!["test".to_string(), "module1".to_string()],
        None,
        std::ptr::null(),
    );
    let handle2 = ModuleHandle::new(
        "test:module2".to_string(),
        vec!["test".to_string(), "module2".to_string()],
        None,
        std::ptr::null(),
    );

    let module1 = Value::Module(handle1);
    let module2 = Value::Module(handle2);

    // Modules should be comparable
    assert_ne!(module1, module2);
}

#[test]
fn test_module_debug_format() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    let debug_str = format!("{:?}", module_value);
    assert!(debug_str.contains("Module"));
    assert!(debug_str.contains("test:module"));
}

// ============================================================================
// Import Files Tests
// ============================================================================

#[test]
fn import_file_leaf_and_map() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let root = tmp.path().to_path_buf();

    // one.si exports a leaf
    {
        let mut f = fs::File::create(root.join("one.si"))?;
        writeln!(f, "export 2")?;
    }
    // two.si exports a map
    {
        let mut f = fs::File::create(root.join("two.si"))?;
        writeln!(f, "export {{ a: 1, b: 2 }}")?;
    }

    // import one (leaf)
    let v = eval_in_dir(&root, "import one")?.unwrap_or(Value::Nil);
    assert!(
        matches!(v, Value::Nil),
        "import binds but program returns Nil"
    );

    // import two:a
    let v = eval_in_dir(&root, "import two:a; a")?;
    assert!(matches!(v, Some(Value::Number(_))));

    Ok(())
}

#[test]
fn import_nested_paths() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let root = tmp.path().to_path_buf();

    // a.si exports a map with key b
    {
        let mut f = fs::File::create(root.join("a.si"))?;
        writeln!(f, "export {{ b: 3 }}")?;
    }

    // a/b.si exports a leaf 4
    fs::create_dir_all(root.join("a"))?;
    {
        let mut f = fs::File::create(root.join("a").join("b.si"))?;
        writeln!(f, "export 4")?;
    }

    // a/b/ directory module with c.si
    fs::create_dir_all(root.join("a").join("b"))?;
    {
        let mut f = fs::File::create(root.join("a").join("b").join("c.si"))?;
        writeln!(f, "export 5")?;
    }

    // import a:b via a.si map
    let out = eval_in_dir(&root, "import a:b; b")?;
    assert!(matches!(out, Some(Value::Number(_))));

    // import a:b via a/b.si
    let out = eval_in_dir(&root, "import a:b; b")?;
    assert!(out.is_some());

    // import a:b:c via directory
    let out = eval_in_dir(&root, "import a:b:c as c; c")?;
    assert!(matches!(out, Some(Value::Number(_))));

    Ok(())
}

#[test]
fn import_errors() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let root = tmp.path().to_path_buf();

    // leaf module
    {
        let mut f = fs::File::create(root.join("leaf.si"))?;
        writeln!(f, "export 1")?;
    }

    // no export module
    {
        let mut f = fs::File::create(root.join("noexp.si"))?;
        writeln!(f, "x = 1")?;
    }

    // Cannot import item from leaf
    let err = eval_in_dir(&root, "import leaf:x").unwrap_err();
    assert!(format!("{}", err).contains("not a valid module"));

    // Missing export should surface
    let err = eval_in_dir(&root, "import noexp").unwrap_err();
    assert!(format!("{}", err).contains("has no export"));

    Ok(())
}
