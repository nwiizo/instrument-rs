//! JSON output formatter

use super::traits::{FormatterOptions, OutputFormat, OutputFormatter};
use crate::AnalysisResult;
use crate::Result;

/// JSON formatter for analysis results
pub struct JsonFormatter {
    options: FormatterOptions,
}

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new(options: FormatterOptions) -> Self {
        Self { options }
    }
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, result: &AnalysisResult) -> Result<String> {
        let output = serde_json::json!({
            "stats": {
                "total_files": result.stats.total_files,
                "total_functions": result.stats.total_functions,
                "total_lines": result.stats.total_lines,
                "endpoints_count": result.stats.endpoints_count,
                "instrumentation_points": result.stats.instrumentation_points,
            },
            "endpoints": result.endpoints.iter().map(|e| {
                serde_json::json!({
                    "method": e.method,
                    "path": e.path,
                    "handler": e.handler,
                    "framework": e.framework,
                    "location": {
                        "file": e.location.file.display().to_string(),
                        "line": e.location.line,
                    }
                })
            }).collect::<Vec<_>>(),
            "instrumentation_points": result.points.iter().map(|p| {
                serde_json::json!({
                    "function": p.location.function_name,
                    "file": p.location.file.display().to_string(),
                    "line": p.location.line,
                    "kind": format!("{:?}", p.kind),
                    "priority": format!("{:?}", p.priority),
                    "reason": p.reason,
                    "suggested_span_name": p.suggested_span_name,
                })
            }).collect::<Vec<_>>(),
        });

        Ok(serde_json::to_string_pretty(&output)?)
    }

    fn format_type(&self) -> OutputFormat {
        OutputFormat::Json
    }
}
