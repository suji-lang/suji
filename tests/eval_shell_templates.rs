mod common;

use common::{eval_program, eval_string_expr};
use nnlang::runtime::value::Value;

#[test]
fn test_shell_command_templates() {
    let Value::String(s) = eval_string_expr("`echo hello`").unwrap() else {
        panic!("Expected string")
    };
    assert_eq!(s, "hello");

    let Value::String(s) = eval_program("name = \"world\"\n`echo hello ${name}`").unwrap() else {
        panic!("Expected string")
    };
    assert_eq!(s, "hello world");
}
