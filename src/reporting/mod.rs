//! Report generation for coverage and mutation testing results

use crate::config::ReportFormat;
use crate::mutation::MutationSummary;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod console;
pub mod html;
pub mod json;

/// Coverage information for a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    /// Path to the source file
    pub file_path: PathBuf,

    /// Total number of lines
    pub total_lines: usize,

    /// Number of executable lines
    pub executable_lines: usize,

    /// Number of covered lines
    pub covered_lines: usize,

    /// Line-by-line coverage data
    pub line_coverage: Vec<LineCoverage>,

    /// Branch coverage information
    pub branch_coverage: Vec<BranchCoverage>,

    /// Function coverage information
    pub function_coverage: Vec<FunctionCoverage>,
}

/// Coverage data for a single line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineCoverage {
    /// Line number (1-indexed)
    pub line_number: usize,

    /// Whether this line is executable
    pub executable: bool,

    /// Number of times this line was hit
    pub hit_count: u64,
}

/// Branch coverage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchCoverage {
    /// Line number where the branch starts
    pub line_number: usize,

    /// Branch ID
    pub branch_id: String,

    /// Number of times the true branch was taken
    pub true_count: u64,

    /// Number of times the false branch was taken
    pub false_count: u64,
}

/// Function coverage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCoverage {
    /// Function name
    pub name: String,

    /// Starting line number
    pub start_line: usize,

    /// Ending line number
    pub end_line: usize,

    /// Number of times the function was called
    pub hit_count: u64,
}

/// Overall coverage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    /// Total number of files
    pub total_files: usize,

    /// Total lines across all files
    pub total_lines: usize,

    /// Total executable lines
    pub total_executable: usize,

    /// Total covered lines
    pub total_covered: usize,

    /// Line coverage percentage
    pub line_coverage_percent: f64,

    /// Branch coverage percentage
    pub branch_coverage_percent: f64,

    /// Function coverage percentage
    pub function_coverage_percent: f64,

    /// Per-file coverage data
    pub file_coverage: Vec<FileCoverage>,
}

/// Combined report with both coverage and mutation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedReport {
    /// Timestamp when the report was generated
    pub timestamp: u64,

    /// Project name
    pub project_name: String,

    /// Coverage results
    pub coverage: Option<CoverageSummary>,

    /// Mutation testing results
    pub mutations: Option<MutationSummary>,

    /// Overall quality score (0-100)
    pub quality_score: f64,
}

/// Trait for report generators
pub trait ReportGenerator {
    /// Get the format this generator produces
    fn format(&self) -> ReportFormat;

    /// Generate a coverage report
    fn generate_coverage_report(
        &self,
        summary: &CoverageSummary,
        output_path: &Path,
    ) -> crate::Result<()>;

    /// Generate a mutation report
    fn generate_mutation_report(
        &self,
        summary: &MutationSummary,
        output_path: &Path,
    ) -> crate::Result<()>;

    /// Generate a combined report
    fn generate_combined_report(
        &self,
        report: &CombinedReport,
        output_path: &Path,
    ) -> crate::Result<()>;
}

impl FileCoverage {
    /// Calculate line coverage percentage for this file
    pub fn line_coverage_percent(&self) -> f64 {
        if self.executable_lines == 0 {
            100.0
        } else {
            (self.covered_lines as f64 / self.executable_lines as f64) * 100.0
        }
    }

    /// Get uncovered lines
    pub fn uncovered_lines(&self) -> Vec<usize> {
        self.line_coverage
            .iter()
            .filter(|lc| lc.executable && lc.hit_count == 0)
            .map(|lc| lc.line_number)
            .collect()
    }
}

impl CoverageSummary {
    /// Create a new coverage summary from file coverage data
    pub fn from_files(file_coverage: Vec<FileCoverage>) -> Self {
        let total_files = file_coverage.len();
        let total_lines: usize = file_coverage.iter().map(|f| f.total_lines).sum();
        let total_executable: usize = file_coverage.iter().map(|f| f.executable_lines).sum();
        let total_covered: usize = file_coverage.iter().map(|f| f.covered_lines).sum();

        let line_coverage_percent = if total_executable > 0 {
            (total_covered as f64 / total_executable as f64) * 100.0
        } else {
            100.0
        };

        // Calculate branch coverage
        let total_branches: usize = file_coverage
            .iter()
            .map(|f| f.branch_coverage.len() * 2) // Each branch has true/false
            .sum();
        let covered_branches: usize = file_coverage
            .iter()
            .map(|f| {
                f.branch_coverage
                    .iter()
                    .map(|b| {
                        (if b.true_count > 0 { 1 } else { 0 })
                            + (if b.false_count > 0 { 1 } else { 0 })
                    })
                    .sum::<usize>()
            })
            .sum();

        let branch_coverage_percent = if total_branches > 0 {
            (covered_branches as f64 / total_branches as f64) * 100.0
        } else {
            100.0
        };

        // Calculate function coverage
        let total_functions: usize = file_coverage
            .iter()
            .map(|f| f.function_coverage.len())
            .sum();
        let covered_functions: usize = file_coverage
            .iter()
            .map(|f| {
                f.function_coverage
                    .iter()
                    .filter(|fc| fc.hit_count > 0)
                    .count()
            })
            .sum();

        let function_coverage_percent = if total_functions > 0 {
            (covered_functions as f64 / total_functions as f64) * 100.0
        } else {
            100.0
        };

        Self {
            total_files,
            total_lines,
            total_executable,
            total_covered,
            line_coverage_percent,
            branch_coverage_percent,
            function_coverage_percent,
            file_coverage,
        }
    }

    /// Find files with coverage below a threshold
    pub fn files_below_threshold(&self, threshold: f64) -> Vec<&FileCoverage> {
        self.file_coverage
            .iter()
            .filter(|f| f.line_coverage_percent() < threshold)
            .collect()
    }
}

impl CombinedReport {
    /// Create a new combined report
    pub fn new(
        project_name: String,
        coverage: Option<CoverageSummary>,
        mutations: Option<MutationSummary>,
    ) -> Self {
        let quality_score = Self::calculate_quality_score(&coverage, &mutations);

        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            project_name,
            coverage,
            mutations,
            quality_score,
        }
    }

    /// Calculate overall quality score based on coverage and mutation results
    fn calculate_quality_score(
        coverage: &Option<CoverageSummary>,
        mutations: &Option<MutationSummary>,
    ) -> f64 {
        let mut score = 0.0;
        let mut weight = 0.0;

        if let Some(cov) = coverage {
            // Line coverage contributes 40%
            score += cov.line_coverage_percent * 0.4;
            // Branch coverage contributes 20%
            score += cov.branch_coverage_percent * 0.2;
            // Function coverage contributes 10%
            score += cov.function_coverage_percent * 0.1;
            weight += 0.7;
        }

        if let Some(mut_summary) = mutations {
            // Mutation score contributes 30%
            score += mut_summary.mutation_score * 0.3;
            weight += 0.3;
        }

        if weight > 0.0 {
            score / weight
        } else {
            0.0
        }
    }
}

/// Factory for creating report generators
pub struct ReportGeneratorFactory;

impl ReportGeneratorFactory {
    /// Create a report generator for the specified format
    pub fn create(format: ReportFormat) -> Box<dyn ReportGenerator> {
        match format {
            ReportFormat::Html => Box::new(html::HtmlReportGenerator::new()),
            ReportFormat::Json => Box::new(json::JsonReportGenerator::new()),
            ReportFormat::Console => Box::new(console::ConsoleReportGenerator::new()),
            _ => Box::new(json::JsonReportGenerator::new()), // Default to JSON for unimplemented formats
        }
    }
}
