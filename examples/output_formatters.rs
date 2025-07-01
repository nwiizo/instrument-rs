//! Example demonstrating the output formatters

use instrument_rs::output::{FormatterFactory, FormatterOptions, OutputFormat};
use instrument_rs::reporting::{
    CoverageSummary, FileCoverage, LineCoverage, BranchCoverage, FunctionCoverage,
    CombinedReport,
};
use instrument_rs::mutation::{MutationSummary, MutationResult, Mutation};
use instrument_rs::config::MutationOperator;
use instrument_rs::ast::{InstrumentableElement, ElementKind};
use instrument_rs::ast;
use std::path::PathBuf;

fn create_sample_coverage() -> CoverageSummary {
    // Create sample file coverage data
    let file1 = FileCoverage {
        file_path: PathBuf::from("src/main.rs"),
        total_lines: 100,
        executable_lines: 80,
        covered_lines: 70,
        line_coverage: vec![
            LineCoverage { line_number: 1, executable: true, hit_count: 5 },
            LineCoverage { line_number: 2, executable: true, hit_count: 5 },
            LineCoverage { line_number: 3, executable: false, hit_count: 0 },
            LineCoverage { line_number: 4, executable: true, hit_count: 0 }, // Uncovered
            LineCoverage { line_number: 5, executable: true, hit_count: 3 },
        ],
        branch_coverage: vec![
            BranchCoverage {
                line_number: 10,
                branch_id: "if_1".to_string(),
                true_count: 5,
                false_count: 3,
            },
            BranchCoverage {
                line_number: 20,
                branch_id: "match_1".to_string(),
                true_count: 0, // Uncovered branch
                false_count: 2,
            },
        ],
        function_coverage: vec![
            FunctionCoverage {
                name: "main".to_string(),
                start_line: 1,
                end_line: 10,
                hit_count: 1,
            },
            FunctionCoverage {
                name: "process_data".to_string(),
                start_line: 12,
                end_line: 25,
                hit_count: 10,
            },
            FunctionCoverage {
                name: "unused_function".to_string(),
                start_line: 27,
                end_line: 35,
                hit_count: 0, // Uncovered function
            },
        ],
    };
    
    let file2 = FileCoverage {
        file_path: PathBuf::from("src/lib.rs"),
        total_lines: 200,
        executable_lines: 150,
        covered_lines: 60, // Low coverage
        line_coverage: vec![],
        branch_coverage: vec![],
        function_coverage: vec![],
    };
    
    let file3 = FileCoverage {
        file_path: PathBuf::from("src/utils/helpers.rs"),
        total_lines: 50,
        executable_lines: 40,
        covered_lines: 38, // High coverage
        line_coverage: vec![],
        branch_coverage: vec![],
        function_coverage: vec![],
    };
    
    CoverageSummary::from_files(vec![file1, file2, file3])
}

fn create_sample_mutations() -> MutationSummary {
    // Create a dummy element for the mutations
    let element = InstrumentableElement {
        id: "elem1".to_string(),
        kind: ElementKind::BinaryOp,
        location: ast::Location::new(15, 1, 15, 20),
        parent_id: None,
        is_test: false,
    };
    
    let mutation1 = Mutation {
        id: "mut1".to_string(),
        operator: MutationOperator::ArithmeticOperatorReplacement,
        element: element.clone(),
        original_code: "a + b".to_string(),
        mutated_code: "a - b".to_string(),
        mutation_tokens: Default::default(),
        description: "Changed + to -".to_string(),
    };
    
    // Create sample mutation results
    let results = vec![
        MutationResult {
            mutation: mutation1,
            killed: true,
            killing_tests: vec!["test_addition".to_string()],
            execution_time_ms: 150,
            timed_out: false,
            compile_error: false,
            error_message: None,
            survived: false,
            file_path: PathBuf::from("src/main.rs"),
            line_number: 15,
            original_code: "a + b".to_string(),
            mutated_code: "a - b".to_string(),
        },
        MutationResult {
            mutation: Mutation {
                id: "mut2".to_string(),
                operator: MutationOperator::ComparisonOperatorReplacement,
                element: InstrumentableElement {
                    id: "elem2".to_string(),
                    kind: ElementKind::BinaryOp,
                    location: ast::Location::new(42, 1, 42, 10),
                    parent_id: None,
                    is_test: false,
                },
                original_code: "x > 0".to_string(),
                mutated_code: "x >= 0".to_string(),
                mutation_tokens: Default::default(),
                description: "Changed > to >=".to_string(),
            },
            killed: false,
            killing_tests: vec![],
            execution_time_ms: 200,
            timed_out: false,
            compile_error: false,
            error_message: None,
            survived: true, // Survived mutation
            file_path: PathBuf::from("src/lib.rs"),
            line_number: 42,
            original_code: "x > 0".to_string(),
            mutated_code: "x >= 0".to_string(),
        },
        MutationResult {
            mutation: Mutation {
                id: "mut3".to_string(),
                operator: MutationOperator::StatementRemoval,
                element: InstrumentableElement {
                    id: "elem3".to_string(),
                    kind: ElementKind::Statement,
                    location: ast::Location::new(8, 1, 8, 15),
                    parent_id: None,
                    is_test: false,
                },
                original_code: "loop { }".to_string(),
                mutated_code: "return".to_string(),
                mutation_tokens: Default::default(),
                description: "Removed infinite loop".to_string(),
            },
            killed: false,
            killing_tests: vec![],
            execution_time_ms: 30000,
            timed_out: true, // Timeout
            compile_error: false,
            error_message: Some("Test execution timed out".to_string()),
            survived: false,
            file_path: PathBuf::from("src/utils/helpers.rs"),
            line_number: 8,
            original_code: "loop { }".to_string(),
            mutated_code: "return".to_string(),
        },
    ];
    
    MutationSummary::from_results(results)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data
    let coverage = create_sample_coverage();
    let mutations = create_sample_mutations();
    let combined_report = CombinedReport::new(
        "instrument-rs-demo".to_string(),
        Some(coverage.clone()),
        Some(mutations.clone()),
    );
    
    // Test different formatter options
    let default_options = FormatterOptions::default();
    let compact_options = FormatterOptions {
        compact: true,
        use_color: true,
        ..Default::default()
    };
    let detailed_options = FormatterOptions {
        include_files: true,
        include_lines: true,
        include_source: false,
        use_color: true,
        sort_by_coverage: true,
        coverage_threshold: Some(80.0),
        ..Default::default()
    };
    
    println!("=== TREE FORMATTER (Default) ===\n");
    let tree_formatter = FormatterFactory::create(OutputFormat::Tree, default_options.clone());
    println!("{}", tree_formatter.format_coverage(&coverage)?);
    
    println!("\n\n=== TREE FORMATTER (Compact) ===\n");
    let tree_formatter_compact = FormatterFactory::create(OutputFormat::Tree, compact_options.clone());
    println!("{}", tree_formatter_compact.format_combined(&combined_report)?);
    
    println!("\n\n=== JSON FORMATTER (Pretty) ===\n");
    let json_formatter = FormatterFactory::create(OutputFormat::Json, default_options.clone());
    println!("{}", json_formatter.format_coverage(&coverage)?);
    
    println!("\n\n=== JSON FORMATTER (Compact) ===\n");
    let json_formatter_compact = FormatterFactory::create(OutputFormat::Json, compact_options);
    println!("{}", json_formatter_compact.format_mutations(&mutations)?);
    
    println!("\n\n=== MERMAID FORMATTER ===\n");
    let mermaid_formatter = FormatterFactory::create(OutputFormat::Mermaid, detailed_options);
    println!("{}", mermaid_formatter.format_combined(&combined_report)?);
    
    // Demonstrate file-specific formatting
    println!("\n\n=== SINGLE FILE COVERAGE ===\n");
    if let Some(file) = coverage.file_coverage.first() {
        println!("{}", tree_formatter.format_file_coverage(file)?);
    }
    
    Ok(())
}