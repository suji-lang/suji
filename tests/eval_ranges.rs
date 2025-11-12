use suji_values::{DecimalNumber, Value};

mod common;
use common::{eval_program, eval_string_expr};

#[test]
fn test_inclusive_range_ascending_basic() {
    let result = eval_string_expr("0..=5").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(0)),
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(3)),
            Value::Number(DecimalNumber::from_i64(4)),
            Value::Number(DecimalNumber::from_i64(5)),
        ])
    );
}

#[test]
fn test_inclusive_range_descending() {
    let result = eval_string_expr("5..=0").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(5)),
            Value::Number(DecimalNumber::from_i64(4)),
            Value::Number(DecimalNumber::from_i64(3)),
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(0)),
        ])
    );
}

#[test]
fn test_inclusive_range_single_element() {
    let result = eval_string_expr("5..=5").unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(DecimalNumber::from_i64(5))])
    );
}

#[test]
fn test_inclusive_range_negative() {
    let result = eval_string_expr("-2..=2").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(-2)),
            Value::Number(DecimalNumber::from_i64(-1)),
            Value::Number(DecimalNumber::from_i64(0)),
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(2)),
        ])
    );
}

#[test]
fn test_inclusive_range_negative_descending() {
    let result = eval_string_expr("2..=-2").unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::Number(DecimalNumber::from_i64(2)),
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Number(DecimalNumber::from_i64(0)),
            Value::Number(DecimalNumber::from_i64(-1)),
            Value::Number(DecimalNumber::from_i64(-2)),
        ])
    );
}

#[test]
fn test_exclusive_vs_inclusive() {
    let exclusive = eval_string_expr("0..5").unwrap();
    let inclusive = eval_string_expr("0..=5").unwrap();

    if let (Value::List(ex), Value::List(inc)) = (exclusive, inclusive) {
        assert_eq!(ex.len(), 5);
        assert_eq!(inc.len(), 6);
    } else {
        panic!("Expected lists");
    }
}

#[test]
fn test_inclusive_range_in_loop() {
    let code = r#"
        sum = 0
        loop through 1..=5 with i {
            sum = sum + i
        }
        sum
    "#;
    let result = eval_program(code).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(15))); // 1+2+3+4+5
}

#[test]
fn test_inclusive_range_assignment() {
    let code = r#"
        nums = 0..=3
        nums
    "#;
    let result = eval_program(code).unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 4);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_inclusive_range_descending_10_to_5() {
    let result = eval_string_expr("10..=5").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 6);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(10)));
        assert_eq!(items[5], Value::Number(DecimalNumber::from_i64(5)));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_inclusive_range_in_list_method() {
    let code = r#"
        nums = 1..=10
        nums::length()
    "#;
    let result = eval_program(code).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(10)));
}

#[test]
fn test_inclusive_range_with_indexing() {
    let code = r#"
        nums = 0..=10
        nums[5]
    "#;
    let result = eval_program(code).unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(5)));
}

#[test]
fn test_range_comparison() {
    let code = r#"
        exclusive = 0..10
        inclusive = 0..=10
        exclusive::length() < inclusive::length()
    "#;
    let result = eval_program(code).unwrap();
    assert_eq!(result, Value::Boolean(true));
}
