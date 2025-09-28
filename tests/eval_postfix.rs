mod common;

use common::eval_program;
use nnlang::runtime::value::{DecimalNumber, Value};

#[test]
fn test_postfix_operators() {
    assert_eq!(
        eval_program("x = 5\nx++").unwrap(),
        Value::Number(DecimalNumber::from_i64(6))
    );
    assert_eq!(
        eval_program("y = 10\ny--").unwrap(),
        Value::Number(DecimalNumber::from_i64(9))
    );
    assert_eq!(
        eval_program("z = 7\nz++\nz").unwrap(),
        Value::Number(DecimalNumber::from_i64(8))
    );
}
