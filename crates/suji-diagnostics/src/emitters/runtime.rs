use suji_values::RuntimeError;

use crate::context::DiagnosticContext;
use crate::error_builder::ErrorBuilder;
use crate::error_template::ErrorTemplate;
use crate::runtime_errors::ErrorTemplateRouter;
use crate::suggestions::{find_similar_variables, find_variable_usage};

use super::parse::print_parse_error;

pub fn print_runtime_error(
    error: RuntimeError,
    context: &DiagnosticContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if error has an embedded span (preferred path)
    if let Some(span) = error.span() {
        // Use the embedded span for precise error highlighting
        let template = error.without_span().to_template();
        ErrorBuilder::new(template, context.clone()).print_with_span(span)?;
        return Ok(());
    }

    if let RuntimeError::Parse(parse_error) = error {
        return print_parse_error(parse_error, context);
    }

    // Fall back to special cases and heuristics for errors without spans
    if let RuntimeError::UndefinedVariable { name } = error.without_span() {
        let suggestions = find_similar_variables(name, &context.source);
        let mut template = error.to_template();

        if !suggestions.is_empty() {
            template = ErrorTemplate::new(template.code, template.title, &template.message)
                .with_suggestion(&format!(
                    "Variable '{}' is not defined. Did you mean: {}?",
                    name,
                    suggestions.join(", ")
                ));
        }

        // Try to find the variable usage in the source code
        if let Some(span) = find_variable_usage(name, &context.source) {
            ErrorBuilder::new(template, context.clone()).print_with_range(span)?;
        } else {
            ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0)?;
        }
    } else {
        // Use centralized router for all other errors
        let template = error.to_template();
        ErrorBuilder::new(template, context.clone()).print_with_range_no_label(0..0)?;
    }
    Ok(())
}
