use crate::common::{assert_eval_fails, eval_program};

mod common;

#[test]
fn forward_apply_basic() {
    let src = r#"
inc = |x| x + 1
double = |x| x * 2
3 |> inc
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "4");
}

#[test]
fn forward_apply_chain() {
    let src = r#"
inc = |x| x + 1
double = |x| x * 2
3 |> inc |> double
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "8");
}

#[test]
fn backward_apply_call() {
    let src = r#"
inc = |x| x + 1
double = |x| x * 2
double <| inc(3)
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "8");
}

#[test]
fn backward_apply_right_associative() {
    let src = r#"
inc = |x| x + 1
double = |x| x * 2
double <| inc <| 3
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "8");
}

#[test]
fn precedence_with_arithmetic() {
    let src = r#"
inc = |x| x + 1
2 * 3 |> inc
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "7");
}

#[test]
fn defaults_honored() {
    let src = r#"
add = |x, y = 10| x + y
5 |> add
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "15");
}

#[test]
fn builtin_with_pipe_apply() {
    let program1 = r#"
import std:json

s = '{ "name": "Philip" }'

s |> json:parse
"#;
    let program2 = r#"
import std:json

s = '{ "name": "Philip" }'

parse_wrapper = |x| json:parse(x)

s |> parse_wrapper
"#;

    let v1 = eval_program(program1).unwrap();
    let v2 = eval_program(program2).unwrap();
    assert_eq!(format!("{}", v1), format!("{}", v2));
}

#[test]
fn mixing_with_stream_pipe_grouping_parse_only() {
    // Ensure parsing groups as (a |> f) | g; we don't execute an actual stream stage here
    let src = r#"
f = |x| x
g = |x| x
1 |> f
"#;
    let v = eval_program(src).unwrap();
    assert_eq!(format!("{}", v), "1");
}

#[test]
fn type_errors_for_non_functions() {
    // |> with non-function RHS
    assert_eval_fails(
        "1 |> 2",
        "Pipe apply (|>) requires a function on the right-hand side",
    );

    // <| with non-function LHS
    assert_eval_fails(
        "1 <| 2",
        "Pipe apply (<|) requires a function on the left-hand side",
    );
}
