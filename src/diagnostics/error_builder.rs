use crate::token::Span;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

use super::DiagnosticContext;
use super::templates::ErrorTemplate;

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

    /// Build and print a diagnostic report
    pub fn print(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut colors = ColorGenerator::new();
        let _color = colors.next();

        let mut report = Report::build(ReportKind::Error, (&self.context.file_id, 0..0))
            .with_code(self.template.code)
            .with_message(self.template.title);

        // Add suggestions as notes
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
        let mut colors = ColorGenerator::new();
        let _color = colors.next();

        let range = span.start..span.end;
        let mut report = Report::build(ReportKind::Error, (&self.context.file_id, range.clone()))
            .with_code(self.template.code)
            .with_message(self.template.title)
            .with_label(
                Label::new((&self.context.file_id, range))
                    .with_message(self.template.message.as_str())
                    .with_color(_color),
            );

        // Add suggestions as notes
        for suggestion in &self.template.suggestions {
            report = report.with_note(suggestion.as_str());
        }

        report
            .finish()
            .print((&self.context.file_id, Source::from(&self.context.source)))?;

        Ok(())
    }

    /// Build and print a diagnostic report with a custom range
    pub fn print_with_range(
        self,
        range: std::ops::Range<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut colors = ColorGenerator::new();
        let _color = colors.next();

        let mut report = Report::build(ReportKind::Error, (&self.context.file_id, range.clone()))
            .with_code(self.template.code)
            .with_message(self.template.title)
            .with_label(
                Label::new((&self.context.file_id, range))
                    .with_message(self.template.message.as_str())
                    .with_color(_color),
            );

        // Add suggestions as notes
        for suggestion in &self.template.suggestions {
            report = report.with_note(suggestion.as_str());
        }

        report
            .finish()
            .print((&self.context.file_id, Source::from(&self.context.source)))?;

        Ok(())
    }

    /// Build and print a diagnostic report with a custom range and no label
    pub fn print_with_range_no_label(
        self,
        range: std::ops::Range<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut report = Report::build(ReportKind::Error, (&self.context.file_id, range))
            .with_code(self.template.code)
            .with_message(self.template.title);

        // Add suggestions as notes
        for suggestion in &self.template.suggestions {
            report = report.with_note(suggestion.as_str());
        }

        report
            .finish()
            .print((&self.context.file_id, Source::from(&self.context.source)))?;

        Ok(())
    }
}

/// Helper function to convert a Span to ariadne's range format
pub fn span_to_range(span: &Span) -> std::ops::Range<usize> {
    span.start..span.end
}

/// Helper function to convert line/column to ariadne's range format (for errors without spans)
pub fn line_column_to_range(source: &str, line: usize, column: usize) -> std::ops::Range<usize> {
    // Handle the case where the source ends with a newline (common case)
    let lines: Vec<&str> = source.lines().collect();
    let actual_line_count = if source.ends_with('\n') {
        lines.len() + 1
    } else {
        lines.len()
    };

    if line > 0 && line <= actual_line_count {
        // Calculate the byte position of the start of the line
        let mut pos = 0;
        for (i, line_content) in source.lines().enumerate() {
            if i == line - 1 {
                break;
            }
            pos += line_content.len() + 1; // +1 for the newline character
        }
        // Add the column offset (column is 1-based, so subtract 1)
        let final_pos = pos + column.saturating_sub(1);
        final_pos..final_pos
    } else {
        0..0
    }
}
