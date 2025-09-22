mod common;

use common::{assert_import_works, eval_program_with_modules};
use nnlang::runtime::value::Value;

#[test]
fn test_export_single_per_file_and_parsing() {
    assert_import_works("export { value: 42 }");

    use common::assert_parse_fails;
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

    let result = eval_program_with_modules("export { add: |x, y| { return x + y } }")
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
fn test_export_parsing_variations_and_empty_export() {
    use nnlang::parser::parse_program;
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
    use common::assert_eval_fails;
    assert_eval_fails("export { value: undefined_var }", "Undefined variable");
}
