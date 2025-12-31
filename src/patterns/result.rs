//! Pattern matching results and category determination.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Categories of code patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    /// Unit test (standard #[test] functions)
    UnitTest,

    /// Integration test (tests in tests/ directory)
    IntegrationTest,

    /// Property-based test (QuickCheck, Proptest)
    PropertyTest,

    /// Benchmark test
    Benchmark,

    /// Fuzzing test
    Fuzz,

    /// Mock or stub implementation
    Mock,

    /// Test utility or helper function
    TestUtility,

    /// Example code
    Example,

    /// Database operation (query, insert, update, delete)
    Database,

    /// HTTP client call (reqwest, hyper client, etc.)
    HttpClient,

    /// External service call (API, gRPC, etc.)
    ExternalService,

    /// Cache operation (get, set, invalidate)
    Cache,

    /// Message queue operation (publish, consume)
    MessageQueue,

    /// Error handling code
    ErrorHandling,

    /// Authentication/authorization code
    Auth,

    /// Business logic
    BusinessLogic,

    /// Unknown/uncategorized code
    Unknown,
}

impl Category {
    /// Returns a human-readable name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            Self::UnitTest => "Unit Test",
            Self::IntegrationTest => "Integration Test",
            Self::PropertyTest => "Property Test",
            Self::Benchmark => "Benchmark",
            Self::Fuzz => "Fuzz Test",
            Self::Mock => "Mock/Stub",
            Self::TestUtility => "Test Utility",
            Self::Example => "Example",
            Self::Database => "Database",
            Self::HttpClient => "HTTP Client",
            Self::ExternalService => "External Service",
            Self::Cache => "Cache",
            Self::MessageQueue => "Message Queue",
            Self::ErrorHandling => "Error Handling",
            Self::Auth => "Authentication",
            Self::BusinessLogic => "Business Logic",
            Self::Unknown => "Unknown",
        }
    }

    /// Returns a short identifier for the category.
    pub fn id(&self) -> &'static str {
        match self {
            Self::UnitTest => "unit",
            Self::IntegrationTest => "integration",
            Self::PropertyTest => "property",
            Self::Benchmark => "bench",
            Self::Fuzz => "fuzz",
            Self::Mock => "mock",
            Self::TestUtility => "utility",
            Self::Example => "example",
            Self::Database => "database",
            Self::HttpClient => "http_client",
            Self::ExternalService => "external_service",
            Self::Cache => "cache",
            Self::MessageQueue => "message_queue",
            Self::ErrorHandling => "error_handling",
            Self::Auth => "auth",
            Self::BusinessLogic => "business_logic",
            Self::Unknown => "unknown",
        }
    }
}

/// Details about a single pattern match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDetail {
    /// The pattern that matched
    pub pattern: String,

    /// The weight of this pattern
    pub weight: f64,

    /// The location in the source where it matched (line, column)
    pub location: Option<(usize, usize)>,

    /// The actual text that matched
    pub matched_text: String,

    /// Additional context about the match
    pub context: Option<String>,
}

/// Result of pattern matching analysis.
///
/// Contains the confidence scores, matched patterns, and determined category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// File where the pattern was matched
    pub file: std::path::PathBuf,

    /// Function name where the pattern was matched
    pub function_name: String,

    /// Line number where the pattern was matched
    pub line: usize,

    /// Overall confidence that this is test-related code (0.0 to 1.0)
    pub confidence: f64,

    /// Confidence scores for specific categories
    pub category_scores: HashMap<Category, f64>,

    /// The most likely category based on pattern matches
    pub category: Category,

    /// Detailed information about matched patterns
    pub matches: Vec<MatchDetail>,

    /// Detected testing frameworks
    pub frameworks: Vec<String>,

    /// Additional metadata about the match
    pub metadata: HashMap<String, String>,
}

impl MatchResult {
    /// Creates a new empty match result.
    pub fn new() -> Self {
        Self {
            file: std::path::PathBuf::new(),
            function_name: String::new(),
            line: 0,
            confidence: 0.0,
            category_scores: HashMap::new(),
            category: Category::Unknown,
            matches: Vec::new(),
            frameworks: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new match result with location information.
    pub fn with_location(file: std::path::PathBuf, function_name: String, line: usize) -> Self {
        Self {
            file,
            function_name,
            line,
            confidence: 0.0,
            category_scores: HashMap::new(),
            category: Category::Unknown,
            matches: Vec::new(),
            frameworks: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a pattern match to the result.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern that matched
    /// * `weight` - The weight of the pattern
    /// * `matched_text` - The actual text that matched
    pub fn add_match(&mut self, pattern: &str, weight: f64, matched_text: &str) {
        self.matches.push(MatchDetail {
            pattern: pattern.to_string(),
            weight,
            location: None,
            matched_text: matched_text.to_string(),
            context: None,
        });
    }

    /// Adds a pattern match with location information.
    pub fn add_match_with_location(
        &mut self,
        pattern: &str,
        weight: f64,
        matched_text: &str,
        line: usize,
        column: usize,
    ) {
        self.matches.push(MatchDetail {
            pattern: pattern.to_string(),
            weight,
            location: Some((line, column)),
            matched_text: matched_text.to_string(),
            context: None,
        });
    }

    /// Calculates the final confidence scores and determines the category.
    ///
    /// This method should be called after all pattern matches have been added.
    pub fn finalize(&mut self) {
        // Calculate total weight
        let total_weight: f64 = self.matches.iter().map(|m| m.weight).sum();

        // Normalize confidence to 0.0-1.0 range
        self.confidence = (total_weight / self.matches.len().max(1) as f64).min(1.0);

        // Calculate category scores based on patterns
        self.calculate_category_scores();

        // Determine the most likely category
        self.category = self
            .category_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(cat, _)| *cat)
            .unwrap_or(Category::Unknown);
    }

    /// Calculates confidence scores for each category based on matched patterns.
    fn calculate_category_scores(&mut self) {
        for detail in &self.matches {
            // Unit test indicators
            if detail.pattern.contains("#[test]")
                || detail.pattern.contains("test_")
                || detail.pattern.contains("mod tests")
            {
                *self
                    .category_scores
                    .entry(Category::UnitTest)
                    .or_insert(0.0) += detail.weight;
            }

            // Property test indicators
            if detail.pattern.contains("quickcheck")
                || detail.pattern.contains("proptest")
                || detail.pattern.contains("#[quickcheck]")
                || detail.pattern.contains("#[proptest]")
            {
                *self
                    .category_scores
                    .entry(Category::PropertyTest)
                    .or_insert(0.0) += detail.weight;
            }

            // Benchmark indicators
            if detail.pattern.contains("bench")
                || detail.pattern.contains("#[bench]")
                || detail.pattern.contains("criterion")
            {
                *self
                    .category_scores
                    .entry(Category::Benchmark)
                    .or_insert(0.0) += detail.weight;
            }

            // Mock indicators
            if detail.pattern.contains("mock")
                || detail.pattern.contains("stub")
                || detail.pattern.contains("automock")
            {
                *self.category_scores.entry(Category::Mock).or_insert(0.0) += detail.weight;
            }

            // Test utility indicators
            if detail.pattern.contains("helper")
                || detail.pattern.contains("fixture")
                || detail.pattern.contains("setup")
                || detail.pattern.contains("teardown")
            {
                *self
                    .category_scores
                    .entry(Category::TestUtility)
                    .or_insert(0.0) += detail.weight;
            }

            // Example indicators
            if detail.pattern.contains("example") || detail.pattern.contains("doc(") {
                *self.category_scores.entry(Category::Example).or_insert(0.0) += detail.weight;
            }

            // Fuzz test indicators
            if detail.pattern.contains("fuzz") || detail.pattern.contains("arbitrary") {
                *self.category_scores.entry(Category::Fuzz).or_insert(0.0) += detail.weight;
            }
        }

        // Normalize category scores
        let max_score = self.category_scores.values().cloned().fold(0.0, f64::max);
        if max_score > 0.0 {
            for score in self.category_scores.values_mut() {
                *score /= max_score;
            }
        }
    }

    /// Returns true if the confidence is above the given threshold.
    pub fn is_confident(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }

    /// Returns the top N most likely categories with their scores.
    pub fn top_categories(&self, n: usize) -> Vec<(Category, f64)> {
        let mut categories: Vec<_> = self
            .category_scores
            .iter()
            .map(|(cat, score)| (*cat, *score))
            .collect();
        categories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        categories.truncate(n);
        categories
    }
}

impl Default for MatchResult {
    fn default() -> Self {
        Self::new()
    }
}
