mod common;

use common::{eval_program, eval_string_expr};
use nnlang::runtime::value::Value;

#[test]
fn test_list_indexing_and_slicing() {
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][0]").unwrap(),
        Value::Number(10.0)
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][1]").unwrap(),
        Value::Number(20.0)
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][3]").unwrap(),
        Value::Number(40.0)
    );

    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][-1]").unwrap(),
        Value::Number(40.0)
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][-2]").unwrap(),
        Value::Number(30.0)
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][-4]").unwrap(),
        Value::Number(10.0)
    );

    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][1:3]").unwrap(),
        Value::List(vec![Value::Number(20.0), Value::Number(30.0)])
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][:2]").unwrap(),
        Value::List(vec![Value::Number(10.0), Value::Number(20.0)])
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][2:]").unwrap(),
        Value::List(vec![Value::Number(30.0), Value::Number(40.0)])
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][-2:]").unwrap(),
        Value::List(vec![Value::Number(30.0), Value::Number(40.0)])
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][:-2]").unwrap(),
        Value::List(vec![Value::Number(10.0), Value::Number(20.0)])
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][:]").unwrap(),
        Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
            Value::Number(40.0),
        ])
    );
    assert_eq!(
        eval_string_expr("[10, 20, 30, 40][2:2]").unwrap(),
        Value::List(vec![])
    );
}

#[test]
fn test_list_assignment() {
    let result = eval_program("xs = [10, 20, 30]\nxs[1] = 99\nresult = xs[1]").unwrap();
    assert_eq!(result, Value::Number(99.0));

    let result = eval_program("xs = [10, 20, 30]\nxs[-1] = 0\nresult = xs[2]").unwrap();
    assert_eq!(result, Value::Number(0.0));

    let result = eval_program("xs = [1, 2, 3]\nxs[0] = \"hello\"\nresult = xs[0]").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    let result =
        eval_program("xs = [1, 2, 3]\nxs[0] = 10\nxs[1] = 20\nxs[2] = 30\nresult = xs").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0)
        ])
    );
}

#[test]
fn test_map_access_and_assignment() {
    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m:name").unwrap();
    assert_eq!(result, Value::String("Ada".to_string()));
    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m:age").unwrap();
    assert_eq!(result, Value::Number(37.0));
    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m[\"name\"]").unwrap();
    assert_eq!(result, Value::String("Ada".to_string()));
    let result = eval_program("m = { name: \"Ada\", age: 37 }\nresult = m[\"age\"]").unwrap();
    assert_eq!(result, Value::Number(37.0));
    let result =
        eval_program("m = { name: \"Ada\", age: 37 }\nk = \"name\"\nresult = m[k]").unwrap();
    assert_eq!(result, Value::String("Ada".to_string()));
    let result =
        eval_program("m = { 1: \"one\", \"two\": 2, true: \"boolean\" }\nresult = m[1]").unwrap();
    assert_eq!(result, Value::String("one".to_string()));
    let result =
        eval_program("m = { 1: \"one\", \"two\": 2, true: \"boolean\" }\nresult = m[\"two\"]")
            .unwrap();
    assert_eq!(result, Value::Number(2.0));
    let result =
        eval_program("m = { 1: \"one\", \"two\": 2, true: \"boolean\" }\nresult = m[true]")
            .unwrap();
    assert_eq!(result, Value::String("boolean".to_string()));

    let result =
        eval_program("m = { name: \"Ada\" }\nm:name = \"Lovelace\"\nresult = m:name").unwrap();
    assert_eq!(result, Value::String("Lovelace".to_string()));
    let result =
        eval_program("m = { name: \"Ada\" }\nm[\"age\"] = 37\nresult = m[\"age\"]").unwrap();
    assert_eq!(result, Value::Number(37.0));
    let result = eval_program(
        "m = { name: \"Ada\" }\nk = \"country\"\nm[k] = \"UK\"\nresult = m[\"country\"]",
    )
    .unwrap();
    assert_eq!(result, Value::String("UK".to_string()));
}

#[test]
fn test_indexing_errors_and_strings() {
    assert!(eval_string_expr("[1, 2, 3][5]").is_err());
    assert!(eval_string_expr("[1, 2, 3][-5]").is_err());
    assert!(eval_string_expr("[1, 2, 3][1.5]").is_err());
    assert!(eval_string_expr("[1, 2, 3][\"0\"]").is_err());
    assert!(eval_program("m = { name: \"Ada\" }\nresult = m[\"missing\"]").is_err());
    assert!(eval_string_expr("42[0]").is_err());

    let result = eval_string_expr("\"hello\"[0]").unwrap();
    assert_eq!(result, Value::String("h".to_string()));
    let result2 = eval_string_expr("\"hello\"[-1]").unwrap();
    assert_eq!(result2, Value::String("o".to_string()));
    let result3 = eval_string_expr("\"test\"[2]").unwrap();
    assert_eq!(result3, Value::String("s".to_string()));
    assert!(eval_string_expr("\"hello\"[5]").is_err());
    assert!(eval_string_expr("\"hello\"[-6]").is_err());
    assert!(eval_string_expr("\"hello\"[1.5]").is_err());
}

#[test]
fn test_list_concatenation_and_map_iteration_contains() {
    // List concatenation with +
    assert_eq!(
        eval_string_expr("[1, 2] + [3, 4]").unwrap(),
        Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ])
    );

    // Original lists unchanged (pass-by-value semantics)
    let result = eval_program("a = [1,2]\nb = [3,4]\nc = a + b\nresult = a").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
    );

    // Map iteration with key/value
    let result = eval_program(
        "m = { one: 1, two: 2 }\nacc = \"\"\nloop through m with k, v { acc = acc + k + \"=\" + v::to_string() + \" \" }\nresult = acc",
    )
    .unwrap();
    let s = result.to_string();
    assert!(s.contains("one=1") && s.contains("two=2"));

    // Map contains
    let result = eval_program(
        "m = { name: \"Alice\", age: 30 }\nresult = m::contains(\"age\") && !m::contains(\"missing\")",
    )
    .unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_deep_and_mixed_nested_access_and_assignment() {
    // Deep list nesting access and assignment
    let result =
        eval_program("matrix = [[1, 2], [3, 4]]\nmatrix[0][1] = 99\nresult = matrix[0][1]")
            .unwrap();
    assert_eq!(result, Value::Number(99.0));

    // Deep map nesting access and assignment
    let result = eval_program(
        "config = { user: { profile: { settings: { display: { theme: \"dark\", layout: { columns: 3, rows: 2 } } } } } }\nconfig:user:profile:settings:display:layout:columns = 4\nresult = config:user:profile:settings:display:layout:columns",
    )
    .unwrap();
    assert_eq!(result, Value::Number(4.0));

    // Mixed nested structures
    let result = eval_program(
        "data = [{ name: \"item1\" }, { name: \"item2\" }]\ndata[0]:name = \"updated\"\nresult = data[0]:name",
    )
    .unwrap();
    assert_eq!(result, Value::String("updated".to_string()));
}

#[test]
fn test_list_first_last_defaults_and_average() {
    // first/last with defaults
    let result = eval_program("xs = []\nresult = xs::first(42)").unwrap();
    assert_eq!(result, Value::Number(42.0));
    let result = eval_program("ys = []\nresult = ys::last(\"n/a\")").unwrap();
    assert_eq!(result, Value::String("n/a".to_string()));
    let result = eval_program("zs = [1,2,3]\nresult = zs::first(0)").unwrap();
    assert_eq!(result, Value::Number(1.0));

    // average
    let result = eval_program("nums = [1,2,3,4]\nresult = nums::average()").unwrap();
    assert_eq!(result, Value::Number(2.5));
}
