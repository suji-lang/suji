//! Centralized error code definitions for diagnostics
//!
//! Ranges:
//! - Lexer:   1xx (LEX_1XX)
//! - Parser:  2xx (PARSE_2XX)
//! - Runtime: 3xx (RUNTIME_3XX)

// Lexer (LEX_1XX)
pub const LEX_UNTERMINATED_STRING: u32 = 101;
pub const LEX_UNTERMINATED_SHELL_COMMAND: u32 = 102;
pub const LEX_UNTERMINATED_REGEX: u32 = 103;
pub const LEX_INVALID_ESCAPE: u32 = 104;
pub const LEX_INVALID_NUMBER: u32 = 105;
pub const LEX_UNEXPECTED_CHARACTER: u32 = 106;

// Parser (PARSE_2XX)
pub const PARSE_UNEXPECTED_TOKEN: u32 = 201;
pub const PARSE_UNEXPECTED_EOF: u32 = 202;
pub const PARSE_GENERIC_ERROR: u32 = 203;
pub const PARSE_MULTIPLE_EXPORTS: u32 = 204;
pub const PARSE_EXPECTED_TOKEN: u32 = 205;
pub const PARSE_INVALID_IMPORT_PATH: u32 = 206;
pub const PARSE_INVALID_ALIAS: u32 = 207;

// Runtime (RUNTIME_3XX)
pub const RUNTIME_TYPE_ERROR: u32 = 300;
pub const RUNTIME_UNDEFINED_VARIABLE: u32 = 301;
pub const RUNTIME_INVALID_OPERATION: u32 = 302;
pub const RUNTIME_INDEX_OUT_OF_BOUNDS: u32 = 303;
pub const RUNTIME_KEY_NOT_FOUND: u32 = 304;
pub const RUNTIME_INVALID_KEY_TYPE: u32 = 305;
pub const RUNTIME_SHELL_ERROR: u32 = 306;
pub const RUNTIME_REGEX_ERROR: u32 = 307;
pub const RUNTIME_ARITY_MISMATCH: u32 = 308;
pub const RUNTIME_METHOD_ERROR: u32 = 309;
pub const RUNTIME_INVALID_NUMBER_CONVERSION: u32 = 310;
pub const RUNTIME_CONTROL_FLOW: u32 = 311;
pub const RUNTIME_STRING_INDEX_ERROR: u32 = 312;
pub const RUNTIME_RANGE_ERROR: u32 = 313;
pub const RUNTIME_LIST_CONCATENATION_ERROR: u32 = 314;
pub const RUNTIME_MAP_CONTAINS_ERROR: u32 = 315;
pub const RUNTIME_CONDITIONAL_MATCH_ERROR: u32 = 316;
pub const RUNTIME_JSON_PARSE_ERROR: u32 = 317;
pub const RUNTIME_JSON_GENERATE_ERROR: u32 = 318;
pub const RUNTIME_YAML_PARSE_ERROR: u32 = 319;
pub const RUNTIME_YAML_GENERATE_ERROR: u32 = 320;
pub const RUNTIME_TOML_PARSE_ERROR: u32 = 321;
pub const RUNTIME_TOML_GENERATE_ERROR: u32 = 322;
pub const RUNTIME_TOML_CONVERSION_ERROR: u32 = 323;
pub const RUNTIME_CSV_PARSE_ERROR: u32 = 324;
pub const RUNTIME_CSV_GENERATE_ERROR: u32 = 325;
pub const RUNTIME_MAP_METHOD_ERROR: u32 = 326;
pub const RUNTIME_STREAM_ERROR: u32 = 327;
pub const RUNTIME_SERIALIZATION_ERROR: u32 = 328;
pub const RUNTIME_PIPE_STAGE_TYPE_ERROR: u32 = 329;
pub const RUNTIME_EMPTY_PIPE_EXPRESSION: u32 = 330;
pub const RUNTIME_PIPE_EXECUTION_ERROR: u32 = 331;
pub const RUNTIME_PIPE_APPLY_RIGHT_TYPE_ERROR: u32 = 332;
pub const RUNTIME_PIPE_APPLY_LEFT_TYPE_ERROR: u32 = 333;
pub const RUNTIME_DESTRUCTURE_TYPE_ERROR: u32 = 334;
pub const RUNTIME_DESTRUCTURE_ARITY_MISMATCH: u32 = 335;
pub const RUNTIME_DESTRUCTURE_INVALID_TARGET: u32 = 336;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn lexer_codes_unique_and_in_range() {
        let codes = vec![
            LEX_UNTERMINATED_STRING,
            LEX_UNTERMINATED_SHELL_COMMAND,
            LEX_UNTERMINATED_REGEX,
            LEX_INVALID_ESCAPE,
            LEX_INVALID_NUMBER,
            LEX_UNEXPECTED_CHARACTER,
        ];

        let mut set = HashSet::new();
        for &c in &codes {
            assert!(set.insert(c), "duplicate lexer code: {}", c);
            assert!(
                (100..200).contains(&c),
                "lexer code not in 1xx range: {}",
                c
            );
        }
    }

    #[test]
    fn parser_codes_unique_and_in_range() {
        let codes = vec![
            PARSE_UNEXPECTED_TOKEN,
            PARSE_UNEXPECTED_EOF,
            PARSE_GENERIC_ERROR,
            PARSE_MULTIPLE_EXPORTS,
        ];

        let mut set = HashSet::new();
        for &c in &codes {
            assert!(set.insert(c), "duplicate parser code: {}", c);
            assert!(
                (200..300).contains(&c),
                "parser code not in 2xx range: {}",
                c
            );
        }
    }

    #[test]
    fn runtime_codes_unique_and_in_range() {
        let codes = vec![
            RUNTIME_TYPE_ERROR,
            RUNTIME_UNDEFINED_VARIABLE,
            RUNTIME_INVALID_OPERATION,
            RUNTIME_INDEX_OUT_OF_BOUNDS,
            RUNTIME_KEY_NOT_FOUND,
            RUNTIME_INVALID_KEY_TYPE,
            RUNTIME_SHELL_ERROR,
            RUNTIME_REGEX_ERROR,
            RUNTIME_ARITY_MISMATCH,
            RUNTIME_METHOD_ERROR,
            RUNTIME_INVALID_NUMBER_CONVERSION,
            RUNTIME_CONTROL_FLOW,
            RUNTIME_STRING_INDEX_ERROR,
            RUNTIME_RANGE_ERROR,
            RUNTIME_LIST_CONCATENATION_ERROR,
            RUNTIME_MAP_CONTAINS_ERROR,
            RUNTIME_CONDITIONAL_MATCH_ERROR,
            RUNTIME_JSON_PARSE_ERROR,
            RUNTIME_JSON_GENERATE_ERROR,
            RUNTIME_YAML_PARSE_ERROR,
            RUNTIME_YAML_GENERATE_ERROR,
            RUNTIME_TOML_PARSE_ERROR,
            RUNTIME_TOML_GENERATE_ERROR,
            RUNTIME_TOML_CONVERSION_ERROR,
            RUNTIME_CSV_PARSE_ERROR,
            RUNTIME_CSV_GENERATE_ERROR,
            RUNTIME_MAP_METHOD_ERROR,
            RUNTIME_STREAM_ERROR,
            RUNTIME_SERIALIZATION_ERROR,
            RUNTIME_PIPE_STAGE_TYPE_ERROR,
            RUNTIME_EMPTY_PIPE_EXPRESSION,
            RUNTIME_PIPE_EXECUTION_ERROR,
            RUNTIME_PIPE_APPLY_RIGHT_TYPE_ERROR,
            RUNTIME_PIPE_APPLY_LEFT_TYPE_ERROR,
            RUNTIME_DESTRUCTURE_TYPE_ERROR,
            RUNTIME_DESTRUCTURE_ARITY_MISMATCH,
            RUNTIME_DESTRUCTURE_INVALID_TARGET,
        ];

        let mut set = HashSet::new();
        for &c in &codes {
            assert!(set.insert(c), "duplicate runtime code: {}", c);
            assert!(
                (300..400).contains(&c),
                "runtime code not in 3xx range: {}",
                c
            );
        }
    }
}
