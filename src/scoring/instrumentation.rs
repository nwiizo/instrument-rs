//! Instrumentation scoring system for prioritizing code instrumentation
//!
//! This module implements the scoring algorithm described in section 4 of the specification,
//! which determines which functions and code blocks should be prioritized for instrumentation
//! based on business criticality, error handling, external calls, and complexity.

use std::collections::HashMap;

/// Factors that contribute to the instrumentation score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoringFactor {
    /// Business criticality of the function
    BusinessCriticality,
    /// Presence and quality of error handling
    ErrorHandling,
    /// External system calls (DB, API, etc.)
    ExternalCalls,
    /// Code complexity (cyclomatic, cognitive)
    Complexity,
}

impl ScoringFactor {
    /// Get the default weight for this factor
    pub fn default_weight(&self) -> f64 {
        match self {
            Self::BusinessCriticality => 0.35,
            Self::ErrorHandling => 0.25,
            Self::ExternalCalls => 0.25,
            Self::Complexity => 0.15,
        }
    }

    /// Get a human-readable name for the factor
    pub fn name(&self) -> &'static str {
        match self {
            Self::BusinessCriticality => "Business Criticality",
            Self::ErrorHandling => "Error Handling",
            Self::ExternalCalls => "External Calls",
            Self::Complexity => "Complexity",
        }
    }
}

/// Result of instrumentation scoring for a code element
#[derive(Debug, Clone)]
pub struct InstrumentationScore {
    /// Overall score (0.0 - 100.0)
    pub overall_score: f64,
    
    /// Individual factor scores
    pub factor_scores: HashMap<ScoringFactor, f64>,
    
    /// Priority level based on the score
    pub priority: InstrumentationPriority,
    
    /// Detailed reasoning for the score
    pub reasoning: Vec<String>,
}

/// Priority levels for instrumentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstrumentationPriority {
    /// Critical - must instrument (score >= 80)
    Critical,
    /// High - should instrument (score >= 60)
    High,
    /// Medium - consider instrumenting (score >= 40)
    Medium,
    /// Low - optional instrumentation (score >= 20)
    Low,
    /// Minimal - rarely needs instrumentation (score < 20)
    Minimal,
}

impl InstrumentationPriority {
    /// Determine priority from score
    pub fn from_score(score: f64) -> Self {
        match score as u32 {
            80..=100 => Self::Critical,
            60..=79 => Self::High,
            40..=59 => Self::Medium,
            20..=39 => Self::Low,
            _ => Self::Minimal,
        }
    }

    /// Get a color representation for the priority
    pub fn color(&self) -> &'static str {
        match self {
            Self::Critical => "red",
            Self::High => "orange",
            Self::Medium => "yellow",
            Self::Low => "lightgreen",
            Self::Minimal => "gray",
        }
    }

    /// Get a description of what this priority means
    pub fn description(&self) -> &'static str {
        match self {
            Self::Critical => "Critical instrumentation required - high business impact",
            Self::High => "High priority - significant risk or complexity",
            Self::Medium => "Medium priority - moderate complexity or external dependencies",
            Self::Low => "Low priority - simple logic with minimal external impact",
            Self::Minimal => "Minimal priority - trivial code with no significant impact",
        }
    }
}

/// Configuration for the instrumentation scorer
#[derive(Debug, Clone)]
pub struct ScoringConfig {
    /// Weights for each scoring factor
    pub weights: HashMap<ScoringFactor, f64>,
    
    /// Thresholds for different priority levels
    pub thresholds: PriorityThresholds,
    
    /// Patterns that indicate business criticality
    pub critical_patterns: Vec<String>,
    
    /// Patterns that indicate external calls
    pub external_patterns: Vec<String>,
}

/// Thresholds for priority levels
#[derive(Debug, Clone)]
pub struct PriorityThresholds {
    /// Minimum score for critical priority
    pub critical: f64,
    /// Minimum score for high priority
    pub high: f64,
    /// Minimum score for medium priority
    pub medium: f64,
    /// Minimum score for low priority
    pub low: f64,
}

impl Default for PriorityThresholds {
    fn default() -> Self {
        Self {
            critical: 80.0,
            high: 60.0,
            medium: 40.0,
            low: 20.0,
        }
    }
}

impl Default for ScoringConfig {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(ScoringFactor::BusinessCriticality, 0.35);
        weights.insert(ScoringFactor::ErrorHandling, 0.25);
        weights.insert(ScoringFactor::ExternalCalls, 0.25);
        weights.insert(ScoringFactor::Complexity, 0.15);

        Self {
            weights,
            thresholds: PriorityThresholds::default(),
            critical_patterns: vec![
                "payment".to_string(),
                "auth".to_string(),
                "security".to_string(),
                "transaction".to_string(),
                "order".to_string(),
                "user_data".to_string(),
            ],
            external_patterns: vec![
                "http".to_string(),
                "database".to_string(),
                "api".to_string(),
                "client".to_string(),
                "request".to_string(),
                "fetch".to_string(),
                "query".to_string(),
            ],
        }
    }
}

/// Main instrumentation scorer
pub struct InstrumentationScorer {
    config: ScoringConfig,
}

impl InstrumentationScorer {
    /// Create a new scorer with default configuration
    pub fn new() -> Self {
        Self {
            config: ScoringConfig::default(),
        }
    }

    /// Create a scorer with custom configuration
    pub fn with_config(config: ScoringConfig) -> Self {
        Self { config }
    }

    /// Score a function for instrumentation priority
    ///
    /// # Arguments
    ///
    /// * `function_name` - Name of the function
    /// * `complexity` - Cyclomatic complexity
    /// * `has_error_handling` - Whether the function has error handling
    /// * `external_call_count` - Number of external calls
    /// * `is_public` - Whether the function is public
    ///
    /// # Returns
    ///
    /// An `InstrumentationScore` with the overall score and breakdown
    pub fn score_function(
        &self,
        function_name: &str,
        complexity: u32,
        has_error_handling: bool,
        external_call_count: usize,
        is_public: bool,
    ) -> InstrumentationScore {
        let mut factor_scores = HashMap::new();
        let mut reasoning = Vec::new();

        // Score business criticality
        let criticality_score = self.score_business_criticality(function_name, is_public);
        factor_scores.insert(ScoringFactor::BusinessCriticality, criticality_score);
        if criticality_score > 70.0 {
            reasoning.push(format!(
                "High business criticality detected (score: {:.1})",
                criticality_score
            ));
        }

        // Score error handling
        let error_score = self.score_error_handling(has_error_handling, complexity);
        factor_scores.insert(ScoringFactor::ErrorHandling, error_score);
        if !has_error_handling && complexity > 10 {
            reasoning.push("Complex function without error handling".to_string());
        }

        // Score external calls
        let external_score = self.score_external_calls(external_call_count);
        factor_scores.insert(ScoringFactor::ExternalCalls, external_score);
        if external_call_count > 0 {
            reasoning.push(format!(
                "{} external call(s) detected",
                external_call_count
            ));
        }

        // Score complexity
        let complexity_score = self.score_complexity(complexity);
        factor_scores.insert(ScoringFactor::Complexity, complexity_score);
        if complexity > 20 {
            reasoning.push(format!("High complexity ({})", complexity));
        }

        // Calculate overall score
        let overall_score = self.calculate_overall_score(&factor_scores);
        let priority = InstrumentationPriority::from_score(overall_score);

        InstrumentationScore {
            overall_score,
            factor_scores,
            priority,
            reasoning,
        }
    }

    /// Score business criticality based on function name and visibility
    fn score_business_criticality(&self, function_name: &str, is_public: bool) -> f64 {
        let name_lower = function_name.to_lowercase();
        let mut score: f64 = 0.0;

        // Check against critical patterns
        for pattern in &self.config.critical_patterns {
            if name_lower.contains(pattern) {
                score = score.max(80.0);
                break;
            }
        }

        // Public functions are more critical
        if is_public {
            score += 20.0;
        }

        // Special keywords that indicate criticality
        if name_lower.contains("validate") || name_lower.contains("verify") {
            score += 30.0;
        }

        if name_lower.contains("process") || name_lower.contains("handle") {
            score += 20.0;
        }

        score.min(100.0)
    }

    /// Score error handling presence and quality
    fn score_error_handling(&self, has_error_handling: bool, complexity: u32) -> f64 {
        if has_error_handling {
            // Good error handling gets high score
            70.0 + (complexity as f64).min(30.0)
        } else {
            // No error handling - score based on how critical it is
            match complexity {
                0..=5 => 20.0,   // Simple functions might not need error handling
                6..=10 => 10.0,  // Should probably have error handling
                _ => 0.0,        // Complex functions without error handling are bad
            }
        }
    }

    /// Score based on external calls
    fn score_external_calls(&self, external_call_count: usize) -> f64 {
        match external_call_count {
            0 => 0.0,
            1 => 40.0,
            2 => 60.0,
            3 => 80.0,
            _ => 100.0,
        }
    }

    /// Score based on complexity
    fn score_complexity(&self, complexity: u32) -> f64 {
        match complexity {
            0..=5 => 10.0,
            6..=10 => 30.0,
            11..=20 => 60.0,
            21..=30 => 80.0,
            _ => 100.0,
        }
    }

    /// Calculate overall score from factor scores
    fn calculate_overall_score(&self, factor_scores: &HashMap<ScoringFactor, f64>) -> f64 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (factor, score) in factor_scores {
            let weight = self.config.weights.get(factor).copied().unwrap_or(0.0);
            weighted_sum += score * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Update the weight for a specific factor
    pub fn set_weight(&mut self, factor: ScoringFactor, weight: f64) {
        self.config.weights.insert(factor, weight);
    }

    /// Add a critical pattern
    pub fn add_critical_pattern(&mut self, pattern: String) {
        self.config.critical_patterns.push(pattern);
    }

    /// Add an external pattern
    pub fn add_external_pattern(&mut self, pattern: String) {
        self.config.external_patterns.push(pattern);
    }
}

impl Default for InstrumentationScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_from_score() {
        assert_eq!(InstrumentationPriority::from_score(90.0), InstrumentationPriority::Critical);
        assert_eq!(InstrumentationPriority::from_score(70.0), InstrumentationPriority::High);
        assert_eq!(InstrumentationPriority::from_score(50.0), InstrumentationPriority::Medium);
        assert_eq!(InstrumentationPriority::from_score(30.0), InstrumentationPriority::Low);
        assert_eq!(InstrumentationPriority::from_score(10.0), InstrumentationPriority::Minimal);
    }

    #[test]
    fn test_business_critical_function() {
        let scorer = InstrumentationScorer::new();
        let score = scorer.score_function(
            "process_payment",
            15,  // moderate complexity
            true, // has error handling
            2,    // external calls
            true, // is public
        );

        assert!(score.overall_score > 70.0);
        assert_eq!(score.priority, InstrumentationPriority::Critical);
    }

    #[test]
    fn test_simple_internal_function() {
        let scorer = InstrumentationScorer::new();
        let score = scorer.score_function(
            "format_string",
            3,     // low complexity
            false, // no error handling
            0,     // no external calls
            false, // not public
        );

        assert!(score.overall_score < 30.0);
        assert_eq!(score.priority, InstrumentationPriority::Low);
    }

    #[test]
    fn test_complex_function_without_error_handling() {
        let scorer = InstrumentationScorer::new();
        let score = scorer.score_function(
            "calculate_metrics",
            25,    // high complexity
            false, // no error handling!
            1,     // one external call
            true,  // is public
        );

        // Should have high score due to complexity and lack of error handling
        assert!(score.overall_score > 50.0);
        assert!(!score.reasoning.is_empty());
    }

    #[test]
    fn test_external_heavy_function() {
        let scorer = InstrumentationScorer::new();
        let score = scorer.score_function(
            "fetch_user_data",
            8,    // moderate complexity
            true, // has error handling
            5,    // many external calls
            true, // is public
        );

        // Should have high score due to external calls
        assert!(score.overall_score > 60.0);
        assert!(score.factor_scores[&ScoringFactor::ExternalCalls] > 80.0);
    }
}