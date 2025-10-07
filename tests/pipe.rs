use crate::common::{assert_eval_fails, eval_program};

mod common;

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
