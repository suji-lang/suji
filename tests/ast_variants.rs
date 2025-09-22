use nnlang::ast::*;

#[test]
fn test_loop_bindings() {
    let none = LoopBindings::None;
    let one = LoopBindings::One("x".to_string());
    let two = LoopBindings::Two("k".to_string(), "v".to_string());

    // Test they can be created and compared
    assert_eq!(none, LoopBindings::None);
    assert_ne!(one, none);
    assert_ne!(two, one);
}

#[test]
fn test_import_spec_variants() {
    let module_import = ImportSpec::Module {
        name: "math".to_string(),
    };
    let item_import = ImportSpec::Item {
        module: "math".to_string(),
        name: "sin".to_string(),
    };
    let item_as_import = ImportSpec::ItemAs {
        module: "math".to_string(),
        name: "sin".to_string(),
        alias: "sine".to_string(),
    };

    // Test different import types are not equal
    assert_ne!(module_import, item_import);
    assert_ne!(item_import, item_as_import);
}
