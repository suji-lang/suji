//! Built-in: csv:generate(rows, delimiter) -> string.

use suji_runtime::value::{RuntimeError, Value};

/// Generate CSV string from SUJI list of lists.
pub fn builtin_csv_generate(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            message: "csv:generate() takes 1 or 2 arguments".to_string(),
        });
    }

    let rows = match &args[0] {
        Value::List(rows) => rows,
        other => {
            return Err(RuntimeError::CsvGenerateError {
                message: "csv:generate expects a list of lists".to_string(),
                value_type: other.type_name().to_string(),
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
    if rows.is_empty() {
        return Ok(Value::String(String::new()));
    }

    // Build CSV
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(vec![]);

    for row_value in rows {
        let row = match row_value {
            Value::List(row) => row,
            other => {
                return Err(RuntimeError::CsvGenerateError {
                    message: "csv:generate expects all rows to be lists".to_string(),
                    value_type: other.type_name().to_string(),
                });
            }
        };

        let record: Result<Vec<String>, RuntimeError> = row
            .iter()
            .map(|cell| match cell {
                Value::String(s) => Ok(s.clone()),
                other => Err(RuntimeError::CsvGenerateError {
                    message: "csv:generate expects all cells to be strings".to_string(),
                    value_type: other.type_name().to_string(),
                }),
            })
            .collect();

        writer
            .write_record(&record?)
            .map_err(|e| RuntimeError::CsvGenerateError {
                message: format!("Failed to write CSV record: {}", e),
                value_type: "List".to_string(),
            })?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|e| RuntimeError::CsvGenerateError {
            message: format!("Failed to finalize CSV: {}", e),
            value_type: "List".to_string(),
        })?;

    let csv_string = String::from_utf8(bytes).map_err(|_| RuntimeError::CsvGenerateError {
        message: "CSV output is not valid UTF-8".to_string(),
        value_type: "List".to_string(),
    })?;

    Ok(Value::String(csv_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_generate_basic() {
        let rows = Value::List(vec![
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
            ]),
            Value::List(vec![
                Value::String("1".to_string()),
                Value::String("2".to_string()),
            ]),
        ]);
        let result = builtin_csv_generate(&[rows]).unwrap();
        if let Value::String(csv) = result {
            assert_eq!(csv, "a,b\n1,2\n");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_csv_generate_custom_delimiter() {
        let rows = Value::List(vec![Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ])]);
        let result = builtin_csv_generate(&[rows, Value::String("|".to_string())]).unwrap();
        if let Value::String(csv) = result {
            assert_eq!(csv, "a|b\n");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_csv_generate_empty_input() {
        let rows = Value::List(vec![]);
        let result = builtin_csv_generate(&[rows]).unwrap();
        assert_eq!(result, Value::String(String::new()));
    }

    #[test]
    fn test_csv_generate_quoted_fields() {
        let rows = Value::List(vec![Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b,c".to_string()),
            Value::String("d".to_string()),
        ])]);
        let result = builtin_csv_generate(&[rows]).unwrap();
        if let Value::String(csv) = result {
            // CSV should quote fields containing delimiter
            assert!(csv.contains("\"b,c\""));
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_csv_generate_non_string_cells() {
        let rows = Value::List(vec![Value::List(vec![
            Value::Number(suji_runtime::value::DecimalNumber::parse("1").unwrap()),
            Value::String("b".to_string()),
        ])]);
        let result = builtin_csv_generate(&[rows]);
        assert!(matches!(result, Err(RuntimeError::CsvGenerateError { .. })));
    }

    #[test]
    fn test_csv_generate_non_list_row() {
        let rows = Value::List(vec![
            Value::List(vec![Value::String("a".to_string())]),
            Value::String("not a list".to_string()),
        ]);
        let result = builtin_csv_generate(&[rows]);
        assert!(matches!(result, Err(RuntimeError::CsvGenerateError { .. })));
    }

    #[test]
    fn test_csv_generate_non_list_input() {
        let result = builtin_csv_generate(&[Value::String("not a list".to_string())]);
        assert!(matches!(result, Err(RuntimeError::CsvGenerateError { .. })));
    }

    #[test]
    fn test_csv_generate_invalid_delimiter() {
        let rows = Value::List(vec![Value::List(vec![Value::String("a".to_string())])]);
        let result = builtin_csv_generate(&[rows, Value::String("ab".to_string())]);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    #[test]
    fn test_csv_generate_wrong_arity() {
        let result = builtin_csv_generate(&[]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));

        let rows = Value::List(vec![]);
        let result = builtin_csv_generate(&[
            rows,
            Value::String(",".to_string()),
            Value::String("extra".to_string()),
        ]);
        assert!(matches!(result, Err(RuntimeError::ArityMismatch { .. })));
    }
}
