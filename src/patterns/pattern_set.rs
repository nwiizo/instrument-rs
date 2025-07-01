//! Pattern definitions and pattern set management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single pattern with its associated weight.
///
/// Patterns can be simple string matches or more complex regex patterns.
/// Each pattern has a weight that contributes to the overall confidence score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// The pattern string (can be a simple string or regex pattern)
    pub pattern: String,

    /// Weight of this pattern (0.0 to 1.0)
    pub weight: f64,

    /// Whether this is a regex pattern or simple string match
    pub is_regex: bool,

    /// Optional description of what this pattern matches
    pub description: Option<String>,
}

impl Pattern {
    /// Creates a new simple string pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern string to match
    /// * `weight` - The weight of this pattern
    ///
    /// # Examples
    ///
    /// ```
    /// use instrument_rs::patterns::Pattern;
    ///
    /// let pattern = Pattern::simple("test_", 0.8);
    /// assert_eq!(pattern.weight, 0.8);
    /// ```
    pub fn simple(pattern: impl Into<String>, weight: f64) -> Self {
        Self {
            pattern: pattern.into(),
            weight,
            is_regex: false,
            description: None,
        }
    }

    /// Creates a new regex pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The regex pattern string
    /// * `weight` - The weight of this pattern
    ///
    /// # Examples
    ///
    /// ```
    /// use instrument_rs::patterns::Pattern;
    ///
    /// let pattern = Pattern::regex(r"^test_\w+", 0.9);
    /// assert!(pattern.is_regex);
    /// ```
    pub fn regex(pattern: impl Into<String>, weight: f64) -> Self {
        Self {
            pattern: pattern.into(),
            weight,
            is_regex: true,
            description: None,
        }
    }

    /// Sets the description for this pattern.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Collection of patterns organized by category.
///
/// The PatternSet holds all the patterns used for identifying test-related code,
/// organized into different categories like function names, attributes, framework patterns, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternSet {
    /// Function name patterns (e.g., "test_", "should_", "it_")
    pub function_names: Vec<Pattern>,

    /// Attribute patterns (e.g., "#[test]", "#[cfg(test)]")
    pub attributes: Vec<Pattern>,

    /// Framework-specific patterns (e.g., "describe!", "it!", "context!")
    pub framework_patterns: HashMap<String, Vec<Pattern>>,

    /// Assertion patterns (e.g., "assert!", "assert_eq!", "expect")
    pub assertions: Vec<Pattern>,

    /// Error handling patterns (e.g., "should_panic", "unwrap", "expect")
    pub error_handling: Vec<Pattern>,

    /// Module patterns (e.g., "mod tests", "mod test")
    pub modules: Vec<Pattern>,

    /// Import patterns (e.g., "use super::*", test framework imports)
    pub imports: Vec<Pattern>,
}

impl PatternSet {
    /// Creates a new empty pattern set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a pattern set with default test patterns.
    ///
    /// This includes common Rust testing patterns and popular framework patterns.
    pub fn with_defaults() -> Self {
        let mut pattern_set = Self::new();

        // Function name patterns
        pattern_set.function_names = vec![
            Pattern::simple("test_", 0.9).with_description("Functions starting with 'test_'"),
            Pattern::simple("should_", 0.8).with_description("BDD-style 'should_' functions"),
            Pattern::simple("it_", 0.8).with_description("BDD-style 'it_' functions"),
            Pattern::simple("when_", 0.7).with_description("BDD-style 'when_' functions"),
            Pattern::simple("given_", 0.7).with_description("BDD-style 'given_' functions"),
            Pattern::regex(r"_test$", 0.8).with_description("Functions ending with '_test'"),
            Pattern::regex(r"_should_\w+", 0.7)
                .with_description("Functions with '_should_' pattern"),
        ];

        // Attribute patterns
        pattern_set.attributes = vec![
            Pattern::simple("#[test]", 1.0).with_description("Standard Rust test attribute"),
            Pattern::simple("#[cfg(test)]", 0.9).with_description("Test configuration attribute"),
            Pattern::simple("#[tokio::test]", 1.0).with_description("Tokio async test attribute"),
            Pattern::simple("#[async_std::test]", 1.0).with_description("async-std test attribute"),
            Pattern::simple("#[quickcheck]", 0.9).with_description("QuickCheck property test"),
            Pattern::simple("#[proptest]", 0.9).with_description("Proptest property test"),
            Pattern::simple("#[rstest]", 0.9).with_description("rstest parameterized test"),
            Pattern::simple("#[test_case", 0.9).with_description("test-case crate attribute"),
            Pattern::simple("#[should_panic", 0.8).with_description("Panic test attribute"),
            Pattern::simple("#[ignore]", 0.6).with_description("Ignored test attribute"),
        ];

        // Framework patterns
        pattern_set.framework_patterns.insert(
            "mockall".to_string(),
            vec![
                Pattern::simple("mock!", 0.8).with_description("Mockall mock macro"),
                Pattern::simple("automock", 0.8).with_description("Mockall automock attribute"),
                Pattern::simple("predicate::", 0.7).with_description("Mockall predicate usage"),
            ],
        );

        pattern_set.framework_patterns.insert(
            "spectral".to_string(),
            vec![
                Pattern::simple("assert_that!", 0.9).with_description("Spectral assertion macro"),
                Pattern::simple("asserting!", 0.8).with_description("Spectral asserting block"),
                Pattern::simple("is_equal_to", 0.7).with_description("Spectral matcher"),
                Pattern::simple("contains", 0.6).with_description("Spectral contains matcher"),
            ],
        );

        pattern_set.framework_patterns.insert(
            "cucumber".to_string(),
            vec![
                Pattern::simple("given!", 0.8).with_description("Cucumber given step"),
                Pattern::simple("when!", 0.8).with_description("Cucumber when step"),
                Pattern::simple("then!", 0.8).with_description("Cucumber then step"),
                Pattern::simple("#[given", 0.8).with_description("Cucumber given attribute"),
                Pattern::simple("#[when", 0.8).with_description("Cucumber when attribute"),
                Pattern::simple("#[then", 0.8).with_description("Cucumber then attribute"),
            ],
        );

        // Assertion patterns
        pattern_set.assertions = vec![
            Pattern::simple("assert!", 0.9).with_description("Basic assertion"),
            Pattern::simple("assert_eq!", 0.9).with_description("Equality assertion"),
            Pattern::simple("assert_ne!", 0.9).with_description("Inequality assertion"),
            Pattern::simple("debug_assert!", 0.7).with_description("Debug assertion"),
            Pattern::simple("assert_matches!", 0.8).with_description("Pattern matching assertion"),
            Pattern::simple("assert_approx_eq!", 0.8)
                .with_description("Approximate equality assertion"),
            Pattern::regex(r"\.expect\(", 0.6).with_description("Expect method call"),
            Pattern::regex(r"\.unwrap\(", 0.5).with_description("Unwrap method call"),
        ];

        // Error handling patterns
        pattern_set.error_handling = vec![
            Pattern::simple("should_panic", 0.9).with_description("Panic expectation"),
            Pattern::simple("catch_unwind", 0.7).with_description("Panic catching"),
            Pattern::simple("Result<(), ", 0.6).with_description("Result error type"),
            Pattern::regex(r"Err\(.+\)", 0.6).with_description("Error variant"),
            Pattern::simple(".is_err()", 0.7).with_description("Error checking"),
            Pattern::simple(".is_ok()", 0.6).with_description("Success checking"),
        ];

        // Module patterns
        pattern_set.modules = vec![
            Pattern::simple("mod tests", 1.0).with_description("Standard test module"),
            Pattern::simple("mod test", 0.9).with_description("Alternative test module"),
            Pattern::regex(r"mod \w+_tests", 0.8).with_description("Named test module"),
            Pattern::simple("#[cfg(test)]", 0.9).with_description("Test configuration"),
        ];

        // Import patterns
        pattern_set.imports = vec![
            Pattern::simple("use super::*", 0.7).with_description("Parent module import"),
            Pattern::simple("use crate::", 0.5).with_description("Crate-level import"),
            Pattern::regex(r"use .+test", 0.6).with_description("Test-related import"),
            Pattern::simple("use mockall::", 0.8).with_description("Mockall framework import"),
            Pattern::simple("use spectral::", 0.8).with_description("Spectral framework import"),
            Pattern::simple("use proptest::", 0.8).with_description("Proptest framework import"),
            Pattern::simple("use quickcheck::", 0.8)
                .with_description("QuickCheck framework import"),
        ];

        pattern_set
    }

    /// Adds a custom pattern to a specific category.
    ///
    /// # Arguments
    ///
    /// * `category` - The category to add the pattern to
    /// * `pattern` - The pattern to add
    pub fn add_pattern(&mut self, category: &str, pattern: Pattern) {
        match category {
            "function_names" => self.function_names.push(pattern),
            "attributes" => self.attributes.push(pattern),
            "assertions" => self.assertions.push(pattern),
            "error_handling" => self.error_handling.push(pattern),
            "modules" => self.modules.push(pattern),
            "imports" => self.imports.push(pattern),
            framework => {
                self.framework_patterns
                    .entry(framework.to_string())
                    .or_default()
                    .push(pattern);
            }
        }
    }

    /// Merges another pattern set into this one.
    ///
    /// This is useful for combining default patterns with custom patterns.
    pub fn merge(&mut self, other: PatternSet) {
        self.function_names.extend(other.function_names);
        self.attributes.extend(other.attributes);
        self.assertions.extend(other.assertions);
        self.error_handling.extend(other.error_handling);
        self.modules.extend(other.modules);
        self.imports.extend(other.imports);

        for (framework, patterns) in other.framework_patterns {
            self.framework_patterns
                .entry(framework)
                .or_default()
                .extend(patterns);
        }
    }
}
