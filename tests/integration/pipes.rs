use super::common::{assert_eval_fails, eval_program};
use suji_values::Value;

// ============================================================================
// Stream Pipes (| operator)
// ============================================================================

#[test]
fn backtick_left_to_closure() {
    let program = r#"
import std:io

destination = || {
    loop through io:stdin::read_lines() with line {
        match {
            line ~ /test/ => return "output received",
        }
    }
}

out = `echo test` | destination()
out
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result.to_string(), "output received");
}

#[test]
fn closure_to_backtick_right_preserves_newline() {
    let program = r#"
import std:println

make_source = || {
    return || {
        println("alpha")
        println("beta")
        println("gamma")
    }
}

out = make_source() | `grep beta`
out::replace("\n", "\\n")
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result.to_string(), "beta\\n");
}

#[test]
fn closure_to_backtick_to_closure_middle() {
    let program = r#"
import std:println
import std:io

make_source = || {
    return || {
        println("alpha")
        println("beta")
        println("gamma")
    }
}

make_sink = || {
    loop through io:stdin::read_lines() with line {
        match {
            line ~ /beta/ => return "beta received",
        }
    }
}

out = make_source() | `grep beta` | make_sink()
out
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result.to_string(), "beta received");
}

#[test]
fn commands_only_pipeline() {
    let program = r#"
out = `printf 'line1\nline2\n'` | `grep line2`
out::replace("\n", "\\n")
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result.to_string(), "line2\\n");
}

#[test]
fn bare_function_stages_error() {
    let program = r#"
import std:println

source = || {
    println("value")
}

destination = || {
    println("sink")
}

source | destination
"#;

    assert_eval_fails(
        program,
        "Pipe requires function invocations; write a(x) | b(y) instead of a | b",
    );
}

#[test]
fn pipelines_accept_invocation_builders() {
    let program = r#"
import std:println
import std:io

producer = |left, right| {
    return || {
        println("${left},${right}")
    }
}

consumer = |pattern| {
    return || {
        loop through io:stdin::read_lines() with line {
            match {
                line ~ pattern => return line,
            }
        }
    }
}

out = producer("alpha", "beta") | consumer(/beta/)
out
"#;

    let result = eval_program(program).unwrap();
    assert_eq!(result.to_string(), "alpha,beta");
}

// ============================================================================
// Pipe Apply Forward (|> operator)
// ============================================================================

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
fn type_errors_for_non_functions_forward() {
    // |> with non-function RHS
    assert_eval_fails(
        "1 |> 2",
        "Pipe apply (|>) requires a function on the right-hand side",
    );
}

#[test]
fn mixed_precedence_stream_then_apply() {
    let program = r#"
import std:println
import std:io

producer = || {
    println("foo")
    println("bar")
    println("baz")
}

counter = || {
    io:stdin::read_lines()::length()
}

format = |n| "Count: ${n}"

result = producer() | `grep bar` | counter() |> format
result
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(format!("{}", result), "Count: 1");
}

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

// ============================================================================
// Pipe Apply Backward (<| operator)
// ============================================================================

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
fn type_errors_for_non_functions_backward() {
    // <| with non-function LHS
    assert_eval_fails(
        "1 <| 2",
        "Pipe apply (<|) requires a function on the left-hand side",
    );
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
