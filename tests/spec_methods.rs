use nnlang::runtime::value::DecimalNumber;
mod common;

use common::{eval_program, eval_string_expr};
use nnlang::runtime::value::Value;

#[test]
fn test_string_methods() {
    assert_eq!(
        eval_string_expr("\"hello\"::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_string_expr("\"\"::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(0))
    );
    assert_eq!(
        eval_string_expr("\"hello world\"::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(11))
    );

    assert_eq!(
        eval_string_expr("\"hello world\"::split()").unwrap(),
        Value::List(vec![
            Value::String("hello".to_string()),
            Value::String("world".to_string())
        ])
    );

    assert_eq!(
        eval_string_expr("\"a,b,c\"::split(\",\")").unwrap(),
        Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])
    );

    assert_eq!(
        eval_string_expr("\"hello::world::test\"::split(\"::\")").unwrap(),
        Value::List(vec![
            Value::String("hello".to_string()),
            Value::String("world".to_string()),
            Value::String("test".to_string()),
        ])
    );

    assert_eq!(
        eval_string_expr("\"hello\"::split(\",\")").unwrap(),
        Value::List(vec![Value::String("hello".to_string())])
    );

    let result = eval_string_expr("\"\"::split()").unwrap();
    match result {
        Value::List(list) => {
            assert!(
                list.is_empty() || (list.len() == 1 && list[0] == Value::String("".to_string()))
            );
        }
        _ => panic!("Expected list result"),
    }
}

#[test]
fn test_list_methods() {
    assert_eq!(
        eval_string_expr("[1, 2, 3]::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(3))
    );
    assert_eq!(
        eval_string_expr("[]::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(0))
    );

    let result = eval_program("list = [1, 2]\nlist::push(3)\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3))
        ])
    );

    let result = eval_program("list = [1, 2]\nlist::push(\"hello\")\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::String("hello".to_string())
        ])
    );

    let result = eval_program("list = [1, 2, 3]\nlast = list::pop()\nresult = last").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

    let result = eval_program("list = [1, 2, 3]\nlist::pop()\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2))
        ])
    );

    let result = eval_program("list = [42]\nlast = list::pop()\nresult = last").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(42)));

    assert_eq!(
        eval_string_expr("[\"a\", \"b\", \"c\"]::join()").unwrap(),
        Value::String("a b c".to_string())
    );
    assert_eq!(
        eval_string_expr("[\"a\", \"b\", \"c\"]::join(\",\")").unwrap(),
        Value::String("a,b,c".to_string())
    );
    assert_eq!(
        eval_string_expr("[1, 2, 3]::join(\"-\")").unwrap(),
        Value::String("1-2-3".to_string())
    );
    assert_eq!(
        eval_string_expr("[1, \"hello\", true]::join(\"|\")").unwrap(),
        Value::String("1|hello|true".to_string())
    );
    assert_eq!(
        eval_string_expr("[]::join()").unwrap(),
        Value::String("".to_string())
    );
    assert_eq!(
        eval_string_expr("[\"single\"]::join()").unwrap(),
        Value::String("single".to_string())
    );
}

#[test]
fn test_map_methods_and_requirements() {
    let result = eval_program("m = { a: 1, b: 2, c: 3 }\nm::delete(\"b\")\nresult = m").unwrap();
    assert!(result.to_string().contains("a"));
    assert!(result.to_string().contains("c"));
    assert!(!result.to_string().contains("b"));

    let result = eval_program("m = { a: 1 }\nm::delete(\"missing\")\nresult = m").unwrap();
    assert!(result.to_string().contains("a"));

    let result =
        eval_program("m = { a: 1, b: 2 }\nm::delete(\"a\")\nm::delete(\"b\")\nresult = m").unwrap();
    assert_eq!(result, Value::Map(indexmap::IndexMap::new()));

    let result = eval_program("m = { x: 1, y: 2 }\nk = \"x\"\nm::delete(k)\nresult = m").unwrap();
    assert!(result.to_string().contains("y"));
    assert!(!result.to_string().contains("x"));

    assert!(eval_program("list = []\nresult = list::pop()").is_err());
    assert!(eval_string_expr("\"hello\"::split(42)").is_err());
    assert!(eval_string_expr("[\"a\", \"b\"]::join(42)").is_err());
    assert!(eval_string_expr("null::length()").is_err());
    assert!(eval_string_expr("|| { return 1 }::length()").is_err());

    let result = eval_program("list = [1, 2]\nlist::push(3)\nresult = list").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3))
        ])
    );
    assert_eq!(
        eval_string_expr("[1, 2, 3]::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(3))
    );
    assert_eq!(
        eval_string_expr("\"hello\"::length()").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert!(eval_string_expr("[1, 2]::push(3)").is_err());
    assert!(eval_string_expr("{ a: 1, b: 2 }::delete(\"a\")").is_err());
}

#[test]
fn test_advanced_methods_v0_1_1_and_v0_1_5() {
    // list::filter/map/fold/sum/product
    let result = eval_program(
        "numbers = [1,2,3,4,5]\nev = numbers::filter(|x| { return x % 2 == 0 })\nresult = ev",
    )
    .unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(4))
        ])
    );

    let result = eval_program(
        "numbers = [1,2,3]\nsquares = numbers::map(|x| { return x * x })\nresult = squares",
    )
    .unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(4)),
            Value::Number(DecimalNumber::from_i64(9))
        ])
    );

    let result = eval_program(
        "numbers = [1,2,3,4,5]\nsum = numbers::fold(0, |acc, x| { return acc + x })\nresult = sum",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(15)));

    let result = eval_program("[1,2,3,4,5]::sum()").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(15)));
    let result = eval_program("[1,2,3,4]::product()").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(24)));

    // index_of on list and string
    let result = eval_program(
        "fruits = [\"apple\", \"banana\", \"cherry\"]\nresult = fruits::index_of(\"banana\")",
    )
    .unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));
    let result =
        eval_program("text = \"hello world\"\nresult = text::index_of(\"world\")").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(6)));

    // New string methods
    assert_eq!(
        eval_string_expr("\"hello world\"::contains(\"world\")").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("\"document.pdf\"::ends_with(\".pdf\")").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        eval_string_expr("\"Hello\"::lower()").unwrap(),
        Value::String("hello".to_string())
    );
    assert_eq!(
        eval_string_expr("\"ha\"::repeat(3)").unwrap(),
        Value::String("hahaha".to_string())
    );

    // New list methods
    assert_eq!(
        eval_string_expr("[1,2,3]::contains(2)").unwrap(),
        Value::Boolean(true)
    );
    let result = eval_string_expr("[3,1,2]::sort()").unwrap();
    match result {
        Value::List(xs) => {
            assert_eq!(
                xs,
                vec![
                    Value::Number(DecimalNumber::from_i64(1)),
                    Value::Number(DecimalNumber::from_i64(2)),
                    Value::Number(DecimalNumber::from_i64(3))
                ]
            );
        }
        _ => panic!("expected list"),
    }

    // Map get and merge
    let result = eval_program("m = { name: \"Alice\" }\nresult = m::get(\"age\", 0)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(0)));
    let result =
        eval_program("user = { name: \"Bob\" }\nuser::merge({ city: \"NY\" })\nresult = user:city")
            .unwrap();
    assert_eq!(result, Value::String("NY".to_string()));

    // Number methods
    assert_eq!(
        eval_string_expr("(-5)::abs() ").unwrap(),
        Value::Number(DecimalNumber::from_i64(5))
    );
    assert_eq!(
        eval_string_expr("3.6::floor() ").unwrap(),
        Value::Number(DecimalNumber::from_i64(3))
    );
    assert_eq!(
        eval_string_expr("2::pow(3) ").unwrap(),
        Value::Number(DecimalNumber::from_i64(8))
    );

    // Tuple methods
    assert_eq!(
        eval_string_expr("(1,2,3)::length() ").unwrap(),
        Value::Number(DecimalNumber::from_i64(3))
    );
}
