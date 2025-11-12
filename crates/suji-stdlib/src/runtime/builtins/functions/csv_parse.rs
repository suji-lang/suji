//! Built-in: csv:parse(text, delimiter) -> list of lists.

use suji_values::value::{RuntimeError, Value};

/// Parse CSV text to SUJI list of lists.
pub fn builtin_csv_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "csv:parse() takes 1 or 2 arguments".to_string(),
        });
    }

    let text = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "csv:parse() first argument must be a string".to_string(),
            });
        }
    };

    let delimiter = if args.len() == 2 {
        match &args[1] {
            Value::String(d) => {
                if d.len() != 1 {
                    return Err(RuntimeError::TypeError {
                        message: "delimiter must be a single character string".to_string(),
                    });
                }
                d.chars().next().unwrap() as u8
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "delimiter must be a string".to_string(),
                });
            }
        }
    } else {
        b','
    };

    // Handle empty input
    if text.is_empty() {
        return Ok(Value::List(vec![]));
    }

    // Parse CSV (treat all rows as data, not headers)
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .from_reader(text.as_bytes());

    let mut rows = vec![];
    for result in reader.records() {
        let record = result.map_err(|e| RuntimeError::CsvParseError {
            message: format!("Invalid CSV: {}", e),
            csv_input: Some(text.clone()),
        })?;

        let row: Vec<Value> = record
            .iter()
            .map(|field| Value::String(field.to_string()))
            .collect();
        rows.push(Value::List(row));
    }

    Ok(Value::List(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parse_basic() {
        let result = builtin_csv_parse(&[Value::String("a,b,c\n1,2,3".to_string())]).unwrap();
        if let Value::List(rows) = result {
            assert_eq!(rows.len(), 2);
            if let Value::List(row1) = &rows[0] {
                assert_eq!(row1.len(), 3);
                assert_eq!(row1[0], Value::String("a".to_string()));
                assert_eq!(row1[1], Value::String("b".to_string()));
                assert_eq!(row1[2], Value::String("c".to_string()));
            } else {
                panic!("Expected list for first row");
            }
            if let Value::List(row2) = &rows[1] {
                assert_eq!(row2.len(), 3);
                assert_eq!(row2[0], Value::String("1".to_string()));
                assert_eq!(row2[1], Value::String("2".to_string()));
                assert_eq!(row2[2], Value::String("3".to_string()));
            } else {
                panic!("Expected list for second row");
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_csv_parse_custom_delimiter() {
        let result = builtin_csv_parse(&[
            Value::String("a|b|c\n1|2|3".to_string()),
            Value::String("|".to_string()),
        ])
        .unwrap();
        if let Value::List(rows) = result {
            assert_eq!(rows.len(), 2);
            if let Value::List(row1) = &rows[0] {
                assert_eq!(row1[0], Value::String("a".to_string()));
                assert_eq!(row1[1], Value::String("b".to_string()));
            } else {
                panic!("Expected list");
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_csv_parse_empty_input() {
        let result = builtin_csv_parse(&[Value::String("".to_string())]).unwrap();
        assert_eq!(result, Value::List(vec![]));
    }

    #[test]
    fn test_csv_parse_quoted_fields() {
        let result =
            builtin_csv_parse(&[Value::String("\"a\",\"b,c\",\"d\"".to_string())]).unwrap();
        if let Value::List(rows) = result {
            assert_eq!(rows.len(), 1);
            if let Value::List(row) = &rows[0] {
                assert_eq!(row.len(), 3);
                assert_eq!(row[0], Value::String("a".to_string()));
                assert_eq!(row[1], Value::String("b,c".to_string()));
                assert_eq!(row[2], Value::String("d".to_string()));
            } else {
                panic!("Expected list");
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_csv_parse_invalid_delimiter() {
        let result = builtin_csv_parse(&[
            Value::String("a,b,c".to_string()),
            Value::String("ab".to_string()),
        ]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_csv_parse_wrong_type() {
        let result = builtin_csv_parse(&[Value::Number(
            suji_values::value::DecimalNumber::parse("42").unwrap(),
        )]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_csv_parse_wrong_arity() {
        let result = builtin_csv_parse(&[]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        let result = builtin_csv_parse(&[
            Value::String("a,b".to_string()),
            Value::String(",".to_string()),
            Value::String("extra".to_string()),
        ]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }

    #[test]
    fn test_csv_parse_malformed_csv() {
        let result = builtin_csv_parse(&[Value::String("a,b\n\"unclosed".to_string())]);
        assert!(matches!(result, Err(RuntimeError::CsvParseError { .. })));
    }
}
