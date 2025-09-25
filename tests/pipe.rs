use crate::common::eval_program;

mod common;

#[test]
fn backtick_left_to_closure() {
    let program = r#"
import std:io

destination = || {
    loop through io:stdin::read_lines() with line {
        match {
            line ~ /test/: return "output received"
        }
    }
}

out = `echo test` | destination
out
"#;
    let result = eval_program(program).unwrap();
    assert_eq!(result.to_string(), "output received");
}

#[test]
fn closure_to_backtick_right_preserves_newline() {
    let program = r#"
import std:println

source = || {
    println("alpha")
    println("beta")
    println("gamma")
}

out = source | `grep beta`
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

source = || {
    println("alpha")
    println("beta")
    println("gamma")
}

sink = || {
    loop through io:stdin::read_lines() with line {
        match {
            line ~ /beta/: return "beta received"
        }
    }
}

out = source | `grep beta` | sink
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
