use suji_lang::runtime::value::DecimalNumber;

mod common;
use common::{assert_eval_fails, eval_program};
use suji_lang::runtime::value::Value;

#[test]
fn test_compose_right_runtime() {
    let program = r#"
        add2 = |n| n + 2
        mul3 = |n| n * 3
        f = add2 >> mul3
        f(1)
    "#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(9)));
}

#[test]
fn test_compose_left_runtime() {
    let program = r#"
        add2 = |n| n + 2
        mul3 = |n| n * 3
        f = add2 << mul3
        f(1)
    "#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));
}

#[test]
fn test_composition_chaining() {
    let program = r#"
        add2 = |n| n + 2
        mul3 = |n| n * 3
        g = (add2 >> mul3) >> add2
        g(1)
    "#;
    let result = eval_program(program).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(11)));
}

#[test]
fn test_composition_type_errors() {
    assert_eval_fails("f = 1 >> (|x| x)", "Cannot compose");
    assert_eval_fails("f = (|x| x) << \"s\"", "Cannot compose");
}
