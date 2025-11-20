use suji_parser::ParseError;

use crate::context::DiagnosticContext;
use crate::error_builder::ErrorBuilder;
use crate::error_template::ErrorTemplate;
use crate::parser_errors;

use super::lex::print_lex_error;

pub fn print_parse_error(
    error: ParseError,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match error {
        ParseError::Lex(lex_error) => print_lex_error(lex_error, context)?,
        ParseError::UnexpectedToken { token, span } => {
            let base = parser_errors::unexpected_token();
            let suggestion = parser_errors::unexpected_token_suggestion(&token);
            let template = ErrorTemplate::new(base.code, base.title, &base.message)
                .with_suggestion(&suggestion);
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        ParseError::UnexpectedEof => {
            let range = context.source.len()..context.source.len();
            ErrorBuilder::new(parser_errors::unexpected_eof(), context.clone())
                .print_with_range_no_label(range)?;
        }
        ParseError::Generic { message } => {
            let template = parser_errors::generic_parse_error(&message);
            ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0)?;
        }
        ParseError::MultipleExports { span } => {
            ErrorBuilder::new(parser_errors::multiple_exports(), context.clone())
                .print_with_span(span)?;
        }
        ParseError::ExpectedToken {
            expected,
            found,
            span,
        } => {
            let template = parser_errors::expected_token(&expected, &found);
            ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        }
        ParseError::InvalidImportPath { span } => {
            ErrorBuilder::new(parser_errors::invalid_import_path(), context.clone())
                .print_with_span(span)?;
        }
        ParseError::InvalidAlias { span } => {
            ErrorBuilder::new(parser_errors::invalid_alias(), context.clone())
                .print_with_span(span)?;
        }
    }
    Ok(())
}
