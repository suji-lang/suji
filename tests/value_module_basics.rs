use suji_values::{ModuleHandle, Value};

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
fn test_module_equality_same_path() {
    let handle1 = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let handle2 = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "other".to_string()],
        None,
        std::ptr::null(),
    );

    let module1 = Value::Module(handle1);
    let module2 = Value::Module(handle2);

    // Equality is based on module_path
    assert_eq!(module1, module2);
}

#[test]
fn test_module_equality_different_path() {
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

    assert_ne!(module1, module2);
}

#[test]
fn test_module_not_equal_to_other_types() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );
    let module_value = Value::Module(handle);

    assert_ne!(module_value, Value::Nil);
    assert_ne!(module_value, Value::String("test:module".to_string()));
}

#[test]
fn test_module_is_loaded() {
    let handle = ModuleHandle::new(
        "test:module".to_string(),
        vec!["test".to_string(), "module".to_string()],
        None,
        std::ptr::null(),
    );

    assert!(!handle.is_loaded());

    *handle.loaded.borrow_mut() = Some(Box::new(Value::Nil));

    assert!(handle.is_loaded());
}

#[test]
fn test_module_partial_cmp_returns_none() {
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

    // Modules are not orderable
    assert_eq!(module1.partial_cmp(&module2), None);
}
