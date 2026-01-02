//! Tree output formatter for human-readable output

use super::traits::{FormatterOptions, OutputFormat, OutputFormatter};
use crate::AnalysisResult;
use crate::Result;
use crate::detector::rules::{ViolationKind, ViolationSeverity};
use crate::detector::{ExistingKind, GapSeverity};
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

    fn format_existing_kind(&self, kind: &ExistingKind) -> &'static str {
        match kind {
            ExistingKind::TracingInstrument => "#[instrument]",
            ExistingKind::ManualSpan => "span!()",
            ExistingKind::LogMacro => "log macro",
            ExistingKind::Metrics => "metrics",
        }
    }

    fn format_gap_severity(&self, severity: &GapSeverity) -> ColoredString {
        if self.options.use_colors {
            match severity {
                GapSeverity::Critical => "Critical".red().bold(),
                GapSeverity::Major => "Major".yellow(),
                GapSeverity::Minor => "Minor".normal(),
            }
        } else {
            match severity {
                GapSeverity::Critical => "Critical".normal(),
                GapSeverity::Major => "Major".normal(),
                GapSeverity::Minor => "Minor".normal(),
            }
        }
    }

    fn format_quality_status(&self, score: f64) -> ColoredString {
        if self.options.use_colors {
            if score >= 0.8 {
                "✅".green()
            } else if score >= 0.5 {
                "⚠️".yellow()
            } else {
                "❌".red()
            }
        } else if score >= 0.8 {
            "OK".normal()
        } else if score >= 0.5 {
            "WARN".normal()
        } else {
            "BAD".normal()
        }
    }

    fn format_violation_severity(&self, severity: &ViolationSeverity) -> ColoredString {
        if self.options.use_colors {
            match severity {
                ViolationSeverity::Error => "ERROR".red().bold(),
                ViolationSeverity::Warning => "WARN".yellow(),
                ViolationSeverity::Info => "INFO".blue(),
            }
        } else {
            match severity {
                ViolationSeverity::Error => "ERROR".normal(),
                ViolationSeverity::Warning => "WARN".normal(),
                ViolationSeverity::Info => "INFO".normal(),
            }
        }
    }

    fn format_violation_kind(&self, kind: &ViolationKind) -> &'static str {
        match kind {
            ViolationKind::NamingConvention => "Naming",
            ViolationKind::MissingAttribute => "Missing Attr",
            ViolationKind::ForbiddenPattern => "Forbidden",
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
            "   Instrumentation:    {} points\n",
            result.stats.instrumentation_points
        ));
        output.push_str(&format!(
            "   Existing:           {} found\n",
            result.stats.existing_count
        ));
        output.push_str(&format!(
            "   Gaps:               {}\n",
            result.stats.gaps_count
        ));
        output.push_str(&format!(
            "   Rule violations:    {}\n\n",
            result.stats.rule_violations_count
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

        // Existing Instrumentation
        if !result.existing_instrumentation.is_empty() {
            output.push_str("🎯 Existing Instrumentation\n");
            for existing in &result.existing_instrumentation {
                let status = self.format_quality_status(existing.quality.score);
                let kind = self.format_existing_kind(&existing.kind);
                let name = existing
                    .span_name
                    .as_ref()
                    .map_or("(unnamed)".to_string(), |n| format!("\"{}\"", n));

                output.push_str(&format!("   {} {} {}\n", status, kind, name));
                output.push_str(&format!(
                    "      {}:{}\n",
                    existing.location.file.display(),
                    existing.location.line
                ));

                // Show quality issues if any
                for issue in &existing.quality.issues {
                    output.push_str(&format!("      ⚠️  {}\n", issue.message));
                }
            }
            output.push('\n');
        }

        // Instrumentation Gaps
        if !result.gaps.is_empty() {
            output.push_str("🚨 Instrumentation Gaps\n");
            for gap in &result.gaps {
                let severity = self.format_gap_severity(&gap.severity);
                output.push_str(&format!("   [{}] {}\n", severity, gap.description));
                output.push_str(&format!(
                    "      Location: {}:{}\n",
                    gap.location.file.display(),
                    gap.location.line
                ));
                output.push_str(&format!("      Suggested: {}\n", gap.suggested_fix));
                output.push('\n');
            }
        }

        // Rule Violations
        if !result.rule_violations.is_empty() {
            output.push_str("📋 Rule Violations\n");
            for violation in &result.rule_violations {
                let severity = self.format_violation_severity(&violation.severity);
                let kind = self.format_violation_kind(&violation.kind);
                output.push_str(&format!(
                    "   [{}] [{}] {}\n",
                    severity, kind, violation.message
                ));
                output.push_str(&format!(
                    "      Location: {}:{}\n",
                    violation.location.file.display(),
                    violation.location.line
                ));
                output.push_str(&format!("      Suggestion: {}\n", violation.suggestion));
                output.push('\n');
            }
        }

        // Instrumentation points (for backward compatibility, but less prominent)
        if !result.points.is_empty() && result.existing_instrumentation.is_empty() {
            output.push_str("📍 Suggested Instrumentation Points\n");
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
