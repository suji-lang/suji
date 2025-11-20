use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use suji_lexer::Span;

use super::DiagnosticContext;
use super::error_template::ErrorTemplate;

/// Builder for creating diagnostic reports from error templates
pub struct ErrorBuilder {
    template: ErrorTemplate,
    context: DiagnosticContext,
}

impl ErrorBuilder {
    /// Create a new error builder with a template and context
    pub fn new(template: ErrorTemplate, context: DiagnosticContext) -> Self {
        Self { template, context }
    }

    /// Internal unified builder to reduce duplication across public print methods
    fn build_and_print(
        self,
        range: Option<std::ops::Range<usize>>,
        include_label: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut report = match &range {
            Some(r) => Report::build(ReportKind::Error, (&self.context.file_id, r.clone())),
            None => Report::build(ReportKind::Error, (&self.context.file_id, 0..0)),
        }
        .with_code(self.template.code)
        .with_message(self.template.title);

        if include_label && let Some(r) = range.clone() {
            let mut colors = ColorGenerator::new();
            let color = colors.next();
            report = report.with_label(
                Label::new((&self.context.file_id, r))
                    .with_message(self.template.message.as_str())
                    .with_color(color),
            );
        }

        for suggestion in &self.template.suggestions {
            report = report.with_note(suggestion.as_str());
        }

        report
            .finish()
            .print((&self.context.file_id, Source::from(&self.context.source)))?;

        Ok(())
    }

    /// Build and print a diagnostic report with a specific span
    pub fn print_with_span(self, span: Span) -> Result<(), Box<dyn std::error::Error>> {
        let range = span.start..span.end;
        self.build_and_print(Some(range), true)
    }

    /// Build and print a diagnostic report with a custom range
    pub fn print_with_range(
        self,
        range: std::ops::Range<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.build_and_print(Some(range), true)
    }

    /// Build and print a diagnostic report with a custom range and no label
    pub fn print_with_range_no_label(
        self,
        range: std::ops::Range<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.build_and_print(Some(range), false)
    }
}
