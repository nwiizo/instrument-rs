//! Tests for output formatters

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::AnalysisResult;
    use crate::AnalysisStats;
    use crate::ProjectDependencies;
    use crate::call_graph::CallGraph;
    use crate::detector::{
        Endpoint, InstrumentationKind, InstrumentationPoint, Location, Priority,
    };

    fn create_test_analysis_result() -> AnalysisResult {
        let endpoints = vec![Endpoint {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            handler: "get_users".to_string(),
            location: Location {
                file: std::path::PathBuf::from("src/handlers.rs"),
                line: 10,
                column: 1,
                function_name: "get_users".to_string(),
            },
            framework: "axum".to_string(),
        }];

        let points = vec![InstrumentationPoint {
            location: Location {
                file: std::path::PathBuf::from("src/handlers.rs"),
                line: 10,
                column: 1,
                function_name: "get_users".to_string(),
            },
            kind: InstrumentationKind::Endpoint,
            priority: Priority::Critical,
            reason: "GET endpoint handler".to_string(),
            suggested_span_name: "get_api_users".to_string(),
            suggested_fields: vec![],
        }];

        AnalysisResult {
            endpoints,
            call_graph: CallGraph::new(),
            patterns: vec![],
            points,
            existing_instrumentation: vec![],
            gaps: vec![],
            rule_violations: vec![],
            dependencies: ProjectDependencies::default(),
            stats: AnalysisStats {
                total_files: 5,
                total_functions: 20,
                total_lines: 500,
                endpoints_count: 1,
                instrumentation_points: 1,
                existing_count: 0,
                gaps_count: 0,
                rule_violations_count: 0,
            },
        }
    }

    #[test]
    fn test_tree_formatter() {
        let formatter = TreeFormatter::new(FormatterOptions::default());
        let result = create_test_analysis_result();

        let output = formatter.format(&result).unwrap();
        assert!(output.contains("Analysis Results"));
        assert!(output.contains("get_users"));
        assert!(output.contains("/api/users"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new(FormatterOptions::default());
        let result = create_test_analysis_result();

        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["stats"]["total_files"].as_u64().is_some());
        assert!(parsed["endpoints"].is_array());
        assert!(parsed["instrumentation_points"].is_array());
    }

    #[test]
    fn test_mermaid_formatter() {
        let formatter = MermaidFormatter::new(FormatterOptions::default());
        let result = create_test_analysis_result();

        let output = formatter.format(&result).unwrap();
        assert!(output.contains("graph TD"));
        assert!(output.contains("GET /api/users"));
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
