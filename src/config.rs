//! Configuration structures for instrument-rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The main configuration structure for instrument-rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project settings
    pub project: ProjectConfig,

    /// Instrumentation settings
    pub instrumentation: InstrumentationConfig,

    /// Mutation testing settings
    pub mutation: MutationConfig,

    /// Reporting settings
    pub reporting: ReportingConfig,
}

/// Project-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Root directory of the project
    pub root_dir: PathBuf,

    /// Source directories to analyze
    pub source_dirs: Vec<PathBuf>,

    /// Test directories
    pub test_dirs: Vec<PathBuf>,

    /// Patterns to exclude from analysis
    pub exclude_patterns: Vec<String>,

    /// Target directory for build artifacts
    pub target_dir: PathBuf,
}

/// Instrumentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationConfig {
    /// Type of instrumentation to apply
    pub mode: InstrumentationMode,

    /// Whether to preserve original files
    pub preserve_originals: bool,

    /// Output directory for instrumented files
    pub output_dir: PathBuf,

    /// Enable parallel processing
    pub parallel: bool,

    /// Number of threads to use (None = use all available)
    pub threads: Option<usize>,
}

/// Available instrumentation modes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InstrumentationMode {
    /// Coverage tracking only
    Coverage,
    /// Mutation testing only
    Mutation,
    /// Both coverage and mutation
    Combined,
}

/// Mutation testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationConfig {
    /// Mutation operators to apply
    pub operators: Vec<MutationOperator>,

    /// Maximum number of mutations per file
    pub max_mutations_per_file: Option<usize>,

    /// Timeout for each mutation test run (in seconds)
    pub timeout_seconds: u64,

    /// Random seed for deterministic mutation selection
    pub seed: Option<u64>,
}

/// Available mutation operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MutationOperator {
    /// Replace arithmetic operators (+, -, *, /, %)
    ArithmeticOperatorReplacement,
    /// Replace comparison operators (<, >, <=, >=, ==, !=)
    ComparisonOperatorReplacement,
    /// Replace logical operators (&&, ||, !)
    LogicalOperatorReplacement,
    /// Replace assignment operators (+=, -=, etc.)
    AssignmentOperatorReplacement,
    /// Remove statements
    StatementDeletion,
    /// Replace constants with different values
    ConstantReplacement,
    /// Replace return values
    ReturnValueReplacement,
    /// Replace function calls
    FunctionCallReplacement,
    /// Modify loop conditions
    LoopConditionModification,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Output formats for reports
    pub formats: Vec<ReportFormat>,

    /// Output directory for reports
    pub output_dir: PathBuf,

    /// Include source code in reports
    pub include_source: bool,

    /// Minimum coverage threshold (0-100)
    pub coverage_threshold: Option<f64>,

    /// Minimum mutation score threshold (0-100)
    pub mutation_threshold: Option<f64>,
}

/// Available report formats
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    /// JSON format
    Json,
    /// HTML format
    Html,
    /// Markdown format
    Markdown,
    /// XML format (Cobertura-compatible)
    Xml,
    /// LCOV format
    Lcov,
    /// Console output
    Console,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                root_dir: PathBuf::from("."),
                source_dirs: vec![PathBuf::from("src")],
                test_dirs: vec![PathBuf::from("tests")],
                exclude_patterns: vec!["target/**".to_string(), "**/*.rs.bk".to_string()],
                target_dir: PathBuf::from("target"),
            },
            instrumentation: InstrumentationConfig {
                mode: InstrumentationMode::Coverage,
                preserve_originals: true,
                output_dir: PathBuf::from("target/instrument-rs"),
                parallel: true,
                threads: None,
            },
            mutation: MutationConfig {
                operators: vec![
                    MutationOperator::ArithmeticOperatorReplacement,
                    MutationOperator::ComparisonOperatorReplacement,
                    MutationOperator::LogicalOperatorReplacement,
                ],
                max_mutations_per_file: Some(100),
                timeout_seconds: 30,
                seed: None,
            },
            reporting: ReportingConfig {
                formats: vec![ReportFormat::Html, ReportFormat::Json],
                output_dir: PathBuf::from("target/instrument-rs/reports"),
                include_source: true,
                coverage_threshold: Some(80.0),
                mutation_threshold: Some(60.0),
            },
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to save the configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
