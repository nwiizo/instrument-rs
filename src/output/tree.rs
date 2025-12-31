//! Tree output formatter for human-readable output

use super::traits::{FormatterOptions, OutputFormat, OutputFormatter};
use crate::AnalysisResult;
use crate::Result;
use colored::*;

/// Tree formatter for human-readable output
pub struct TreeFormatter {
    options: FormatterOptions,
}

impl TreeFormatter {
    /// Create a new tree formatter
    pub fn new(options: FormatterOptions) -> Self {
        Self { options }
    }

    fn format_priority(&self, priority: &crate::detector::Priority) -> ColoredString {
        let name = priority.name();
        if self.options.use_colors {
            match priority {
                crate::detector::Priority::Critical => name.red().bold(),
                crate::detector::Priority::High => name.yellow().bold(),
                crate::detector::Priority::Medium => name.blue(),
                crate::detector::Priority::Low => name.normal(),
            }
        } else {
            name.normal()
        }
    }
}

impl OutputFormatter for TreeFormatter {
    fn format(&self, result: &AnalysisResult) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str("instrument-rs Analysis Results\n");
        output.push_str("═══════════════════════════════\n\n");

        // Statistics
        output.push_str("📊 Statistics\n");
        output.push_str(&format!(
            "   Files analyzed:     {}\n",
            result.stats.total_files
        ));
        output.push_str(&format!(
            "   Functions found:    {}\n",
            result.stats.total_functions
        ));
        output.push_str(&format!(
            "   Lines of code:      {}\n",
            result.stats.total_lines
        ));
        output.push_str(&format!(
            "   Endpoints:          {}\n",
            result.stats.endpoints_count
        ));
        output.push_str(&format!(
            "   Instrumentation:    {} points\n\n",
            result.stats.instrumentation_points
        ));

        // Endpoints
        if !result.endpoints.is_empty() {
            output.push_str("🔗 Detected Endpoints\n");
            for endpoint in &result.endpoints {
                output.push_str(&format!(
                    "   {} {} → {}\n",
                    endpoint.method, endpoint.path, endpoint.handler
                ));
                output.push_str(&format!(
                    "      {}:{}\n",
                    endpoint.location.file.display(),
                    endpoint.location.line
                ));
            }
            output.push('\n');
        }

        // Instrumentation points
        if !result.points.is_empty() {
            output.push_str("📍 Instrumentation Points\n");
            for point in &result.points {
                let priority = self.format_priority(&point.priority);
                output.push_str(&format!(
                    "   [{priority}] {} ({})\n",
                    point.location.function_name,
                    point.kind.name()
                ));
                output.push_str(&format!("      Reason: {}\n", point.reason));
                output.push_str(&format!(
                    "      Suggested span: {}\n",
                    point.suggested_span_name
                ));
                output.push_str(&format!(
                    "      Location: {}:{}\n",
                    point.location.file.display(),
                    point.location.line
                ));
                output.push('\n');
            }
        }

        Ok(output)
    }

    fn format_type(&self) -> OutputFormat {
        OutputFormat::Tree
    }
}
