use suji_values::DecimalNumber;

use super::common::eval_program;
use suji_values::Value;

#[test]
fn test_function_implicit_return_and_match_expression() {
    // Implicit return from last expression
    let result = eval_program("mul = |x, y| x * y\nresult = mul(3, 4)").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(12)));

    // Match as an expression yielding a value
    let result =
        eval_program("x = 3\ny = match x { 3 => { 5 + 5 }, 4 => { 2 * 3 }, }\nresult = y").unwrap();
    assert_eq!(result, Value::Number(DecimalNumber::from_i64(10)));
}
