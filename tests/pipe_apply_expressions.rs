mod common;
use common::eval_program;
use suji_values::Value;

#[test]
fn test_pipe_apply_with_expression_left_side() {
    // Define an identity unary function and use |> with expression on the left
    let program2 = r#"
        id = |s| s
        '{"k":2}' |> id
    "#;
    if let Value::String(s) = eval_program(program2).unwrap() {
        assert_eq!(s, "{\"k\":2}");
    } else {
        panic!("Expected string value");
    }
}

#[test]
fn test_pipe_apply_with_expression_right_side() {
    let program = r#"
        id = |s| s
        id <| '{"k":3}'
    "#;
    if let Value::String(s) = eval_program(program).unwrap() {
        assert_eq!(s, "{\"k\":3}");
    } else {
        panic!("Expected string value");
    }
}
