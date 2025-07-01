//! Helper functions for AST analysis

use crate::ast::{AnalysisResult, ComplexityMetrics, ErrorHandlingInfo, FunctionInfo};

/// Filters for selecting functions based on various criteria
pub struct FunctionFilter;

impl FunctionFilter {
    /// Filter functions by complexity threshold
    ///
    /// Returns functions with cyclomatic complexity above the threshold
    ///
    /// # Arguments
    ///
    /// * `functions` - Slice of functions to filter
    /// * `threshold` - Minimum complexity level
    ///
    /// # Returns
    ///
    /// Vector of functions exceeding the complexity threshold
    pub fn by_complexity<'a>(
        functions: &'a [FunctionInfo],
        threshold: usize,
    ) -> Vec<&'a FunctionInfo> {
        functions
            .iter()
            .filter(|f| f.complexity.cyclomatic > threshold)
            .collect()
    }

    /// Filter functions that use unsafe error handling
    ///
    /// Returns functions that use unwrap() or expect() calls
    ///
    /// # Arguments
    ///
    /// * `functions` - Slice of functions to filter
    ///
    /// # Returns
    ///
    /// Vector of functions with unsafe error handling
    pub fn with_unsafe_error_handling<'a>(functions: &'a [FunctionInfo]) -> Vec<&'a FunctionInfo> {
        functions
            .iter()
            .filter(|f| f.error_handling.unwrap_calls > 0 || f.error_handling.expect_calls > 0)
            .collect()
    }

    /// Filter async functions
    ///
    /// # Arguments
    ///
    /// * `functions` - Slice of functions to filter
    ///
    /// # Returns
    ///
    /// Vector of async functions
    pub fn async_functions<'a>(functions: &'a [FunctionInfo]) -> Vec<&'a FunctionInfo> {
        functions.iter().filter(|f| f.is_async).collect()
    }

    /// Filter functions by line count
    ///
    /// Returns functions with more lines than the threshold
    ///
    /// # Arguments
    ///
    /// * `functions` - Slice of functions to filter
    /// * `threshold` - Minimum line count
    ///
    /// # Returns
    ///
    /// Vector of functions exceeding the line count threshold
    pub fn by_line_count<'a>(
        functions: &'a [FunctionInfo],
        threshold: usize,
    ) -> Vec<&'a FunctionInfo> {
        functions
            .iter()
            .filter(|f| f.complexity.lines_of_code > threshold)
            .collect()
    }
}

/// Statistics calculator for analysis results
pub struct AnalysisStats;

impl AnalysisStats {
    /// Calculate average complexity across all functions
    ///
    /// # Arguments
    ///
    /// * `result` - Analysis result to calculate stats for
    ///
    /// # Returns
    ///
    /// Average cyclomatic complexity, or 0.0 if no functions
    pub fn average_complexity(result: &AnalysisResult) -> f64 {
        if result.functions.is_empty() {
            return 0.0;
        }

        let total: usize = result
            .functions
            .iter()
            .map(|f| f.complexity.cyclomatic)
            .sum();

        total as f64 / result.functions.len() as f64
    }

    /// Calculate the percentage of functions with proper error handling
    ///
    /// A function is considered to have proper error handling if it:
    /// - Returns Result/Option and uses ? operator, or
    /// - Uses match/if-let for error handling
    /// - Doesn't use unwrap/expect
    ///
    /// # Arguments
    ///
    /// * `result` - Analysis result to calculate stats for
    ///
    /// # Returns
    ///
    /// Percentage (0-100) of functions with proper error handling
    pub fn error_handling_score(result: &AnalysisResult) -> f64 {
        if result.functions.is_empty() {
            return 100.0;
        }

        let proper_handling_count = result
            .functions
            .iter()
            .filter(|f| {
                let has_unsafe =
                    f.error_handling.unwrap_calls > 0 || f.error_handling.expect_calls > 0;
                let has_safe = f.error_handling.question_mark_ops > 0
                    || f.error_handling.error_matches > 0
                    || f.error_handling.error_if_lets > 0;

                !has_unsafe && (has_safe || f.error_handling.result_returns == 0)
            })
            .count();

        (proper_handling_count as f64 / result.functions.len() as f64) * 100.0
    }

    /// Find the most complex function
    ///
    /// # Arguments
    ///
    /// * `result` - Analysis result to search
    ///
    /// # Returns
    ///
    /// Reference to the most complex function, or None if no functions
    pub fn most_complex_function(result: &AnalysisResult) -> Option<&FunctionInfo> {
        result
            .functions
            .iter()
            .max_by_key(|f| f.complexity.cyclomatic)
    }

    /// Calculate test coverage percentage
    ///
    /// Simple calculation based on number of test functions vs regular functions
    ///
    /// # Arguments
    ///
    /// * `result` - Analysis result to calculate coverage for
    ///
    /// # Returns
    ///
    /// Estimated test coverage percentage (0-100)
    pub fn test_coverage_estimate(result: &AnalysisResult) -> f64 {
        if result.functions.is_empty() {
            return 100.0;
        }

        let test_count = result.test_functions.len();
        let function_count = result.functions.len();

        // Simple heuristic: assume each test covers one function
        let coverage = (test_count as f64 / function_count as f64).min(1.0);
        coverage * 100.0
    }
}

/// Complexity analysis helpers
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    /// Determine the risk level based on complexity metrics
    ///
    /// # Arguments
    ///
    /// * `metrics` - Complexity metrics to analyze
    ///
    /// # Returns
    ///
    /// Risk level as a string: "Low", "Medium", "High", or "Critical"
    pub fn risk_level(metrics: &ComplexityMetrics) -> &'static str {
        match metrics.cyclomatic {
            0..=5 => "Low",
            6..=10 => "Medium",
            11..=20 => "High",
            _ => "Critical",
        }
    }

    /// Check if a function should be refactored based on complexity
    ///
    /// # Arguments
    ///
    /// * `metrics` - Complexity metrics to check
    ///
    /// # Returns
    ///
    /// true if the function should be considered for refactoring
    pub fn needs_refactoring(metrics: &ComplexityMetrics) -> bool {
        metrics.cyclomatic > 10
            || metrics.cognitive > 15
            || metrics.lines_of_code > 50
            || metrics.max_nesting_depth > 4
    }
}

/// Function relationship analyzer
pub struct CallGraphAnalyzer;

impl CallGraphAnalyzer {
    /// Find all functions that call a specific function
    ///
    /// # Arguments
    ///
    /// * `result` - Analysis result containing all functions
    /// * `target_function` - Name of the function to find callers for
    ///
    /// # Returns
    ///
    /// Vector of functions that call the target function
    pub fn find_callers<'a>(
        result: &'a AnalysisResult,
        target_function: &str,
    ) -> Vec<&'a FunctionInfo> {
        result
            .functions
            .iter()
            .filter(|f| f.calls.iter().any(|call| call.callee == target_function))
            .collect()
    }

    /// Find all functions called by a specific function
    ///
    /// # Arguments
    ///
    /// * `function` - Function to analyze
    ///
    /// # Returns
    ///
    /// Vector of unique function names called
    pub fn get_callees(function: &FunctionInfo) -> Vec<String> {
        let mut callees: Vec<String> = function.calls.iter().map(|c| c.callee.clone()).collect();

        callees.sort();
        callees.dedup();
        callees
    }

    /// Check if a function is recursive (calls itself)
    ///
    /// # Arguments
    ///
    /// * `function` - Function to check
    ///
    /// # Returns
    ///
    /// true if the function calls itself
    pub fn is_recursive(function: &FunctionInfo) -> bool {
        function.calls.iter().any(|call| {
            call.callee == function.name || call.callee.ends_with(&format!("::{}", function.name))
        })
    }
}

/// 💡 **Improvement Suggestion**: Add call graph visualization
/// **Time saved**: ~15 minutes understanding code relationships
/// **Implementation**: Generate DOT format output for Graphviz
/// **Benefits**: Visual understanding of function dependencies
///
/// Future enhancement: Add methods to generate call graphs in various
/// formats (DOT, mermaid, JSON) for visualization tools.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Location;

    fn create_test_function(name: &str, complexity: usize, has_unwrap: bool) -> FunctionInfo {
        FunctionInfo {
            id: format!("test_{}", name),
            name: name.to_string(),
            full_path: name.to_string(),
            is_async: false,
            is_unsafe: false,
            is_test: false,
            is_generic: false,
            param_count: 0,
            return_type: None,
            calls: vec![],
            error_handling: ErrorHandlingInfo {
                unwrap_calls: if has_unwrap { 1 } else { 0 },
                ..Default::default()
            },
            complexity: ComplexityMetrics {
                cyclomatic: complexity,
                ..Default::default()
            },
            location: Location::new(1, 1, 10, 1),
        }
    }

    #[test]
    fn test_complexity_filter() {
        let functions = vec![
            create_test_function("simple", 3, false),
            create_test_function("medium", 8, false),
            create_test_function("complex", 15, false),
        ];

        let complex_functions = FunctionFilter::by_complexity(&functions, 7);
        assert_eq!(complex_functions.len(), 2);
        assert_eq!(complex_functions[0].name, "medium");
        assert_eq!(complex_functions[1].name, "complex");
    }

    #[test]
    fn test_unsafe_error_handling_filter() {
        let functions = vec![
            create_test_function("safe", 5, false),
            create_test_function("unsafe", 5, true),
        ];

        let unsafe_functions = FunctionFilter::with_unsafe_error_handling(&functions);
        assert_eq!(unsafe_functions.len(), 1);
        assert_eq!(unsafe_functions[0].name, "unsafe");
    }

    #[test]
    fn test_risk_level() {
        assert_eq!(
            ComplexityAnalyzer::risk_level(&ComplexityMetrics {
                cyclomatic: 3,
                ..Default::default()
            }),
            "Low"
        );

        assert_eq!(
            ComplexityAnalyzer::risk_level(&ComplexityMetrics {
                cyclomatic: 15,
                ..Default::default()
            }),
            "High"
        );

        assert_eq!(
            ComplexityAnalyzer::risk_level(&ComplexityMetrics {
                cyclomatic: 25,
                ..Default::default()
            }),
            "Critical"
        );
    }
}
