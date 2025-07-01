//! Tests for output formatters

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::reporting::{CoverageSummary, FileCoverage, LineCoverage};
    use std::path::PathBuf;

    fn create_test_coverage() -> CoverageSummary {
        let file = FileCoverage {
            file_path: PathBuf::from("test.rs"),
            total_lines: 100,
            executable_lines: 50,
            covered_lines: 40,
            line_coverage: vec![
                LineCoverage { line_number: 1, executable: true, hit_count: 5 },
                LineCoverage { line_number: 2, executable: true, hit_count: 0 },
            ],
            branch_coverage: vec![],
            function_coverage: vec![],
        };
        
        CoverageSummary::from_files(vec![file])
    }

    #[test]
    fn test_tree_formatter() {
        let formatter = TreeFormatter::new(FormatterOptions::default());
        let coverage = create_test_coverage();
        
        let output = formatter.format_coverage(&coverage).unwrap();
        assert!(output.contains("Project Coverage Summary"));
        assert!(output.contains("80.0%")); // Line coverage
        assert!(output.contains("test.rs"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new(FormatterOptions::default());
        let coverage = create_test_coverage();
        
        let output = formatter.format_coverage(&coverage).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        
        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["generated_at"].is_string());
        assert_eq!(parsed["summary"]["line_coverage_percent"], 80.0);
    }

    #[test]
    fn test_mermaid_formatter() {
        let formatter = MermaidFormatter::new(FormatterOptions::default());
        let coverage = create_test_coverage();
        
        let output = formatter.format_coverage(&coverage).unwrap();
        assert!(output.contains("```mermaid"));
        assert!(output.contains("flowchart TD"));
        assert!(output.contains("Project Coverage"));
    }

    #[test]
    fn test_formatter_factory() {
        let options = FormatterOptions::default();
        
        let tree_formatter = FormatterFactory::create(OutputFormat::Tree, options.clone());
        assert_eq!(tree_formatter.format_type(), OutputFormat::Tree);
        
        let json_formatter = FormatterFactory::create(OutputFormat::Json, options.clone());
        assert_eq!(json_formatter.format_type(), OutputFormat::Json);
        
        let mermaid_formatter = FormatterFactory::create(OutputFormat::Mermaid, options);
        assert_eq!(mermaid_formatter.format_type(), OutputFormat::Mermaid);
    }

    #[test]
    fn test_colorization() {
        use super::super::utils::{colorize, coverage_color, format_coverage};
        
        assert_eq!(colorize("test", utils::colors::RED, false), "test");
        
        assert_eq!(coverage_color(90.0), utils::colors::GREEN);
        assert_eq!(coverage_color(70.0), utils::colors::YELLOW);
        assert_eq!(coverage_color(30.0), utils::colors::RED);
        
        let formatted = format_coverage(85.5, false);
        assert_eq!(formatted, " 85.5%");
    }
}