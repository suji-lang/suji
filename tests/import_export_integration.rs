use nnlang::runtime::value::DecimalNumber;
mod common;

use common::{assert_import_works, eval_program_with_modules};
use nnlang::runtime::value::Value;

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
            map.get(&nnlang::runtime::value::MapKey::String(
                "answer".to_string()
            )),
            Some(&Value::Number(DecimalNumber::from_i64(42)))
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
fn test_std_module_completeness() {
    let result = eval_program_with_modules("import std\nstd:println")
        .unwrap()
        .unwrap();
    assert!(matches!(result, Value::Function(_)));
}

#[test]
fn test_integration_with_existing_features() {
    assert_import_works(
        r#"
        import std:println
        loop through [1, 2, 3] with x {
            println(x)
        }
    "#,
    );

    assert_import_works(
        r#"
        import std:println
        greet = |name| { println("Hello, ${name}!") }
        greet("World")
    "#,
    );

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
