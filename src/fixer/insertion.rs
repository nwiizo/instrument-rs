//! Line-based text insertion algorithm for adding #[instrument] attributes
//!
//! This module provides functions to safely insert instrumentation attributes
//! into Rust source files while preserving formatting and handling edge cases.

use crate::detector::InstrumentationGap;

/// Represents a planned insertion into source code
#[derive(Debug, Clone)]
pub struct PlannedInsertion {
    /// Line number to insert BEFORE (1-based)
    pub target_line: usize,
    /// Content to insert (without trailing newline)
    pub content: String,
    /// Indentation to apply (spaces/tabs)
    pub indentation: String,
}

/// Plan insertions for a file based on detected gaps
///
/// This function analyzes the source code and determines the optimal
/// insertion points for each gap, handling doc comments and existing attributes.
pub fn plan_insertions(source: &str, gaps: &[&InstrumentationGap]) -> Vec<PlannedInsertion> {
    let lines: Vec<&str> = source.lines().collect();
    let mut insertions = Vec::new();

    for gap in gaps {
        let target_line = gap.location.line;

        // Find the actual insertion point (after doc comments, before fn)
        let (insert_line, indentation) = find_insertion_point(&lines, target_line);

        // Extract just the #[instrument(...)] part from suggested_fix
        let attr_line = extract_instrument_attr(&gap.suggested_fix);

        insertions.push(PlannedInsertion {
            target_line: insert_line,
            content: attr_line,
            indentation,
        });
    }

    // Sort by line number descending (process bottom-up to maintain line numbers)
    insertions.sort_by(|a, b| b.target_line.cmp(&a.target_line));

    insertions
}

/// Find the optimal insertion point for an attribute
///
/// Returns (line_number, indentation) where:
/// - line_number is 1-based
/// - indentation is the whitespace prefix to use
fn find_insertion_point(lines: &[&str], fn_line: usize) -> (usize, String) {
    // fn_line is 1-based, convert to 0-based index
    let fn_idx = fn_line.saturating_sub(1);

    if fn_idx >= lines.len() {
        return (fn_line, String::new());
    }

    let fn_line_content = lines[fn_idx];

    // Extract indentation from the function line
    let indentation: String = fn_line_content
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();

    // Walk backwards to find where to insert
    // We want to insert after doc comments and other attributes, but before fn
    let mut insert_at = fn_idx;

    for i in (0..fn_idx).rev() {
        let line = lines[i].trim();

        if line.starts_with("///") || line.starts_with("//!") {
            // Doc comment - continue going up
            continue;
        } else if line.starts_with("#[") {
            // Check if it's already an instrument attribute
            if line.contains("instrument") {
                // Already has instrument, insert right before fn
                insert_at = fn_idx;
                break;
            }
            // Other attribute - insert after the first line of attributes
            // but we need to find where the attribute block starts
            continue;
        } else if line.is_empty() {
            // Blank line - this is a good boundary
            insert_at = i + 1;
            break;
        } else {
            // Some other code - insert after this
            insert_at = i + 1;
            break;
        }
    }

    // If we're at the function line, check if there are attributes above
    // and if so, insert between the last attribute and the function
    if insert_at == fn_idx && fn_idx > 0 {
        let prev_line = lines[fn_idx - 1].trim();
        if prev_line.starts_with("#[") && !prev_line.contains("instrument") {
            // There's an attribute right above, insert between it and fn
            insert_at = fn_idx;
        }
    }

    // Convert back to 1-based
    (insert_at + 1, indentation)
}

/// Extract just the #[instrument(...)] line from the suggested fix
fn extract_instrument_attr(suggested_fix: &str) -> String {
    for line in suggested_fix.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[instrument") {
            return trimmed.to_string();
        }
    }

    // Fallback: return as-is if no attribute found
    suggested_fix.lines().next().unwrap_or("").to_string()
}

/// Apply planned insertions to source code
///
/// Insertions must be sorted in descending order by line number
/// to preserve line number validity during processing.
pub fn apply_insertions(source: &str, insertions: &[PlannedInsertion]) -> String {
    let mut lines: Vec<String> = source.lines().map(String::from).collect();

    // Process in reverse order (insertions should already be sorted descending)
    for insertion in insertions {
        let idx = insertion.target_line.saturating_sub(1);
        let new_line = format!("{}{}", insertion.indentation, insertion.content);

        if idx <= lines.len() {
            lines.insert(idx, new_line);
        }
    }

    // Preserve original trailing newline
    let mut result = lines.join("\n");
    if source.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Ensure that `use tracing::instrument;` exists in the source
///
/// Returns (modified_source, was_added)
pub fn ensure_use_statement(source: &str) -> (String, bool) {
    // Check if already present
    if source.contains("use tracing::instrument")
        || source.contains("use tracing::*")
        || source.contains("use tracing::{") && source.contains("instrument")
    {
        return (source.to_string(), false);
    }

    // Also check for fully qualified usage
    if source.contains("tracing::instrument") {
        return (source.to_string(), false);
    }

    let lines: Vec<&str> = source.lines().collect();
    let mut insert_idx = 0;
    let mut found_use_block = false;
    let mut last_use_line = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip module-level doc comments and attributes at the start
        if trimmed.starts_with("//!") || trimmed.starts_with("#![") {
            insert_idx = i + 1;
            continue;
        }

        // Track use statements
        if trimmed.starts_with("use ") {
            found_use_block = true;
            last_use_line = i;
        } else if found_use_block && !trimmed.is_empty() && !trimmed.starts_with("//") {
            // End of use block
            break;
        }
    }

    // Determine insertion point
    let use_insert_idx = if found_use_block {
        // Insert after the last use statement
        last_use_line + 1
    } else {
        // Insert after module-level comments/attributes
        insert_idx
    };

    // Build the new source
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    new_lines.insert(use_insert_idx, "use tracing::instrument;".to_string());

    let mut result = new_lines.join("\n");
    if source.ends_with('\n') {
        result.push('\n');
    }

    (result, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::{GapSeverity, InstrumentationGap, Location};
    use std::path::PathBuf;

    fn create_test_gap(line: usize, suggested_fix: &str) -> InstrumentationGap {
        InstrumentationGap {
            location: Location {
                file: PathBuf::from("test.rs"),
                line,
                column: 1,
                function_name: "test_fn".to_string(),
            },
            description: "Test gap".to_string(),
            suggested_fix: suggested_fix.to_string(),
            severity: GapSeverity::Critical,
        }
    }

    #[test]
    fn test_find_insertion_point_simple() {
        let source = "fn foo() {}";
        let lines: Vec<&str> = source.lines().collect();

        let (line, indent) = find_insertion_point(&lines, 1);
        assert_eq!(line, 1);
        assert_eq!(indent, "");
    }

    #[test]
    fn test_find_insertion_point_with_doc_comment() {
        let source = "/// Doc comment\nfn foo() {}";
        let lines: Vec<&str> = source.lines().collect();

        let (line, indent) = find_insertion_point(&lines, 2);
        assert_eq!(line, 2); // Insert between doc and fn
        assert_eq!(indent, "");
    }

    #[test]
    fn test_find_insertion_point_indented() {
        let source = "impl Foo {\n    fn bar() {}\n}";
        let lines: Vec<&str> = source.lines().collect();

        let (line, indent) = find_insertion_point(&lines, 2);
        assert_eq!(indent, "    ");
    }

    #[test]
    fn test_apply_insertions_single() {
        let source = "fn foo() {}\n";
        let insertions = vec![PlannedInsertion {
            target_line: 1,
            content: "#[instrument]".to_string(),
            indentation: String::new(),
        }];

        let result = apply_insertions(source, &insertions);
        assert_eq!(result, "#[instrument]\nfn foo() {}\n");
    }

    #[test]
    fn test_apply_insertions_multiple_same_file() {
        let source = "fn foo() {}\nfn bar() {}\n";
        let insertions = vec![
            PlannedInsertion {
                target_line: 2,
                content: "#[instrument]".to_string(),
                indentation: String::new(),
            },
            PlannedInsertion {
                target_line: 1,
                content: "#[instrument]".to_string(),
                indentation: String::new(),
            },
        ];

        // Already sorted descending
        let result = apply_insertions(source, &insertions);
        assert_eq!(
            result,
            "#[instrument]\nfn foo() {}\n#[instrument]\nfn bar() {}\n"
        );
    }

    #[test]
    fn test_ensure_use_statement_adds_when_missing() {
        let source = "fn foo() {}\n";
        let (result, added) = ensure_use_statement(source);

        assert!(added);
        assert!(result.contains("use tracing::instrument;"));
    }

    #[test]
    fn test_ensure_use_statement_skips_when_present() {
        let source = "use tracing::instrument;\n\nfn foo() {}\n";
        let (result, added) = ensure_use_statement(source);

        assert!(!added);
        assert_eq!(result, source);
    }

    #[test]
    fn test_ensure_use_statement_skips_glob_import() {
        let source = "use tracing::*;\n\nfn foo() {}\n";
        let (result, added) = ensure_use_statement(source);

        assert!(!added);
        assert_eq!(result, source);
    }

    #[test]
    fn test_ensure_use_statement_after_module_doc() {
        let source = "//! Module doc\n\nfn foo() {}\n";
        let (result, added) = ensure_use_statement(source);

        assert!(added);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "//! Module doc");
        assert_eq!(lines[1], "use tracing::instrument;");
    }

    #[test]
    fn test_ensure_use_statement_with_existing_uses() {
        let source = "use std::io;\nuse std::path::Path;\n\nfn foo() {}\n";
        let (result, added) = ensure_use_statement(source);

        assert!(added);
        assert!(result.contains("use tracing::instrument;"));
        // Should be after the use block
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.iter().position(|l| l.contains("tracing")).unwrap() > 1);
    }

    #[test]
    fn test_extract_instrument_attr() {
        let fix = "#[instrument(name = \"api.test\")]";
        assert_eq!(extract_instrument_attr(fix), fix);

        let fix_with_newlines = "  #[instrument(name = \"api.test\")]  \n";
        assert_eq!(
            extract_instrument_attr(fix_with_newlines),
            "#[instrument(name = \"api.test\")]"
        );
    }

    #[test]
    fn test_plan_insertions_sorted_descending() {
        let source = "fn a() {}\nfn b() {}\nfn c() {}\n";
        let gaps = vec![
            create_test_gap(1, "#[instrument]"),
            create_test_gap(2, "#[instrument]"),
            create_test_gap(3, "#[instrument]"),
        ];
        let gap_refs: Vec<_> = gaps.iter().collect();

        let insertions = plan_insertions(source, &gap_refs);

        // Should be sorted descending
        assert_eq!(insertions.len(), 3);
        assert!(insertions[0].target_line >= insertions[1].target_line);
        assert!(insertions[1].target_line >= insertions[2].target_line);
    }
}
