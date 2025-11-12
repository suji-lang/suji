use super::EvalResult;
use regex::Regex;
use std::rc::Rc;
use suji_ast::ast::Pattern;
use suji_runtime::ModuleRegistry;
use suji_values::Env;
use suji_values::{DecimalNumber, RuntimeError, Value};

/// Check if a pattern matches a value
pub fn pattern_matches(pattern: &Pattern, value: &Value) -> EvalResult<bool> {
    match pattern {
        Pattern::Wildcard { .. } => Ok(true),
        Pattern::Literal {
            value: pattern_value,
            ..
        } => Ok(value_like_matches(pattern_value, value)),
        Pattern::Tuple { patterns, .. } => match value {
            Value::Tuple(tuple_values) => {
                if patterns.len() != tuple_values.len() {
                    Ok(false)
                } else {
                    for (pattern, value) in patterns.iter().zip(tuple_values.iter()) {
                        if !pattern_matches(pattern, value)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
            }
            _ => Ok(false),
        },
        Pattern::Regex {
            pattern: regex_pattern,
            ..
        } => match value {
            Value::String(s) => {
                let regex = Regex::new(regex_pattern).map_err(|err| RuntimeError::RegexError {
                    message: format!("Invalid regex pattern '{}': {}", regex_pattern, err),
                })?;
                Ok(regex.is_match(s))
            }
            _ => Ok(false),
        },
        Pattern::Expression(_) => {
            // Expression patterns are handled separately in conditional match evaluation
            // This should not be called for conditional match
            Ok(false)
        }
    }
}

/// Check if an expression pattern matches (for conditional match)
pub fn expression_pattern_matches(
    pattern: &Pattern,
    env: Rc<Env>,
    registry: Option<&ModuleRegistry>,
) -> EvalResult<bool> {
    match pattern {
        Pattern::Expression(expr) => {
            use super::eval_expr;
            let result = eval_expr(expr, env, registry)?;
            Ok(result.is_truthy())
        }
        _ => Err(RuntimeError::ConditionalMatchError {
            message: "Only boolean expressions are allowed in conditional match conditions"
                .to_string(),
        }),
    }
}

/// Check if a ValueLike matches a Value
pub fn value_like_matches(pattern_value: &suji_ast::ast::ValueLike, value: &Value) -> bool {
    use suji_ast::ast::ValueLike;

    match (pattern_value, value) {
        (ValueLike::Number(a), Value::Number(b)) => {
            // Parse the string representation and compare with DecimalNumber
            match DecimalNumber::parse(a) {
                Ok(decimal_a) => decimal_a == *b,
                Err(_) => false,
            }
        }
        (ValueLike::Boolean(a), Value::Boolean(b)) => a == b,
        (ValueLike::String(a), Value::String(b)) => a == b,
        (ValueLike::Nil, Value::Nil) => true,
        (ValueLike::Tuple(pattern_items), Value::Tuple(value_items)) => {
            if pattern_items.len() != value_items.len() {
                false
            } else {
                pattern_items
                    .iter()
                    .zip(value_items.iter())
                    .all(|(pattern_item, value_item)| value_like_matches(pattern_item, value_item))
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suji_ast::Span;
    use suji_ast::ast::{Pattern, ValueLike};
    use suji_values::Value;

    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::Wildcard {
            span: Span::default(),
        };

        assert!(pattern_matches(&pattern, &Value::Number(DecimalNumber::from_i64(42))).unwrap());
        assert!(pattern_matches(&pattern, &Value::String("hello".to_string())).unwrap());
        assert!(pattern_matches(&pattern, &Value::Boolean(true)).unwrap());
    }

    #[test]
    fn test_literal_pattern() {
        let pattern = Pattern::Literal {
            value: ValueLike::Number("42".to_string()),
            span: Span::default(),
        };

        assert!(pattern_matches(&pattern, &Value::Number(DecimalNumber::from_i64(42))).unwrap());
        assert!(!pattern_matches(&pattern, &Value::Number(DecimalNumber::from_i64(43))).unwrap());
        assert!(!pattern_matches(&pattern, &Value::String("42".to_string())).unwrap());
    }

    #[test]
    fn test_string_literal_pattern() {
        let pattern = Pattern::Literal {
            value: ValueLike::String("hello".to_string()),
            span: Span::default(),
        };

        assert!(pattern_matches(&pattern, &Value::String("hello".to_string())).unwrap());
        assert!(!pattern_matches(&pattern, &Value::String("world".to_string())).unwrap());
        assert!(!pattern_matches(&pattern, &Value::Number(DecimalNumber::from_i64(42))).unwrap());
    }

    #[test]
    fn test_boolean_literal_pattern() {
        let pattern = Pattern::Literal {
            value: ValueLike::Boolean(true),
            span: Span::default(),
        };

        assert!(pattern_matches(&pattern, &Value::Boolean(true)).unwrap());
        assert!(!pattern_matches(&pattern, &Value::Boolean(false)).unwrap());
        assert!(!pattern_matches(&pattern, &Value::Number(DecimalNumber::from_i64(1))).unwrap());
    }

    #[test]
    fn test_tuple_pattern() {
        let pattern = Pattern::Tuple {
            patterns: vec![
                Pattern::Literal {
                    value: ValueLike::Number("1".to_string()),
                    span: Span::default(),
                },
                Pattern::Wildcard {
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::String("anything".to_string()),
        ]);

        let non_matching_tuple = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(2)),
            Value::String("anything".to_string()),
        ]);

        let wrong_length_tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(1))]);

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &non_matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &wrong_length_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &Value::Number(DecimalNumber::from_i64(1))).unwrap());
    }

    #[test]
    fn test_regex_pattern() {
        let pattern = Pattern::Regex {
            pattern: r"^hello".to_string(),
            span: Span::default(),
        };

        assert!(pattern_matches(&pattern, &Value::String("hello world".to_string())).unwrap());
        assert!(!pattern_matches(&pattern, &Value::String("world hello".to_string())).unwrap());
        assert!(!pattern_matches(&pattern, &Value::Number(DecimalNumber::from_i64(42))).unwrap());
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let pattern = Pattern::Regex {
            pattern: "[".to_string(), // Invalid regex
            span: Span::default(),
        };

        let result = pattern_matches(&pattern, &Value::String("test".to_string()));
        assert!(result.is_err());

        if let Err(RuntimeError::RegexError { message }) = result {
            assert!(message.contains("Invalid regex pattern"));
        } else {
            panic!("Expected RegexError");
        }
    }

    #[test]
    fn test_multiple_wildcards_in_tuple() {
        let pattern = Pattern::Tuple {
            patterns: vec![
                Pattern::Wildcard {
                    span: Span::default(),
                },
                Pattern::Wildcard {
                    span: Span::default(),
                },
                Pattern::Literal {
                    value: ValueLike::Number("0".to_string()),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![
            Value::String("anything".to_string()),
            Value::Boolean(true),
            Value::Number(DecimalNumber::from_i64(0)),
        ]);

        let non_matching_tuple = Value::Tuple(vec![
            Value::String("anything".to_string()),
            Value::Boolean(true),
            Value::Number(DecimalNumber::from_i64(1)),
        ]);

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &non_matching_tuple).unwrap());
    }

    #[test]
    fn test_nested_tuple_wildcards() {
        // Test ((_, _), (_, _)) pattern
        let pattern = Pattern::Tuple {
            patterns: vec![
                Pattern::Tuple {
                    patterns: vec![
                        Pattern::Wildcard {
                            span: Span::default(),
                        },
                        Pattern::Wildcard {
                            span: Span::default(),
                        },
                    ],
                    span: Span::default(),
                },
                Pattern::Tuple {
                    patterns: vec![
                        Pattern::Wildcard {
                            span: Span::default(),
                        },
                        Pattern::Wildcard {
                            span: Span::default(),
                        },
                    ],
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![
            Value::Tuple(vec![
                Value::Number(DecimalNumber::from_i64(1)),
                Value::String("test".to_string()),
            ]),
            Value::Tuple(vec![
                Value::Boolean(true),
                Value::Number(DecimalNumber::from_i64(42)),
            ]),
        ]);

        let non_matching_tuple = Value::Tuple(vec![
            Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(1))]), // Wrong length
            Value::Tuple(vec![
                Value::Boolean(true),
                Value::Number(DecimalNumber::from_i64(42)),
            ]),
        ]);

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &non_matching_tuple).unwrap());
    }

    #[test]
    fn test_all_wildcards_tuple_pattern() {
        let pattern = Pattern::Tuple {
            patterns: vec![
                Pattern::Wildcard {
                    span: Span::default(),
                },
                Pattern::Wildcard {
                    span: Span::default(),
                },
                Pattern::Wildcard {
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![
            Value::String("anything".to_string()),
            Value::Number(DecimalNumber::from_i64(42)),
            Value::Boolean(true),
        ]);

        let wrong_length_tuple = Value::Tuple(vec![
            Value::String("anything".to_string()),
            Value::Number(DecimalNumber::from_i64(42)),
        ]);

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &wrong_length_tuple).unwrap());
    }

    #[test]
    fn test_single_element_tuple_wildcard() {
        let pattern = Pattern::Tuple {
            patterns: vec![Pattern::Wildcard {
                span: Span::default(),
            }],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(42))]);
        let wrong_length_tuple = Value::Tuple(vec![]);
        let non_tuple_value = Value::Number(DecimalNumber::from_i64(42));

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &wrong_length_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &non_tuple_value).unwrap());
    }

    #[test]
    fn test_empty_tuple_pattern() {
        let pattern = Pattern::Tuple {
            patterns: vec![],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![]);
        let non_matching_tuple = Value::Tuple(vec![Value::Number(DecimalNumber::from_i64(42))]);

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &non_matching_tuple).unwrap());
    }

    #[test]
    fn test_mixed_patterns_in_tuple() {
        let pattern = Pattern::Tuple {
            patterns: vec![
                Pattern::Literal {
                    value: ValueLike::Number("1".to_string()),
                    span: Span::default(),
                },
                Pattern::Wildcard {
                    span: Span::default(),
                },
                Pattern::Literal {
                    value: ValueLike::String("test".to_string()),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Boolean(true),
            Value::String("test".to_string()),
        ]);

        let non_matching_tuple = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(2)), // Wrong first element
            Value::Boolean(true),
            Value::String("test".to_string()),
        ]);

        let non_matching_tuple2 = Value::Tuple(vec![
            Value::Number(DecimalNumber::from_i64(1)),
            Value::Boolean(true),
            Value::String("wrong".to_string()), // Wrong third element
        ]);

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &non_matching_tuple).unwrap());
        assert!(!pattern_matches(&pattern, &non_matching_tuple2).unwrap());
    }

    #[test]
    fn test_deeply_nested_wildcards() {
        // Test (((_, _), _), _) pattern
        let pattern = Pattern::Tuple {
            patterns: vec![
                Pattern::Tuple {
                    patterns: vec![
                        Pattern::Tuple {
                            patterns: vec![
                                Pattern::Wildcard {
                                    span: Span::default(),
                                },
                                Pattern::Wildcard {
                                    span: Span::default(),
                                },
                            ],
                            span: Span::default(),
                        },
                        Pattern::Wildcard {
                            span: Span::default(),
                        },
                    ],
                    span: Span::default(),
                },
                Pattern::Wildcard {
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let matching_tuple = Value::Tuple(vec![
            Value::Tuple(vec![
                Value::Tuple(vec![
                    Value::Number(DecimalNumber::from_i64(1)),
                    Value::String("test".to_string()),
                ]),
                Value::Boolean(true),
            ]),
            Value::Number(DecimalNumber::from_i64(42)),
        ]);

        assert!(pattern_matches(&pattern, &matching_tuple).unwrap());
    }
}
