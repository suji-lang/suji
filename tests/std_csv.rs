mod common;

use common::eval_program;
use suji_lang::runtime::value::Value;

#[test]
fn test_csv_module_integration() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("a,b,c\n1,2,3")
        rows::length()
    "#,
    );
    assert!(result.is_ok());
    if let Value::Number(n) = result.unwrap() {
        assert_eq!(n.to_string(), "2");
    } else {
        panic!("Expected number output");
    }
}

#[test]
fn test_csv_basic_parse() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("a,b,c\n1,2,3")
        rows[0][0]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("a".to_string()));
}

#[test]
fn test_csv_basic_generate() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate([["a", "b"], ["1", "2"]])
    "#,
    );
    assert!(result.is_ok());
    if let Value::String(csv) = result.unwrap() {
        assert_eq!(csv, "a,b\n1,2\n");
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_csv_roundtrip() {
    let result = eval_program(
        r#"
        import std:csv
        original = [["name", "age"], ["Alice", "30"], ["Bob", "25"]]
        csv_str = csv:generate(original)
        parsed = csv:parse(csv_str)
        parsed[1][0]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("Alice".to_string()));
}

#[test]
fn test_csv_custom_delimiter() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("a|b|c\n1|2|3", "|")
        rows[0][1]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("b".to_string()));
}

#[test]
fn test_csv_generate_custom_delimiter() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate([["a", "b"], ["1", "2"]], "|")
    "#,
    );
    assert!(result.is_ok());
    if let Value::String(csv) = result.unwrap() {
        assert_eq!(csv, "a|b\n1|2\n");
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_csv_quoted_fields() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("\"a\",\"b,c\",\"d\"")
        rows[0][1]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("b,c".to_string()));
}

#[test]
fn test_csv_generate_with_comma_in_field() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate([["a", "b,c", "d"]])
    "#,
    );
    assert!(result.is_ok());
    if let Value::String(csv) = result.unwrap() {
        // CSV should quote fields containing delimiter
        assert!(csv.contains("\"b,c\""));
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_csv_escaped_quotes() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("\"a\"\"b\"")
        rows[0][0]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("a\"b".to_string()));
}

#[test]
fn test_csv_multiline_quoted_field() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("\"line1\nline2\",\"b\"")
        rows[0][0]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("line1\nline2".to_string()));
}

#[test]
fn test_csv_empty_input_parse() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("")
        rows::length()
    "#,
    );
    assert!(result.is_ok());
    if let Value::Number(n) = result.unwrap() {
        assert_eq!(n.to_string(), "0");
    } else {
        panic!("Expected number output");
    }
}

#[test]
fn test_csv_empty_input_generate() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate([])
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String(String::new()));
}

#[test]
fn test_csv_direct_imports() {
    let result = eval_program(
        r#"
        import std:csv:parse
        import std:csv:generate
        rows = parse("a,b\n1,2")
        generate(rows)
    "#,
    );
    assert!(result.is_ok());
    if let Value::String(csv) = result.unwrap() {
        assert_eq!(csv, "a,b\n1,2\n");
    } else {
        panic!("Expected string output");
    }
}

#[test]
fn test_csv_semicolon_delimiter() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("a;b;c\n1;2;3", ";")
        rows[1][2]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("3".to_string()));
}

#[test]
fn test_csv_tab_delimiter() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("a\tb\tc\n1\t2\t3", "\t")
        rows[0][2]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("c".to_string()));
}

#[test]
fn test_csv_generate_error_non_string_cell() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate([[1, 2]])
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("CSV generation error")
            || err.to_string().contains("expects all cells to be strings")
    );
}

#[test]
fn test_csv_generate_error_non_list_row() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate([["a", "b"], "not a list"])
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("CSV generation error")
            || err.to_string().contains("expects all rows to be lists")
    );
}

#[test]
fn test_csv_generate_error_non_list_input() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate("not a list")
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("CSV generation error")
            || err.to_string().contains("expects a list of lists")
    );
}

#[test]
fn test_csv_parse_malformed() {
    let result = eval_program(
        r#"
        import std:csv
        csv:parse("a,b\n\"unclosed")
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("CSV parse error") || err.to_string().contains("Invalid CSV"));
}

#[test]
fn test_csv_invalid_delimiter_multi_char() {
    let result = eval_program(
        r#"
        import std:csv
        csv:parse("a,b", "ab")
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("single character"));
}

#[test]
fn test_csv_whitespace_preserved() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse(" a , b ")
        rows[0][0]
    "#,
    );
    assert!(result.is_ok());
    // CSV crate by default preserves whitespace
    assert_eq!(result.unwrap(), Value::String(" a ".to_string()));
}

#[test]
fn test_csv_multiple_rows() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("name,age\nAlice,30\nBob,25\nCharlie,35")
        rows::length()
    "#,
    );
    assert!(result.is_ok());
    if let Value::Number(n) = result.unwrap() {
        assert_eq!(n.to_string(), "4");
    } else {
        panic!("Expected number output");
    }
}

#[test]
fn test_csv_single_column() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("a\nb\nc")
        rows[1][0]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("b".to_string()));
}

#[test]
fn test_csv_empty_fields() {
    let result = eval_program(
        r#"
        import std:csv
        rows = csv:parse("a,,c\n1,2,")
        rows[0][1]
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String(String::new()));
}

#[test]
fn test_csv_generate_empty_strings() {
    let result = eval_program(
        r#"
        import std:csv
        csv:generate([["a", "", "c"], ["1", "2", ""]])
    "#,
    );
    assert!(result.is_ok());
    if let Value::String(csv) = result.unwrap() {
        assert_eq!(csv, "a,,c\n1,2,\n");
    } else {
        panic!("Expected string output");
    }
}
