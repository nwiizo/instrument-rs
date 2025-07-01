//! Framework detection and integration for web frameworks and testing tools
//!
//! This module provides automatic detection of web frameworks (Axum, Actix-web,
//! Rocket, Tonic) and testing frameworks used in Rust projects. It identifies
//! framework-specific patterns, endpoints, and handlers.
//!
//! # Supported Web Frameworks
//!
//! - **Axum**: Modern web framework with tower middleware
//! - **Actix-web**: Actor-based web framework
//! - **Rocket**: Web framework with code generation
//! - **Tonic**: gRPC framework for Rust
//!
//! # Supported Test Frameworks
//!
//! - Built-in Rust test (`#[test]`)
//! - Tokio test (`#[tokio::test]`)
//! - async-std test (`#[async_std::test]`)
//! - Proptest (property-based testing)
//! - Quickcheck (property-based testing)
//! - Criterion (benchmarking)
//!
//! # Example
//!
//! ```no_run
//! use instrument_rs::framework::detector::FrameworkDetector;
//! use std::path::Path;
//!
//! let detector = FrameworkDetector::new();
//! 
//! // Detect web framework from Cargo.toml
//! if let Some(framework) = detector.detect_from_cargo(Path::new("Cargo.toml"))? {
//!     println!("Detected web framework: {:?}", framework);
//! }
//!
//! // Detect from source code
//! let source = std::fs::read_to_string("src/main.rs")?;
//! if let Some(framework) = detector.detect_from_source(&source) {
//!     println!("Detected framework in source: {:?}", framework);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod adapter;
pub mod detector;
pub mod web;

/// Supported test frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestFramework {
    /// Built-in Rust test framework
    BuiltinTest,
    /// Tokio test framework
    Tokio,
    /// async-std test framework
    AsyncStd,
    /// Proptest property-based testing
    Proptest,
    /// Quickcheck property-based testing
    Quickcheck,
    /// Criterion benchmarking framework
    Criterion,
    /// Custom/unknown framework
    Custom,
}

/// Information about detected test framework
#[derive(Debug, Clone)]
pub struct FrameworkInfo {
    /// The detected framework
    pub framework: TestFramework,

    /// Version of the framework (if detected)
    pub version: Option<String>,

    /// Test attribute patterns used by this framework
    pub test_attributes: Vec<String>,

    /// Benchmark attribute patterns
    pub bench_attributes: Vec<String>,

    /// Common imports/uses for this framework
    pub common_imports: Vec<String>,
}

/// Test runner configuration for a specific framework
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Framework being configured
    pub framework: TestFramework,

    /// Command to run tests
    pub test_command: Vec<String>,

    /// Environment variables to set
    pub env_vars: Vec<(String, String)>,

    /// Additional arguments for coverage mode
    pub coverage_args: Vec<String>,

    /// Additional arguments for mutation testing
    pub mutation_args: Vec<String>,

    /// Timeout multiplier for this framework
    pub timeout_multiplier: f64,
}

impl TestFramework {
    /// Get the display name of the framework
    pub fn name(&self) -> &'static str {
        match self {
            Self::BuiltinTest => "Built-in Test",
            Self::Tokio => "Tokio",
            Self::AsyncStd => "async-std",
            Self::Proptest => "Proptest",
            Self::Quickcheck => "Quickcheck",
            Self::Criterion => "Criterion",
            Self::Custom => "Custom",
        }
    }

    /// Check if this framework supports async tests
    pub fn supports_async(&self) -> bool {
        matches!(self, Self::Tokio | Self::AsyncStd)
    }

    /// Check if this is a property-based testing framework
    pub fn is_property_based(&self) -> bool {
        matches!(self, Self::Proptest | Self::Quickcheck)
    }

    /// Check if this is a benchmarking framework
    pub fn is_benchmark(&self) -> bool {
        matches!(self, Self::Criterion)
    }
}

impl Default for FrameworkInfo {
    fn default() -> Self {
        Self {
            framework: TestFramework::BuiltinTest,
            version: None,
            test_attributes: vec!["test".to_string(), "cfg(test)".to_string()],
            bench_attributes: vec!["bench".to_string()],
            common_imports: vec![],
        }
    }
}

impl FrameworkInfo {
    /// Create framework info for Tokio
    pub fn tokio() -> Self {
        Self {
            framework: TestFramework::Tokio,
            version: None,
            test_attributes: vec!["tokio::test".to_string(), "test".to_string()],
            bench_attributes: vec![],
            common_imports: vec!["tokio::test".to_string(), "tokio::runtime".to_string()],
        }
    }

    /// Create framework info for async-std
    pub fn async_std() -> Self {
        Self {
            framework: TestFramework::AsyncStd,
            version: None,
            test_attributes: vec!["async_std::test".to_string(), "test".to_string()],
            bench_attributes: vec![],
            common_imports: vec!["async_std::test".to_string()],
        }
    }

    /// Create framework info for Proptest
    pub fn proptest() -> Self {
        Self {
            framework: TestFramework::Proptest,
            version: None,
            test_attributes: vec!["proptest".to_string(), "test".to_string()],
            bench_attributes: vec![],
            common_imports: vec!["proptest::prelude::*".to_string()],
        }
    }
}

/// Trait for framework-specific test execution
pub trait TestRunner {
    /// Get the framework this runner supports
    fn framework(&self) -> TestFramework;

    /// Build the command to run tests
    fn build_test_command(&self, config: &RunnerConfig) -> Vec<String>;

    /// Parse test results from output
    fn parse_results(&self, output: &str) -> crate::Result<TestResults>;

    /// Check if a test passed based on exit code and output
    fn is_success(&self, exit_code: i32, output: &str) -> bool;
}

/// Results from running tests
#[derive(Debug, Clone)]
pub struct TestResults {
    /// Number of tests that passed
    pub passed: usize,

    /// Number of tests that failed
    pub failed: usize,

    /// Number of tests that were ignored
    pub ignored: usize,

    /// Total execution time in milliseconds
    pub duration_ms: u64,

    /// Individual test results
    pub tests: Vec<TestResult>,
}

/// Result for a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name
    pub name: String,

    /// Module path to the test
    pub module_path: Vec<String>,

    /// Test outcome
    pub outcome: TestOutcome,

    /// Execution time in milliseconds
    pub duration_ms: Option<u64>,

    /// Error message (if failed)
    pub error_message: Option<String>,
}

/// Possible test outcomes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestOutcome {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test was ignored/skipped
    Ignored,
    /// Test panicked
    Panicked,
    /// Test timed out
    TimedOut,
}
