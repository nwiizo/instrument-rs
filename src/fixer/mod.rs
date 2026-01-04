//! Auto-fix module for instrument-rs
//!
//! This module provides functionality to automatically insert `#[instrument]`
//! attributes into Rust source files based on detected instrumentation gaps.

pub mod backup;
pub mod insertion;
pub mod report;
pub mod validation;

use crate::Result;
use crate::detector::{GapSeverity, InstrumentationGap};
use std::collections::HashMap;
use std::path::PathBuf;

pub use insertion::{PlannedInsertion, apply_insertions, plan_insertions};
pub use report::format_report;
pub use validation::validate_syntax;

/// Configuration for the fixer
#[derive(Debug, Clone, Default)]
pub struct FixerConfig {
    /// Apply fixes (false = dry-run only)
    pub apply: bool,
    /// Create backup files
    pub backup: bool,
    /// Minimum severity to fix (None = fix all)
    pub min_severity: Option<GapSeverity>,
    /// Maximum fixes to apply (None = unlimited)
    pub max_fixes: Option<usize>,
}

/// Result of attempting to fix a single gap
#[derive(Debug, Clone)]
pub struct FixAttempt {
    /// The gap that was fixed
    pub gap: InstrumentationGap,
    /// Status of the fix attempt
    pub status: FixStatus,
    /// Diff preview of the change
    pub diff: Option<String>,
}

/// Status of a fix attempt
#[derive(Debug, Clone)]
pub enum FixStatus {
    /// Fix was applied successfully
    Applied,
    /// Fix shown in dry-run mode
    DryRun,
    /// Fix was skipped (filtered out)
    Skipped {
        /// Reason for skipping
        reason: String,
    },
    /// Fix failed
    Failed {
        /// Error message
        error: String,
    },
}

/// Result of fixing all gaps in a file
#[derive(Debug)]
pub struct FileFixResult {
    /// Path to the file
    pub file: PathBuf,
    /// Fix attempts for this file
    pub attempts: Vec<FixAttempt>,
    /// Path to backup file if created
    pub backup_path: Option<PathBuf>,
    /// Original file content
    pub original_content: String,
    /// Modified content (if applied)
    pub modified_content: Option<String>,
}

/// Overall fix result
#[derive(Debug)]
pub struct FixResult {
    /// Results per file
    pub files: Vec<FileFixResult>,
    /// Total number of gaps found
    pub total_gaps: usize,
    /// Number of fixes applied
    pub applied: usize,
    /// Number of fixes skipped
    pub skipped: usize,
    /// Number of fixes that failed
    pub failed: usize,
}

/// The main fixer that applies instrumentation fixes
pub struct Fixer {
    config: FixerConfig,
}

impl Fixer {
    /// Create a new fixer with the given configuration
    pub fn new(config: FixerConfig) -> Self {
        Self { config }
    }

    /// Apply fixes to all detected gaps
    pub fn apply_fixes(&self, gaps: Vec<InstrumentationGap>) -> Result<FixResult> {
        // Group gaps by file
        let mut gaps_by_file: HashMap<PathBuf, Vec<InstrumentationGap>> = HashMap::new();

        for gap in gaps {
            gaps_by_file
                .entry(gap.location.file.clone())
                .or_default()
                .push(gap);
        }

        let mut file_results = Vec::new();
        let mut total_applied = 0;
        let mut total_skipped = 0;
        let mut total_failed = 0;
        let total_gaps: usize = gaps_by_file.values().map(|v| v.len()).sum();

        for (file_path, file_gaps) in gaps_by_file {
            let result = self.fix_file(&file_path, file_gaps)?;

            for attempt in &result.attempts {
                match &attempt.status {
                    FixStatus::Applied | FixStatus::DryRun => total_applied += 1,
                    FixStatus::Skipped { .. } => total_skipped += 1,
                    FixStatus::Failed { .. } => total_failed += 1,
                }
            }

            file_results.push(result);
        }

        Ok(FixResult {
            files: file_results,
            total_gaps,
            applied: total_applied,
            skipped: total_skipped,
            failed: total_failed,
        })
    }

    fn fix_file(&self, path: &PathBuf, gaps: Vec<InstrumentationGap>) -> Result<FileFixResult> {
        // Read the file
        let original_content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                // Return a result with all gaps marked as failed
                let attempts = gaps
                    .into_iter()
                    .map(|gap| FixAttempt {
                        gap,
                        status: FixStatus::Failed {
                            error: format!("Failed to read file: {}", e),
                        },
                        diff: None,
                    })
                    .collect();

                return Ok(FileFixResult {
                    file: path.clone(),
                    attempts,
                    backup_path: None,
                    original_content: String::new(),
                    modified_content: None,
                });
            }
        };

        let mut attempts = Vec::new();

        // Filter by severity
        let (filtered_gaps, skipped_gaps): (Vec<_>, Vec<_>) =
            gaps.into_iter().partition(|g| self.should_fix(g));

        // Add skipped gaps to attempts
        for gap in skipped_gaps {
            attempts.push(FixAttempt {
                gap,
                status: FixStatus::Skipped {
                    reason: "Severity filter".to_string(),
                },
                diff: None,
            });
        }

        // Apply max_fixes limit
        let gaps_to_fix: Vec<_> = if let Some(max) = self.config.max_fixes {
            let mut gaps = filtered_gaps;
            if gaps.len() > max {
                let skipped: Vec<_> = gaps.drain(max..).collect();
                for gap in skipped {
                    attempts.push(FixAttempt {
                        gap,
                        status: FixStatus::Skipped {
                            reason: "Max fixes limit reached".to_string(),
                        },
                        diff: None,
                    });
                }
            }
            gaps
        } else {
            filtered_gaps
        };

        if gaps_to_fix.is_empty() {
            return Ok(FileFixResult {
                file: path.clone(),
                attempts,
                backup_path: None,
                original_content,
                modified_content: None,
            });
        }

        // Plan insertions
        let gap_refs: Vec<_> = gaps_to_fix.iter().collect();
        let insertions = plan_insertions(&original_content, &gap_refs);

        // Generate diffs for each gap
        // Note: insertions are sorted descending for application, so we need to
        // match each gap with its corresponding insertion by target line
        for gap in &gaps_to_fix {
            // Find the insertion that matches this gap's line
            let insertion = insertions
                .iter()
                .find(|i| i.target_line == gap.location.line)
                .or_else(|| insertions.first());

            let diff = insertion.map(|ins| {
                report::generate_diff(
                    &original_content,
                    ins.target_line,
                    &format!("{}{}", ins.indentation, ins.content),
                )
            });

            let status = if self.config.apply {
                FixStatus::Applied
            } else {
                FixStatus::DryRun
            };

            attempts.push(FixAttempt {
                gap: gap.clone(),
                status,
                diff,
            });
        }

        // Apply changes if not dry-run
        let (modified_content, backup_path) = if self.config.apply && !insertions.is_empty() {
            // First, ensure use statement exists
            let (source_with_use, use_added) = insertion::ensure_use_statement(&original_content);

            // Re-plan insertions if use statement was added (line numbers shifted)
            let adjusted_insertions = if use_added {
                // Recalculate with adjusted line numbers
                let adjusted_gaps: Vec<_> = gaps_to_fix
                    .iter()
                    .map(|g| {
                        let mut adjusted = g.clone();
                        adjusted.location.line += 1; // Account for added use statement
                        adjusted
                    })
                    .collect();
                let gap_refs: Vec<_> = adjusted_gaps.iter().collect();
                plan_insertions(&source_with_use, &gap_refs)
            } else {
                insertions.clone()
            };

            // Then apply attribute insertions
            let new_content = apply_insertions(&source_with_use, &adjusted_insertions);

            // Validate syntax
            if let Err(e) = validate_syntax(&new_content) {
                // Mark all attempts as failed
                for attempt in &mut attempts {
                    if matches!(attempt.status, FixStatus::Applied) {
                        attempt.status = FixStatus::Failed {
                            error: format!("Syntax validation failed: {}", e),
                        };
                    }
                }
                return Ok(FileFixResult {
                    file: path.clone(),
                    attempts,
                    backup_path: None,
                    original_content,
                    modified_content: None,
                });
            }

            // Create backup if requested
            let backup = if self.config.backup {
                Some(backup::create_backup(path)?)
            } else {
                None
            };

            // Write modified content
            std::fs::write(path, &new_content)?;

            (Some(new_content), backup)
        } else {
            (None, None)
        };

        Ok(FileFixResult {
            file: path.clone(),
            attempts,
            backup_path,
            original_content,
            modified_content,
        })
    }

    fn should_fix(&self, gap: &InstrumentationGap) -> bool {
        match self.config.min_severity {
            Some(GapSeverity::Critical) => matches!(gap.severity, GapSeverity::Critical),
            Some(GapSeverity::Major) => {
                matches!(gap.severity, GapSeverity::Critical | GapSeverity::Major)
            }
            Some(GapSeverity::Minor) | None => true,
        }
    }
}
