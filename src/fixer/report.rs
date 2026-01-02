//! Report formatting for fix operations
//!
//! This module provides functions to format fix results for human-readable
//! output, including diff previews and status reports.

use super::{FileFixResult, FixResult, FixStatus};
use colored::Colorize;

/// Generate a human-readable fix report
pub fn format_report(result: &FixResult, use_colors: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str("Instrumentation Fix Report\n");
    output.push_str("==========================\n\n");

    // Summary
    output.push_str(&format!("Total gaps: {}\n", result.total_gaps));

    if result.applied > 0 {
        let applied_str = if use_colors {
            format!("Applied: {}", result.applied).green().to_string()
        } else {
            format!("Applied: {}", result.applied)
        };
        output.push_str(&applied_str);
        output.push('\n');
    } else {
        output.push_str(&format!(
            "Would apply: {}\n",
            result.total_gaps - result.skipped - result.failed
        ));
    }

    if result.skipped > 0 {
        let skipped_str = if use_colors {
            format!("Skipped: {}", result.skipped).yellow().to_string()
        } else {
            format!("Skipped: {}", result.skipped)
        };
        output.push_str(&skipped_str);
        output.push('\n');
    }

    if result.failed > 0 {
        let failed_str = if use_colors {
            format!("Failed: {}", result.failed).red().to_string()
        } else {
            format!("Failed: {}", result.failed)
        };
        output.push_str(&failed_str);
        output.push('\n');
    }

    output.push('\n');

    // Per-file details
    for file_result in &result.files {
        output.push_str(&format_file_result(file_result, use_colors));
    }

    output
}

fn format_file_result(file_result: &FileFixResult, use_colors: bool) -> String {
    let mut output = String::new();

    output.push_str(&format!("File: {}\n", file_result.file.display()));

    for attempt in &file_result.attempts {
        let status_str = format_status(&attempt.status, use_colors);
        let fn_name = &attempt.gap.location.function_name;
        let line = attempt.gap.location.line;

        output.push_str(&format!("  {} {} at line {}\n", status_str, fn_name, line));

        // Show diff for applied/dry-run
        if let Some(diff) = &attempt.diff {
            if matches!(attempt.status, FixStatus::Applied | FixStatus::DryRun) {
                for line in diff.lines() {
                    output.push_str(&format!("    {}\n", format_diff_line(line, use_colors)));
                }
            }
        }
    }

    // Show backup path if created
    if let Some(backup) = &file_result.backup_path {
        output.push_str(&format!("\nBackup created: {}\n", backup.display()));
    }

    output.push('\n');
    output
}

fn format_status(status: &FixStatus, use_colors: bool) -> String {
    match status {
        FixStatus::Applied => {
            if use_colors {
                "[APPLIED]".green().to_string()
            } else {
                "[APPLIED]".to_string()
            }
        }
        FixStatus::DryRun => {
            if use_colors {
                "[DRY-RUN]".yellow().to_string()
            } else {
                "[DRY-RUN]".to_string()
            }
        }
        FixStatus::Skipped { reason } => {
            if use_colors {
                format!("[SKIPPED: {}]", reason).dimmed().to_string()
            } else {
                format!("[SKIPPED: {}]", reason)
            }
        }
        FixStatus::Failed { error } => {
            if use_colors {
                format!("[FAILED: {}]", error).red().to_string()
            } else {
                format!("[FAILED: {}]", error)
            }
        }
    }
}

fn format_diff_line(line: &str, use_colors: bool) -> String {
    if use_colors {
        if line.starts_with('+') {
            line.green().to_string()
        } else if line.starts_with('-') {
            line.red().to_string()
        } else {
            line.dimmed().to_string()
        }
    } else {
        line.to_string()
    }
}

/// Generate a diff preview for a single fix
///
/// Shows context around the insertion point with the new line highlighted.
pub fn generate_diff(source: &str, line: usize, insertion: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let idx = line.saturating_sub(1);

    let mut diff = String::new();

    // Context before (2 lines)
    for i in idx.saturating_sub(2)..idx {
        if i < lines.len() {
            diff.push_str(&format!("  {}\n", lines[i]));
        }
    }

    // The insertion (marked with +)
    diff.push_str(&format!("+ {}\n", insertion));

    // The function line and context after (2 lines)
    for line in lines.iter().skip(idx).take(2) {
        diff.push_str(&format!("  {}\n", line));
    }

    diff
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::{GapSeverity, InstrumentationGap, Location};
    use crate::fixer::FixAttempt;
    use std::path::PathBuf;

    fn create_test_gap() -> InstrumentationGap {
        InstrumentationGap {
            location: Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
                function_name: "test_fn".to_string(),
            },
            description: "Test".to_string(),
            suggested_fix: "#[instrument]".to_string(),
            severity: GapSeverity::Critical,
        }
    }

    #[test]
    fn test_generate_diff() {
        let source = "use std::io;\n\nfn foo() {\n    println!(\"hello\");\n}\n";
        let diff = generate_diff(source, 3, "#[instrument]");

        assert!(diff.contains("+ #[instrument]"));
        assert!(diff.contains("fn foo()"));
    }

    #[test]
    fn test_format_report_empty() {
        let result = FixResult {
            files: vec![],
            total_gaps: 0,
            applied: 0,
            skipped: 0,
            failed: 0,
        };

        let report = format_report(&result, false);
        assert!(report.contains("Total gaps: 0"));
    }

    #[test]
    fn test_format_report_with_fixes() {
        let result = FixResult {
            files: vec![FileFixResult {
                file: PathBuf::from("test.rs"),
                attempts: vec![FixAttempt {
                    gap: create_test_gap(),
                    status: FixStatus::Applied,
                    diff: Some("+ #[instrument]\n  fn test_fn()".to_string()),
                }],
                backup_path: None,
                original_content: String::new(),
                modified_content: None,
            }],
            total_gaps: 1,
            applied: 1,
            skipped: 0,
            failed: 0,
        };

        let report = format_report(&result, false);
        assert!(report.contains("Total gaps: 1"));
        assert!(report.contains("Applied: 1"));
        assert!(report.contains("[APPLIED]"));
        assert!(report.contains("test_fn"));
    }

    #[test]
    fn test_format_status_variants() {
        assert_eq!(format_status(&FixStatus::Applied, false), "[APPLIED]");
        assert_eq!(format_status(&FixStatus::DryRun, false), "[DRY-RUN]");
        assert!(
            format_status(
                &FixStatus::Skipped {
                    reason: "test".to_string()
                },
                false
            )
            .contains("SKIPPED")
        );
        assert!(
            format_status(
                &FixStatus::Failed {
                    error: "test".to_string()
                },
                false
            )
            .contains("FAILED")
        );
    }
}
