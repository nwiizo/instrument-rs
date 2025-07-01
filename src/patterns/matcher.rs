//! Pattern matching engine for identifying test-related code patterns.

use super::{MatchResult, Pattern, PatternSet};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use syn::{File, Item, ItemFn, ItemMod};

/// Pattern matcher for analyzing Rust source code.
///
/// The PatternMatcher uses a weighted scoring system to identify test-related
/// patterns in source code and determine the category of test code.
pub struct PatternMatcher {
    /// The pattern set to use for matching
    pattern_set: PatternSet,

    /// Compiled regex patterns for efficient matching
    compiled_patterns: HashMap<String, Regex>,

    /// Confidence threshold for considering code as test-related
    confidence_threshold: f64,
}

impl PatternMatcher {
    /// Creates a new pattern matcher with default patterns.
    ///
    /// # Examples
    ///
    /// ```
    /// use instrument_rs::patterns::PatternMatcher;
    ///
    /// let matcher = PatternMatcher::new();
    /// ```
    pub fn new() -> Self {
        Self::with_pattern_set(PatternSet::with_defaults())
    }

    /// Creates a new pattern matcher with a custom pattern set.
    ///
    /// # Arguments
    ///
    /// * `pattern_set` - The pattern set to use for matching
    ///
    /// # Examples
    ///
    /// ```
    /// use instrument_rs::patterns::{PatternMatcher, PatternSet};
    ///
    /// let pattern_set = PatternSet::with_defaults();
    /// let matcher = PatternMatcher::with_pattern_set(pattern_set);
    /// ```
    pub fn with_pattern_set(pattern_set: PatternSet) -> Self {
        let mut matcher = Self {
            pattern_set,
            compiled_patterns: HashMap::new(),
            confidence_threshold: 0.5,
        };
        matcher.compile_patterns();
        matcher
    }

    /// Sets the confidence threshold for considering code as test-related.
    ///
    /// # Arguments
    ///
    /// * `threshold` - The confidence threshold (0.0 to 1.0)
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Compiles regex patterns for efficient matching.
    fn compile_patterns(&mut self) {
        let mut compile_pattern = |patterns: &[Pattern]| {
            for pattern in patterns {
                if pattern.is_regex && !self.compiled_patterns.contains_key(&pattern.pattern) {
                    if let Ok(regex) = Regex::new(&pattern.pattern) {
                        self.compiled_patterns
                            .insert(pattern.pattern.clone(), regex);
                    }
                }
            }
        };

        compile_pattern(&self.pattern_set.function_names);
        compile_pattern(&self.pattern_set.attributes);
        compile_pattern(&self.pattern_set.assertions);
        compile_pattern(&self.pattern_set.error_handling);
        compile_pattern(&self.pattern_set.modules);
        compile_pattern(&self.pattern_set.imports);

        for patterns in self.pattern_set.framework_patterns.values() {
            compile_pattern(patterns);
        }
    }

    /// Analyzes a function and returns pattern match results.
    ///
    /// # Arguments
    ///
    /// * `function` - The function item to analyze
    /// * `source` - The source code containing the function
    ///
    /// # Returns
    ///
    /// A `MatchResult` containing confidence scores and matched patterns
    pub fn analyze_function(&self, function: &ItemFn, _source: &str) -> MatchResult {
        let mut result = MatchResult::new();

        // Check function name patterns
        let fn_name = function.sig.ident.to_string();
        self.match_patterns(&fn_name, &self.pattern_set.function_names, &mut result);

        // Check attributes
        for attr in &function.attrs {
            let attr_str = quote::quote!(#attr).to_string();
            self.match_patterns(&attr_str, &self.pattern_set.attributes, &mut result);
        }

        // Extract function body as string for pattern matching
        let body_str = quote::quote!(#function).to_string();

        // Check assertion patterns
        self.match_patterns(&body_str, &self.pattern_set.assertions, &mut result);

        // Check error handling patterns
        self.match_patterns(&body_str, &self.pattern_set.error_handling, &mut result);

        // Check framework-specific patterns
        let mut detected_frameworks = HashSet::new();
        for (framework, patterns) in &self.pattern_set.framework_patterns {
            let matches_before = result.matches.len();
            self.match_patterns(&body_str, patterns, &mut result);
            if result.matches.len() > matches_before {
                detected_frameworks.insert(framework.clone());
            }
        }
        result.frameworks = detected_frameworks.into_iter().collect();

        // Finalize the result
        result.finalize();
        result
    }

    /// Analyzes a module and returns pattern match results.
    ///
    /// # Arguments
    ///
    /// * `module` - The module item to analyze
    /// * `source` - The source code containing the module
    ///
    /// # Returns
    ///
    /// A `MatchResult` containing confidence scores and matched patterns
    pub fn analyze_module(&self, module: &ItemMod, source: &str) -> MatchResult {
        let mut result = MatchResult::new();

        // Check module name
        let mod_name = module.ident.to_string();
        if mod_name == "tests" || mod_name == "test" || mod_name.ends_with("_tests") {
            result.add_match("module_name", 1.0, &mod_name);
        }

        // Check module attributes
        for attr in &module.attrs {
            let attr_str = quote::quote!(#attr).to_string();
            self.match_patterns(&attr_str, &self.pattern_set.attributes, &mut result);

            // Special handling for #[cfg(test)]
            if attr_str.contains("cfg(test)") {
                result.add_match("#[cfg(test)]", 1.0, &attr_str);
            }
        }

        // Check module content if available
        if let Some(content) = &module.content {
            // Check for test functions within the module
            for item in &content.1 {
                if let Item::Fn(func) = item {
                    let func_result = self.analyze_function(func, source);
                    // Aggregate results from nested functions
                    for detail in func_result.matches {
                        result.matches.push(detail);
                    }
                }
            }
        }

        result.finalize();
        result
    }

    /// Analyzes an entire file and returns pattern match results.
    ///
    /// # Arguments
    ///
    /// * `file` - The parsed file AST
    /// * `source` - The source code of the file
    ///
    /// # Returns
    ///
    /// A `MatchResult` containing confidence scores and matched patterns
    pub fn analyze_file(&self, file: &File, source: &str) -> MatchResult {
        let mut result = MatchResult::new();

        // Check file-level attributes
        for attr in &file.attrs {
            let attr_str = quote::quote!(#attr).to_string();
            self.match_patterns(&attr_str, &self.pattern_set.attributes, &mut result);
        }

        // Analyze each item in the file
        for item in &file.items {
            match item {
                Item::Fn(func) => {
                    let func_result = self.analyze_function(func, source);
                    self.merge_results(&mut result, func_result);
                }
                Item::Mod(module) => {
                    let mod_result = self.analyze_module(module, source);
                    self.merge_results(&mut result, mod_result);
                }
                _ => {}
            }
        }

        // Check imports at file level
        let file_str = quote::quote!(#file).to_string();
        self.match_patterns(&file_str, &self.pattern_set.imports, &mut result);

        result.finalize();
        result
    }

    /// Analyzes raw source code text without parsing.
    ///
    /// This is useful for quick pattern matching without full AST parsing.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to analyze
    ///
    /// # Returns
    ///
    /// A `MatchResult` containing confidence scores and matched patterns
    pub fn analyze_source(&self, source: &str) -> MatchResult {
        let mut result = MatchResult::new();

        // Check all pattern categories
        self.match_patterns(source, &self.pattern_set.function_names, &mut result);
        self.match_patterns(source, &self.pattern_set.attributes, &mut result);
        self.match_patterns(source, &self.pattern_set.assertions, &mut result);
        self.match_patterns(source, &self.pattern_set.error_handling, &mut result);
        self.match_patterns(source, &self.pattern_set.modules, &mut result);
        self.match_patterns(source, &self.pattern_set.imports, &mut result);

        // Check framework patterns
        let mut detected_frameworks = HashSet::new();
        for (framework, patterns) in &self.pattern_set.framework_patterns {
            let matches_before = result.matches.len();
            self.match_patterns(source, patterns, &mut result);
            if result.matches.len() > matches_before {
                detected_frameworks.insert(framework.clone());
            }
        }
        result.frameworks = detected_frameworks.into_iter().collect();

        result.finalize();
        result
    }

    /// Matches patterns against text and updates the result.
    fn match_patterns(&self, text: &str, patterns: &[Pattern], result: &mut MatchResult) {
        for pattern in patterns {
            if pattern.is_regex {
                if let Some(regex) = self.compiled_patterns.get(&pattern.pattern) {
                    for capture in regex.find_iter(text) {
                        result.add_match(&pattern.pattern, pattern.weight, capture.as_str());
                    }
                }
            } else if text.contains(&pattern.pattern) {
                result.add_match(&pattern.pattern, pattern.weight, &pattern.pattern);
            }
        }
    }

    /// Merges two match results together.
    fn merge_results(&self, target: &mut MatchResult, source: MatchResult) {
        target.matches.extend(source.matches);
        target.frameworks.extend(source.frameworks);

        for (category, score) in source.category_scores {
            *target.category_scores.entry(category).or_insert(0.0) += score;
        }
    }

    /// Determines if the given code is likely test-related based on pattern matching.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to check
    ///
    /// # Returns
    ///
    /// `true` if the confidence score exceeds the threshold
    pub fn is_test_code(&self, source: &str) -> bool {
        let result = self.analyze_source(source);
        result.is_confident(self.confidence_threshold)
    }

    /// Returns the pattern set being used by this matcher.
    pub fn pattern_set(&self) -> &PatternSet {
        &self.pattern_set
    }

    /// Returns a mutable reference to the pattern set.
    pub fn pattern_set_mut(&mut self) -> &mut PatternSet {
        &mut self.pattern_set
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::Category;

    #[test]
    fn test_basic_function_matching() {
        let matcher = PatternMatcher::new();

        let test_code = r#"
            #[test]
            fn test_addition() {
                assert_eq!(2 + 2, 4);
            }
        "#;

        let result = matcher.analyze_source(test_code);
        assert!(result.confidence > 0.8);
        assert_eq!(result.category, Category::UnitTest);
    }

    #[test]
    fn test_property_test_detection() {
        let matcher = PatternMatcher::new();

        let test_code = r#"
            #[quickcheck]
            fn prop_reverse_twice(xs: Vec<i32>) -> bool {
                let reversed_once: Vec<i32> = xs.iter().cloned().rev().collect();
                let reversed_twice: Vec<i32> = reversed_once.iter().cloned().rev().collect();
                xs == reversed_twice
            }
        "#;

        let result = matcher.analyze_source(test_code);
        assert!(result.confidence > 0.7);
        assert_eq!(result.category, Category::PropertyTest);
    }

    #[test]
    fn test_framework_detection() {
        let matcher = PatternMatcher::new();

        let test_code = r#"
            use mockall::*;
            
            #[automock]
            trait MyTrait {
                fn foo(&self, x: i32) -> i32;
            }
        "#;

        let result = matcher.analyze_source(test_code);
        assert!(result.frameworks.contains(&"mockall".to_string()));
    }
}
