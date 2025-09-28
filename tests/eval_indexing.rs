use nnlang::runtime::value::DecimalNumber;
mod common;

use common::{eval_program, eval_string_expr};
use nnlang::runtime::value::Value;

#[test]
fn test_indexing_slicing_comprehensive() {
    // Test basic indexing
    let result = eval_string_expr("[1, 2, 3, 4][0]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(1)));

    let result = eval_string_expr("[1, 2, 3, 4][2]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(3)));

    // Test slicing
    let result = eval_string_expr("[1, 2, 3, 4][1:3]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(2)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(3)));
    } else {
        panic!("Expected list from slicing");
    }

    // Test slicing with different ranges
    let result = eval_string_expr("[1, 2, 3, 4, 5][0:2]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(DecimalNumber::from_i64(1)));
        assert_eq!(items[1], Value::Number(DecimalNumber::from_i64(2)));
    } else {
        panic!("Expected list from slicing");
    }

    // Test on variables
    let result = eval_program("list = [1, 2, 3, 4]\nlist[1]").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(2)));

    let result = eval_program("list = [1, 2, 3, 4]\nlist[1:3]").unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected list from variable slicing");
    }
}
