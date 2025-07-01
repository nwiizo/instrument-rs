//! Scoring algorithms for code quality metrics

pub mod analyzer;
pub mod instrumentation;

use crate::mutation::MutationSummary;
use crate::reporting::{CoverageSummary, FileCoverage};
use std::path::PathBuf;

pub use analyzer::{
    FunctionAnalyzer, FunctionMetadata, InstrumentationAnalyzer, InstrumentationReport,
    InstrumentationSummary,
};
pub use instrumentation::{
    InstrumentationPriority, InstrumentationScore, InstrumentationScorer, ScoringConfig,
    ScoringFactor,
};

/// Quality score calculation
#[derive(Debug, Clone)]
pub struct QualityScore {
    /// Overall score (0-100)
    pub overall: f64,

    /// Coverage component score
    pub coverage_score: f64,

    /// Mutation component score
    pub mutation_score: f64,

    /// Complexity component score
    pub complexity_score: f64,

    /// Risk level based on the score
    pub risk_level: RiskLevel,
}

/// Risk levels for code quality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Very low risk (score >= 90)
    VeryLow,
    /// Low risk (score >= 80)
    Low,
    /// Medium risk (score >= 70)
    Medium,
    /// High risk (score >= 60)
    High,
    /// Very high risk (score < 60)
    VeryHigh,
}

/// Weights for different scoring components
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    /// Weight for line coverage (default: 0.3)
    pub line_coverage: f64,

    /// Weight for branch coverage (default: 0.2)
    pub branch_coverage: f64,

    /// Weight for function coverage (default: 0.1)
    pub function_coverage: f64,

    /// Weight for mutation score (default: 0.3)
    pub mutation_score: f64,

    /// Weight for complexity (default: 0.1)
    pub complexity: f64,
}

/// Complexity metrics for a file
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity
    pub cyclomatic: u32,

    /// Cognitive complexity
    pub cognitive: u32,

    /// Number of functions
    pub function_count: usize,

    /// Average function length
    pub avg_function_length: f64,

    /// Maximum function length
    pub max_function_length: usize,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            line_coverage: 0.3,
            branch_coverage: 0.2,
            function_coverage: 0.1,
            mutation_score: 0.3,
            complexity: 0.1,
        }
    }
}

impl ScoringWeights {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> bool {
        let sum = self.line_coverage
            + self.branch_coverage
            + self.function_coverage
            + self.mutation_score
            + self.complexity;

        (sum - 1.0).abs() < 0.001
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.line_coverage
            + self.branch_coverage
            + self.function_coverage
            + self.mutation_score
            + self.complexity;

        if sum > 0.0 {
            self.line_coverage /= sum;
            self.branch_coverage /= sum;
            self.function_coverage /= sum;
            self.mutation_score /= sum;
            self.complexity /= sum;
        }
    }
}

/// Calculator for quality scores
pub struct ScoreCalculator {
    weights: ScoringWeights,
}

impl ScoreCalculator {
    /// Create a new score calculator with default weights
    pub fn new() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }

    /// Create a score calculator with custom weights
    pub fn with_weights(mut weights: ScoringWeights) -> Self {
        weights.normalize();
        Self { weights }
    }

    /// Calculate quality score from coverage and mutation data
    pub fn calculate(
        &self,
        coverage: Option<&CoverageSummary>,
        mutations: Option<&MutationSummary>,
        complexity: Option<&ComplexityMetrics>,
    ) -> QualityScore {
        let mut coverage_score = 0.0;
        let mut mutation_score = 0.0;
        let mut complexity_score = 0.0;

        // Calculate coverage score
        if let Some(cov) = coverage {
            coverage_score = self.weights.line_coverage * cov.line_coverage_percent
                + self.weights.branch_coverage * cov.branch_coverage_percent
                + self.weights.function_coverage * cov.function_coverage_percent;
        }

        // Calculate mutation score
        if let Some(mut_summary) = mutations {
            mutation_score = self.weights.mutation_score * mut_summary.mutation_score;
        }

        // Calculate complexity score (inverted - lower complexity is better)
        if let Some(comp) = complexity {
            let complexity_penalty = self.calculate_complexity_penalty(comp);
            complexity_score = self.weights.complexity * (100.0 - complexity_penalty);
        }

        let overall = coverage_score + mutation_score + complexity_score;
        let risk_level = RiskLevel::from_score(overall);

        QualityScore {
            overall,
            coverage_score,
            mutation_score,
            complexity_score,
            risk_level,
        }
    }

    /// Calculate complexity penalty (0-100)
    fn calculate_complexity_penalty(&self, metrics: &ComplexityMetrics) -> f64 {
        // Simple penalty calculation based on thresholds
        let cyclomatic_penalty = match metrics.cyclomatic {
            0..=5 => 0.0,
            6..=10 => 10.0,
            11..=20 => 30.0,
            21..=50 => 50.0,
            _ => 70.0,
        };

        let cognitive_penalty = match metrics.cognitive {
            0..=7 => 0.0,
            8..=15 => 15.0,
            16..=25 => 35.0,
            26..=50 => 55.0,
            _ => 75.0,
        };

        let length_penalty = if metrics.avg_function_length > 50.0 {
            20.0
        } else if metrics.avg_function_length > 30.0 {
            10.0
        } else {
            0.0
        };

        // Weight the penalties
        let total_penalty: f64 =
            cyclomatic_penalty * 0.4 + cognitive_penalty * 0.4 + length_penalty * 0.2;
        total_penalty.min(100.0)
    }
}

impl RiskLevel {
    /// Determine risk level from score
    pub fn from_score(score: f64) -> Self {
        match score as u32 {
            90..=100 => Self::VeryLow,
            80..=89 => Self::Low,
            70..=79 => Self::Medium,
            60..=69 => Self::High,
            _ => Self::VeryHigh,
        }
    }

    /// Get color representation for the risk level
    pub fn color(&self) -> &'static str {
        match self {
            Self::VeryLow => "green",
            Self::Low => "lightgreen",
            Self::Medium => "yellow",
            Self::High => "orange",
            Self::VeryHigh => "red",
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::VeryLow => "✅",
            Self::Low => "🟢",
            Self::Medium => "🟡",
            Self::High => "🟠",
            Self::VeryHigh => "🔴",
        }
    }
}

/// File-level quality assessment
#[derive(Debug, Clone)]
pub struct FileQuality {
    /// File path
    pub path: PathBuf,

    /// Quality score for this file
    pub score: QualityScore,

    /// Specific issues found
    pub issues: Vec<QualityIssue>,
}

/// Specific quality issues
#[derive(Debug, Clone)]
pub struct QualityIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue category
    pub category: IssueCategory,

    /// Description of the issue
    pub description: String,

    /// Location in the file (if applicable)
    pub location: Option<(usize, usize)>, // (line, column)
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

/// Categories of quality issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCategory {
    /// Coverage-related issue
    Coverage,
    /// Mutation survival issue
    MutationSurvival,
    /// Complexity issue
    Complexity,
    /// Test quality issue
    TestQuality,
}

impl Default for ScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyze file quality and identify specific issues
pub fn analyze_file_quality(
    file_coverage: &FileCoverage,
    complexity: Option<&ComplexityMetrics>,
) -> FileQuality {
    let mut issues = Vec::new();

    // Check coverage issues
    let coverage_percent = file_coverage.line_coverage_percent();
    if coverage_percent < 80.0 {
        issues.push(QualityIssue {
            severity: if coverage_percent < 60.0 {
                IssueSeverity::Error
            } else {
                IssueSeverity::Warning
            },
            category: IssueCategory::Coverage,
            description: format!("Low line coverage: {:.1}%", coverage_percent),
            location: None,
        });
    }

    // Check for uncovered lines
    let uncovered_lines = file_coverage.uncovered_lines();
    if uncovered_lines.len() > 10 {
        issues.push(QualityIssue {
            severity: IssueSeverity::Warning,
            category: IssueCategory::Coverage,
            description: format!("{} uncovered lines", uncovered_lines.len()),
            location: None,
        });
    }

    // Check complexity issues
    if let Some(comp) = complexity {
        if comp.cyclomatic > 20 {
            issues.push(QualityIssue {
                severity: if comp.cyclomatic > 50 {
                    IssueSeverity::Error
                } else {
                    IssueSeverity::Warning
                },
                category: IssueCategory::Complexity,
                description: format!("High cyclomatic complexity: {}", comp.cyclomatic),
                location: None,
            });
        }

        if comp.max_function_length > 100 {
            issues.push(QualityIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::Complexity,
                description: format!("Function too long: {} lines", comp.max_function_length),
                location: None,
            });
        }
    }

    // Calculate score for this file
    let file_summary = CoverageSummary::from_files(vec![file_coverage.clone()]);
    let calculator = ScoreCalculator::new();
    let score = calculator.calculate(Some(&file_summary), None, complexity);

    FileQuality {
        path: file_coverage.file_path.clone(),
        score,
        issues,
    }
}
