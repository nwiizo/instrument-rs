//! Tests for configuration file parsing and validation

use crate::common::create_sample_config_file;
use instrument_rs::{
    Config, ProjectConfig, InstrumentationConfig, MutationConfig, ReportingConfig,
    InstrumentationMode, MutationOperator, ReportFormat,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = Config::default();
    
    // Verify default project settings
    assert_eq!(config.project.root_dir, PathBuf::from("."));
    assert_eq!(config.project.source_dirs, vec![PathBuf::from("src")]);
    assert_eq!(config.project.test_dirs, vec![PathBuf::from("tests")]);
    assert_eq!(config.project.exclude_patterns.len(), 2);
    assert!(config.project.exclude_patterns.contains(&"target/**".to_string()));
    
    // Verify default instrumentation settings
    assert_eq!(config.instrumentation.mode, InstrumentationMode::Coverage);
    assert!(config.instrumentation.preserve_originals);
    assert!(config.instrumentation.parallel);
    assert!(config.instrumentation.threads.is_none());
    
    // Verify default mutation settings
    assert_eq!(config.mutation.operators.len(), 3);
    assert!(config.mutation.operators.contains(&MutationOperator::ArithmeticOperatorReplacement));
    assert_eq!(config.mutation.max_mutations_per_file, Some(100));
    assert_eq!(config.mutation.timeout_seconds, 30);
    
    // Verify default reporting settings
    assert_eq!(config.reporting.formats.len(), 2);
    assert!(config.reporting.formats.contains(&ReportFormat::Html));
    assert!(config.reporting.formats.contains(&ReportFormat::Json));
    assert!(config.reporting.include_source);
    assert_eq!(config.reporting.coverage_threshold, Some(80.0));
    assert_eq!(config.reporting.mutation_threshold, Some(60.0));
}

#[test]
fn test_parse_minimal_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_sample_config_file(temp_dir.path(), "minimal");
    
    let config = Config::from_file(&config_path)
        .expect("Should parse minimal config");
    
    // Verify parsed values
    assert_eq!(config.project.root_dir, PathBuf::from("."));
    assert_eq!(config.project.source_dirs, vec![PathBuf::from("src")]);
    assert_eq!(config.project.test_dirs, vec![PathBuf::from("tests")]);
    assert_eq!(config.instrumentation.mode, InstrumentationMode::Coverage);
    
    // Other values should be defaults
    assert_eq!(config.mutation.operators.len(), 3); // Default operators
    assert_eq!(config.reporting.formats.len(), 2); // Default formats
}

#[test]
fn test_parse_full_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_sample_config_file(temp_dir.path(), "full");
    
    let config = Config::from_file(&config_path)
        .expect("Should parse full config");
    
    // Verify project settings
    assert_eq!(config.project.root_dir, PathBuf::from("."));
    assert_eq!(config.project.source_dirs.len(), 2);
    assert!(config.project.source_dirs.contains(&PathBuf::from("src")));
    assert!(config.project.source_dirs.contains(&PathBuf::from("lib")));
    assert_eq!(config.project.test_dirs.len(), 2);
    assert!(config.project.test_dirs.contains(&PathBuf::from("tests")));
    assert!(config.project.test_dirs.contains(&PathBuf::from("benches")));
    assert_eq!(config.project.exclude_patterns.len(), 3);
    
    // Verify instrumentation settings
    assert_eq!(config.instrumentation.mode, InstrumentationMode::Combined);
    assert!(config.instrumentation.preserve_originals);
    assert_eq!(config.instrumentation.output_dir, PathBuf::from("target/instrumented"));
    assert!(config.instrumentation.parallel);
    assert_eq!(config.instrumentation.threads, Some(4));
    
    // Verify mutation settings
    assert_eq!(config.mutation.operators.len(), 4);
    assert!(config.mutation.operators.contains(&MutationOperator::ArithmeticOperatorReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::StatementDeletion));
    assert_eq!(config.mutation.max_mutations_per_file, Some(50));
    assert_eq!(config.mutation.timeout_seconds, 60);
    assert_eq!(config.mutation.seed, Some(12345));
    
    // Verify reporting settings
    assert_eq!(config.reporting.formats.len(), 3);
    assert!(config.reporting.formats.contains(&ReportFormat::Html));
    assert!(config.reporting.formats.contains(&ReportFormat::Json));
    assert!(config.reporting.formats.contains(&ReportFormat::Console));
    assert_eq!(config.reporting.output_dir, PathBuf::from("target/reports"));
    assert!(config.reporting.include_source);
    assert_eq!(config.reporting.coverage_threshold, Some(85.0));
    assert_eq!(config.reporting.mutation_threshold, Some(70.0));
}

#[test]
fn test_parse_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_sample_config_file(temp_dir.path(), "invalid");
    
    let result = Config::from_file(&config_path);
    assert!(result.is_err(), "Should fail to parse invalid config");
}

#[test]
fn test_save_and_load_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Create a custom config
    let mut config = Config::default();
    config.project.root_dir = PathBuf::from("/test/project");
    config.project.source_dirs = vec![PathBuf::from("src"), PathBuf::from("lib")];
    config.instrumentation.mode = InstrumentationMode::Mutation;
    config.instrumentation.threads = Some(8);
    config.mutation.seed = Some(42);
    config.reporting.formats = vec![ReportFormat::Json, ReportFormat::Xml];
    
    // Save config
    config.save(&config_path)
        .expect("Should save config");
    
    // Load config back
    let loaded_config = Config::from_file(&config_path)
        .expect("Should load saved config");
    
    // Verify all values match
    assert_eq!(loaded_config.project.root_dir, PathBuf::from("/test/project"));
    assert_eq!(loaded_config.project.source_dirs, vec![PathBuf::from("src"), PathBuf::from("lib")]);
    assert_eq!(loaded_config.instrumentation.mode, InstrumentationMode::Mutation);
    assert_eq!(loaded_config.instrumentation.threads, Some(8));
    assert_eq!(loaded_config.mutation.seed, Some(42));
    assert_eq!(loaded_config.reporting.formats, vec![ReportFormat::Json, ReportFormat::Xml]);
}

#[test]
fn test_config_validation() {
    // Test various invalid configurations
    let temp_dir = TempDir::new().unwrap();
    
    // Empty source dirs
    let config_content = r#"
[project]
root_dir = "."
source_dirs = []
test_dirs = ["tests"]

[instrumentation]
mode = "coverage"
"#;
    let config_path = temp_dir.path().join("empty_sources.toml");
    fs::write(&config_path, config_content).unwrap();
    let config = Config::from_file(&config_path).unwrap();
    assert!(config.project.source_dirs.is_empty(), "Should allow empty source dirs");
    
    // Invalid instrumentation mode
    let config_content = r#"
[project]
root_dir = "."
source_dirs = ["src"]

[instrumentation]
mode = "invalid_mode"
"#;
    let config_path = temp_dir.path().join("invalid_mode.toml");
    fs::write(&config_path, config_content).unwrap();
    let result = Config::from_file(&config_path);
    assert!(result.is_err(), "Should reject invalid instrumentation mode");
    
    // Invalid mutation operator
    let config_content = r#"
[project]
root_dir = "."
source_dirs = ["src"]

[instrumentation]
mode = "mutation"

[mutation]
operators = ["invalid_operator"]
"#;
    let config_path = temp_dir.path().join("invalid_operator.toml");
    fs::write(&config_path, config_content).unwrap();
    let result = Config::from_file(&config_path);
    assert!(result.is_err(), "Should reject invalid mutation operator");
    
    // Invalid report format
    let config_content = r#"
[project]
root_dir = "."
source_dirs = ["src"]

[instrumentation]
mode = "coverage"

[reporting]
formats = ["invalid_format"]
"#;
    let config_path = temp_dir.path().join("invalid_format.toml");
    fs::write(&config_path, config_content).unwrap();
    let result = Config::from_file(&config_path);
    assert!(result.is_err(), "Should reject invalid report format");
}

#[test]
fn test_config_with_environment_variables() {
    let temp_dir = TempDir::new().unwrap();
    
    // Config with environment variable placeholders
    let config_content = r#"
[project]
root_dir = "."
source_dirs = ["src"]
test_dirs = ["tests"]
target_dir = "target"

[instrumentation]
mode = "coverage"
output_dir = "target/instrumented"
parallel = true

[reporting]
output_dir = "target/reports"
formats = ["json", "html"]
"#;
    
    let config_path = temp_dir.path().join("env_config.toml");
    fs::write(&config_path, config_content).unwrap();
    
    let config = Config::from_file(&config_path)
        .expect("Should parse config with paths");
    
    // Verify paths are parsed correctly
    assert_eq!(config.project.target_dir, PathBuf::from("target"));
    assert_eq!(config.instrumentation.output_dir, PathBuf::from("target/instrumented"));
    assert_eq!(config.reporting.output_dir, PathBuf::from("target/reports"));
}

#[test]
fn test_config_merge_with_defaults() {
    let temp_dir = TempDir::new().unwrap();
    
    // Partial config that should be merged with defaults
    let config_content = r#"
[project]
root_dir = "/custom/path"

[instrumentation]
mode = "mutation"
"#;
    
    let config_path = temp_dir.path().join("partial_config.toml");
    fs::write(&config_path, config_content).unwrap();
    
    let config = Config::from_file(&config_path)
        .expect("Should parse partial config");
    
    // Custom values should be applied
    assert_eq!(config.project.root_dir, PathBuf::from("/custom/path"));
    assert_eq!(config.instrumentation.mode, InstrumentationMode::Mutation);
    
    // Default values should be preserved
    assert_eq!(config.project.source_dirs, vec![PathBuf::from("src")]);
    assert_eq!(config.project.test_dirs, vec![PathBuf::from("tests")]);
    assert!(config.instrumentation.preserve_originals);
    assert_eq!(config.mutation.timeout_seconds, 30);
    assert_eq!(config.reporting.formats.len(), 2);
}

#[test]
fn test_all_mutation_operators() {
    let temp_dir = TempDir::new().unwrap();
    
    // Config with all mutation operators
    let config_content = r#"
[project]
root_dir = "."
source_dirs = ["src"]

[instrumentation]
mode = "mutation"

[mutation]
operators = [
    "arithmetic_operator_replacement",
    "comparison_operator_replacement",
    "logical_operator_replacement",
    "assignment_operator_replacement",
    "statement_deletion",
    "constant_replacement",
    "return_value_replacement",
    "function_call_replacement",
    "loop_condition_modification"
]
"#;
    
    let config_path = temp_dir.path().join("all_operators.toml");
    fs::write(&config_path, config_content).unwrap();
    
    let config = Config::from_file(&config_path)
        .expect("Should parse config with all operators");
    
    assert_eq!(config.mutation.operators.len(), 9);
    assert!(config.mutation.operators.contains(&MutationOperator::ArithmeticOperatorReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::ComparisonOperatorReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::LogicalOperatorReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::AssignmentOperatorReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::StatementDeletion));
    assert!(config.mutation.operators.contains(&MutationOperator::ConstantReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::ReturnValueReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::FunctionCallReplacement));
    assert!(config.mutation.operators.contains(&MutationOperator::LoopConditionModification));
}

#[test]
fn test_all_report_formats() {
    let temp_dir = TempDir::new().unwrap();
    
    // Config with all report formats
    let config_content = r#"
[project]
root_dir = "."
source_dirs = ["src"]

[instrumentation]
mode = "coverage"

[reporting]
formats = ["json", "html", "markdown", "xml", "lcov", "console"]
"#;
    
    let config_path = temp_dir.path().join("all_formats.toml");
    fs::write(&config_path, config_content).unwrap();
    
    let config = Config::from_file(&config_path)
        .expect("Should parse config with all formats");
    
    assert_eq!(config.reporting.formats.len(), 6);
    assert!(config.reporting.formats.contains(&ReportFormat::Json));
    assert!(config.reporting.formats.contains(&ReportFormat::Html));
    assert!(config.reporting.formats.contains(&ReportFormat::Markdown));
    assert!(config.reporting.formats.contains(&ReportFormat::Xml));
    assert!(config.reporting.formats.contains(&ReportFormat::Lcov));
    assert!(config.reporting.formats.contains(&ReportFormat::Console));
}