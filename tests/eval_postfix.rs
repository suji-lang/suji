mod common;

use common::eval_program;
use nnlang::runtime::value::Value;

#[test]
fn test_postfix_operators() {
    assert_eq!(eval_program("x = 5\nx++").unwrap(), Value::Number(6.0));
    assert_eq!(eval_program("y = 10\ny--").unwrap(), Value::Number(9.0));
    assert_eq!(eval_program("z = 7\nz++\nz").unwrap(), Value::Number(8.0));
}
