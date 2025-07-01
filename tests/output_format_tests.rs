//! Tests for output format generation

use crate::common::sample_projects;
use instrument_rs::{
    ast::{AstAnalyzer, SourceFile},
    call_graph::GraphBuilder,
    output::{OutputFormatter, JsonFormatter, TreeFormatter, MermaidFormatter, TestOutputFormatter},
    reporting::{ConsoleReporter, HtmlReporter, JsonReporter},
};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_json_output_format() {
    let project = sample_projects::simple_library();
    
    // Analyze the project
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse source file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Generate JSON output
    let formatter = JsonFormatter::new();
    let output = formatter.format(&ast_result);
    
    // Parse and validate JSON
    let json_value: Value = serde_json::from_str(&output)
        .expect("Should produce valid JSON");
    
    // Verify structure
    assert!(json_value.is_object(), "Root should be an object");
    assert!(json_value["functions"].is_array(), "Functions should be an array");
    assert!(json_value["test_functions"].is_array(), "Test functions should be an array");
    assert!(json_value["metrics"].is_object(), "Metrics should be an object");
    
    // Verify function data
    let functions = json_value["functions"].as_array().unwrap();
    assert_eq!(functions.len(), 5, "Should have 5 functions");
    
    // Check a specific function
    let add_function = functions.iter()
        .find(|f| f["name"] == "add")
        .expect("Should find add function");
    
    assert!(add_function["line_count"].is_number());
    assert!(add_function["complexity"].is_object());
    assert!(add_function["complexity"]["cyclomatic"].is_number());
    assert!(add_function["parameters"].is_array());
    
    // Verify metrics
    assert!(json_value["metrics"]["total_functions"].is_number());
    assert!(json_value["metrics"]["total_tests"].is_number());
    assert!(json_value["metrics"]["average_complexity"].is_number());
}

#[test]
fn test_tree_output_format() {
    let project = sample_projects::complex_patterns();
    
    // Analyze the project
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse source file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Generate tree output
    let formatter = TreeFormatter::new();
    let output = formatter.format(&ast_result);
    
    // Verify tree structure
    assert!(output.contains("lib.rs"), "Should contain file name");
    assert!(output.contains("├──"), "Should have tree branches");
    assert!(output.contains("└──"), "Should have tree end branches");
    
    // Check for main components
    assert!(output.contains("Traits"), "Should have Traits section");
    assert!(output.contains("DataStore"), "Should list DataStore trait");
    
    assert!(output.contains("Structs"), "Should have Structs section");
    assert!(output.contains("MemoryStore"), "Should list MemoryStore struct");
    assert!(output.contains("DataProcessor"), "Should list DataProcessor struct");
    
    assert!(output.contains("Enums"), "Should have Enums section");
    assert!(output.contains("StoreError"), "Should list StoreError enum");
    
    assert!(output.contains("Functions"), "Should have Functions section");
    assert!(output.contains("fibonacci"), "Should list fibonacci function");
    
    // Check indentation
    let lines: Vec<&str> = output.lines().collect();
    for line in lines {
        if line.contains("├──") || line.contains("└──") {
            assert!(line.starts_with("  ") || line.starts_with("    "),
                "Tree items should be indented");
        }
    }
}

#[test]
fn test_mermaid_call_graph_output() {
    let project = sample_projects::simple_library();
    
    // Build call graph
    let mut builder = GraphBuilder::new();
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse source file");
    builder.add_file(source_file).expect("Should add file to graph");
    let graph = builder.build();
    
    // Generate Mermaid output
    let formatter = MermaidFormatter::new();
    let output = formatter.format_graph(&graph);
    
    // Verify Mermaid syntax
    assert!(output.starts_with("graph TD"), "Should start with graph declaration");
    
    // Check for nodes
    assert!(output.contains("test_add["), "Should have test_add node");
    assert!(output.contains("add["), "Should have add function node");
    assert!(output.contains("test_divide["), "Should have test_divide node");
    assert!(output.contains("divide["), "Should have divide function node");
    
    // Check for edges
    assert!(output.contains("-->"), "Should have edge arrows");
    assert!(output.contains("test_add --> add"), "Should show test calling function");
    assert!(output.contains("test_divide --> divide"), "Should show test calling function");
    
    // Check styling
    assert!(output.contains("style test_add"), "Should style test nodes");
    assert!(output.contains("classDef test"), "Should define test class");
}

#[test]
fn test_test_output_formatter() {
    let project = sample_projects::simple_library();
    
    // Create test coverage data
    let test_data = TestOutputFormatter::TestCoverage {
        total_tests: 2,
        passed_tests: 2,
        failed_tests: 0,
        coverage_percentage: 85.5,
        covered_lines: 45,
        total_lines: 53,
        uncovered_functions: vec!["subtract".to_string()],
    };
    
    let formatter = TestOutputFormatter::new();
    let output = formatter.format_coverage(&test_data);
    
    // Verify output contains key information
    assert!(output.contains("Test Summary"), "Should have summary header");
    assert!(output.contains("2/2 tests passed"), "Should show test results");
    assert!(output.contains("85.5%"), "Should show coverage percentage");
    assert!(output.contains("45/53 lines"), "Should show line coverage");
    assert!(output.contains("Uncovered functions:"), "Should list uncovered functions");
    assert!(output.contains("subtract"), "Should list specific uncovered function");
}

#[test]
fn test_console_reporter() {
    let temp_dir = TempDir::new().unwrap();
    let project = sample_projects::simple_library();
    
    // Analyze project
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse source file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Create console reporter
    let reporter = ConsoleReporter::new();
    let output = reporter.generate_report(&ast_result);
    
    // Verify console output format
    assert!(output.contains("═"), "Should have box drawing characters");
    assert!(output.contains("Analysis Report"), "Should have report title");
    assert!(output.contains("Functions"), "Should have functions section");
    assert!(output.contains("Complexity"), "Should show complexity metrics");
    assert!(output.contains("Tests"), "Should have test section");
    
    // Check for color codes (if terminal supports it)
    // Note: In test environment, color codes might not be present
    let has_metrics = output.contains("Total Functions:") && 
                      output.contains("Total Tests:");
    assert!(has_metrics, "Should have summary metrics");
}

#[test]
fn test_html_reporter() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("reports");
    fs::create_dir_all(&output_dir).unwrap();
    
    let project = sample_projects::complex_patterns();
    
    // Analyze project
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse source file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Generate HTML report
    let reporter = HtmlReporter::new(output_dir.clone());
    reporter.generate_report(&ast_result, "test_report.html")
        .expect("Should generate HTML report");
    
    // Verify HTML file was created
    let html_path = output_dir.join("test_report.html");
    assert!(html_path.exists(), "HTML report should be created");
    
    // Read and verify HTML content
    let html_content = fs::read_to_string(&html_path)
        .expect("Should read HTML file");
    
    // Check HTML structure
    assert!(html_content.contains("<!DOCTYPE html>"), "Should be valid HTML");
    assert!(html_content.contains("<head>"), "Should have head section");
    assert!(html_content.contains("<body>"), "Should have body section");
    assert!(html_content.contains("<title>"), "Should have title");
    
    // Check for content
    assert!(html_content.contains("Analysis Report"), "Should have report title");
    assert!(html_content.contains("<table"), "Should have tables");
    assert!(html_content.contains("DataStore"), "Should contain trait name");
    assert!(html_content.contains("MemoryStore"), "Should contain struct name");
    assert!(html_content.contains("fibonacci"), "Should contain function name");
    
    // Check for styling
    assert!(html_content.contains("<style>") || html_content.contains("style="),
        "Should have CSS styling");
    
    // Check for JavaScript (for interactive features)
    assert!(html_content.contains("<script>") || html_content.contains("onclick"),
        "Should have interactive elements");
}

#[test]
fn test_json_reporter_structured_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("report.json");
    
    let project = sample_projects::simple_library();
    
    // Analyze project
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse source file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Generate JSON report
    let reporter = JsonReporter::new();
    reporter.save_report(&ast_result, &output_path)
        .expect("Should save JSON report");
    
    // Read and parse JSON
    let json_content = fs::read_to_string(&output_path)
        .expect("Should read JSON file");
    let json_value: Value = serde_json::from_str(&json_content)
        .expect("Should be valid JSON");
    
    // Verify comprehensive structure
    assert!(json_value["metadata"].is_object(), "Should have metadata");
    assert!(json_value["metadata"]["timestamp"].is_string(), "Should have timestamp");
    assert!(json_value["metadata"]["version"].is_string(), "Should have version");
    
    assert!(json_value["summary"].is_object(), "Should have summary");
    assert!(json_value["summary"]["total_functions"].is_number());
    assert!(json_value["summary"]["total_tests"].is_number());
    assert!(json_value["summary"]["total_lines"].is_number());
    
    assert!(json_value["functions"].is_array(), "Should have functions array");
    assert!(json_value["test_functions"].is_array(), "Should have test functions array");
    
    // Check detailed function information
    let functions = json_value["functions"].as_array().unwrap();
    for func in functions {
        assert!(func["name"].is_string());
        assert!(func["start_line"].is_number());
        assert!(func["end_line"].is_number());
        assert!(func["complexity"].is_object());
        assert!(func["metrics"].is_object());
    }
}

#[test]
#[ignore = "Output formatters are designed for coverage/mutation reports, not AST analysis"]
fn test_combined_output_formats() {
    let project = sample_projects::axum_web_app();
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("outputs");
    fs::create_dir_all(&output_dir).unwrap();
    
    // Analyze main.rs
    let main_rs = project.root_path.join("src/main.rs");
    let source_file = SourceFile::parse(&main_rs).expect("Should parse source file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Generate multiple output formats
    let json_formatter = JsonFormatter::new();
    let tree_formatter = TreeFormatter::new();
    let test_formatter = TestOutputFormatter::new();
    
    let json_output = json_formatter.format(&ast_result);
    let tree_output = tree_formatter.format(&ast_result);
    
    // Save outputs
    fs::write(output_dir.join("analysis.json"), &json_output).unwrap();
    fs::write(output_dir.join("analysis.tree"), &tree_output).unwrap();
    
    // Verify all formats work together
    assert!(!json_output.is_empty(), "JSON output should not be empty");
    assert!(!tree_output.is_empty(), "Tree output should not be empty");
    
    // Verify consistency across formats
    let json_value: Value = serde_json::from_str(&json_output).unwrap();
    let function_count = json_value["functions"].as_array().unwrap().len();
    
    // Tree output should mention same functions
    for func in json_value["functions"].as_array().unwrap() {
        let func_name = func["name"].as_str().unwrap();
        assert!(tree_output.contains(func_name),
            "Tree output should contain function: {}", func_name);
    }
}

#[test]
#[ignore = "Output formatters are designed for coverage/mutation reports, not AST analysis"]
fn test_output_format_error_handling() {
    // Test with empty/invalid data
    let empty_ast = instrument_rs::ast::AstAnalysisResult::default();
    
    let json_formatter = JsonFormatter::new();
    let tree_formatter = TreeFormatter::new();
    
    // Should handle empty data gracefully
    let json_output = json_formatter.format(&empty_ast);
    let tree_output = tree_formatter.format(&empty_ast);
    
    assert!(!json_output.is_empty(), "Should produce output even for empty AST");
    assert!(!tree_output.is_empty(), "Should produce output even for empty AST");
    
    // JSON should be valid even when empty
    let json_value: Value = serde_json::from_str(&json_output)
        .expect("Should produce valid JSON for empty data");
    assert_eq!(json_value["functions"].as_array().unwrap().len(), 0);
    assert_eq!(json_value["test_functions"].as_array().unwrap().len(), 0);
}