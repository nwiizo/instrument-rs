//! Detection of existing instrumentation in code
//!
//! Finds existing tracing, logging, and metrics instrumentation.

use super::{
    ExistingInstrumentation, ExistingKind, InstrumentationQuality, Location, QualityIssue,
    QualityIssueKind,
};
use crate::ast::SourceFile;
use std::path::PathBuf;

/// Detect existing instrumentation in source files
pub fn detect_existing_instrumentation(files: &[SourceFile]) -> Vec<ExistingInstrumentation> {
    let mut results = Vec::new();

    for file in files {
        results.extend(detect_in_file(file));
    }

    results
}

fn detect_in_file(file: &SourceFile) -> Vec<ExistingInstrumentation> {
    let mut results = Vec::new();
    let source = file.source();
    let path = file.path().to_path_buf();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Detect #[instrument] macro
        if let Some(inst) = detect_instrument_macro(trimmed, &path, line_num) {
            results.push(inst);
        }

        // Detect manual span creation
        if let Some(inst) = detect_manual_span(trimmed, &path, line_num) {
            results.push(inst);
        }

        // Detect log macros
        if let Some(inst) = detect_log_macro(trimmed, &path, line_num) {
            results.push(inst);
        }

        // Detect metrics
        if let Some(inst) = detect_metrics(trimmed, &path, line_num) {
            results.push(inst);
        }
    }

    results
}

fn detect_instrument_macro(
    line: &str,
    path: &PathBuf,
    line_num: usize,
) -> Option<ExistingInstrumentation> {
    if !line.starts_with("#[instrument") && !line.starts_with("#[tracing::instrument") {
        return None;
    }

    let span_name = extract_instrument_name(line);
    let quality = assess_instrument_quality(line);

    Some(ExistingInstrumentation {
        location: Location {
            file: path.clone(),
            line: line_num + 1,
            column: 1,
            function_name: String::new(), // Will be filled by caller if needed
        },
        kind: ExistingKind::TracingInstrument,
        span_name,
        quality,
    })
}

fn detect_manual_span(
    line: &str,
    path: &PathBuf,
    line_num: usize,
) -> Option<ExistingInstrumentation> {
    // Look for tracing::span! or info_span!, debug_span!, etc.
    let span_patterns = [
        "tracing::span!",
        "info_span!",
        "debug_span!",
        "trace_span!",
        "warn_span!",
        "error_span!",
    ];

    for pattern in span_patterns {
        if line.contains(pattern) {
            let span_name = extract_span_name(line);
            return Some(ExistingInstrumentation {
                location: Location {
                    file: path.clone(),
                    line: line_num + 1,
                    column: 1,
                    function_name: String::new(),
                },
                kind: ExistingKind::ManualSpan,
                span_name,
                quality: InstrumentationQuality::default(),
            });
        }
    }

    None
}

fn detect_log_macro(
    line: &str,
    path: &PathBuf,
    line_num: usize,
) -> Option<ExistingInstrumentation> {
    // Look for log macros
    let log_patterns = [
        "tracing::info!",
        "tracing::debug!",
        "tracing::warn!",
        "tracing::error!",
        "tracing::trace!",
        "log::info!",
        "log::debug!",
        "log::warn!",
        "log::error!",
        "log::trace!",
        "info!(",
        "debug!(",
        "warn!(",
        "error!(",
        "trace!(",
    ];

    for pattern in log_patterns {
        if line.contains(pattern) {
            return Some(ExistingInstrumentation {
                location: Location {
                    file: path.clone(),
                    line: line_num + 1,
                    column: 1,
                    function_name: String::new(),
                },
                kind: ExistingKind::LogMacro,
                span_name: None,
                quality: InstrumentationQuality::default(),
            });
        }
    }

    None
}

fn detect_metrics(line: &str, path: &PathBuf, line_num: usize) -> Option<ExistingInstrumentation> {
    // Look for common metrics patterns
    let metrics_patterns = [
        "counter!",
        "gauge!",
        "histogram!",
        "metrics::counter",
        "metrics::gauge",
        "metrics::histogram",
        "prometheus::",
    ];

    for pattern in metrics_patterns {
        if line.contains(pattern) {
            return Some(ExistingInstrumentation {
                location: Location {
                    file: path.clone(),
                    line: line_num + 1,
                    column: 1,
                    function_name: String::new(),
                },
                kind: ExistingKind::Metrics,
                span_name: None,
                quality: InstrumentationQuality::default(),
            });
        }
    }

    None
}

fn extract_instrument_name(line: &str) -> Option<String> {
    // Look for name = "..." in #[instrument(name = "...")]
    if let Some(pos) = line.find("name") {
        let after_name = &line[pos..];
        if let Some(start) = after_name.find('"') {
            let rest = &after_name[start + 1..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

fn extract_span_name(line: &str) -> Option<String> {
    // Extract the first string argument from span! macro
    if let Some(start) = line.find('"') {
        let rest = &line[start + 1..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn assess_instrument_quality(line: &str) -> InstrumentationQuality {
    let mut issues = Vec::new();
    let mut score: f64 = 1.0;

    // Check for skip directive on large fields
    if line.contains("body") || line.contains("request") || line.contains("response") {
        if !line.contains("skip") {
            issues.push(QualityIssue {
                kind: QualityIssueKind::MissingSkip,
                message: "Large fields should use skip or skip_all".to_string(),
            });
            score -= 0.2;
        }
    }

    // Check for sensitive fields without redaction
    let sensitive_fields = ["password", "token", "secret", "key", "credential"];
    for field in sensitive_fields {
        if line.contains(field) && !line.contains("skip") {
            issues.push(QualityIssue {
                kind: QualityIssueKind::SensitiveData,
                message: format!("Sensitive field '{field}' should be skipped or redacted"),
            });
            score -= 0.3;
        }
    }

    // Check for err handling
    if line.contains("err") && !line.contains("err = true") && !line.contains("err(") {
        issues.push(QualityIssue {
            kind: QualityIssueKind::NoErrorHandling,
            message: "Consider adding err = true for error tracking".to_string(),
        });
        score -= 0.1;
    }

    InstrumentationQuality {
        score: score.max(0.0),
        issues,
    }
}
