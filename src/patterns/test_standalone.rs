//! Standalone test for the patterns module to verify it compiles correctly

#[cfg(test)]
mod tests {
    use crate::patterns::{Pattern, PatternSet, PatternMatcher, MatchResult, Category};
    
    #[test]
    fn test_pattern_creation() {
        // Test simple pattern creation
        let pattern = Pattern::simple("test_", 0.9);
        assert_eq!(pattern.pattern, "test_");
        assert_eq!(pattern.weight, 0.9);
        assert!(!pattern.is_regex);
        
        // Test regex pattern creation
        let regex_pattern = Pattern::regex(r"^test_\w+", 0.8);
        assert_eq!(regex_pattern.pattern, r"^test_\w+");
        assert!(regex_pattern.is_regex);
        
        // Test pattern with description
        let described = Pattern::simple("should_", 0.7)
            .with_description("BDD style test");
        assert!(described.description.is_some());
    }
    
    #[test]
    fn test_pattern_set_creation() {
        let mut pattern_set = PatternSet::new();
        assert!(pattern_set.function_names.is_empty());
        
        // Add a pattern
        pattern_set.add_pattern("function_names", Pattern::simple("test_", 0.9));
        assert_eq!(pattern_set.function_names.len(), 1);
        
        // Test default pattern set
        let defaults = PatternSet::with_defaults();
        assert!(!defaults.function_names.is_empty());
        assert!(!defaults.attributes.is_empty());
        assert!(!defaults.framework_patterns.is_empty());
    }
    
    #[test]
    fn test_match_result() {
        let mut result = MatchResult::new();
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.category, Category::Unknown);
        
        // Add a match
        result.add_match("#[test]", 1.0, "#[test]");
        assert_eq!(result.matches.len(), 1);
        
        // Finalize and check confidence
        result.finalize();
        assert!(result.confidence > 0.0);
    }
    
    #[test]
    fn test_pattern_matcher_basic() {
        let matcher = PatternMatcher::new();
        
        let test_code = r#"
            #[test]
            fn test_something() {
                assert_eq!(1 + 1, 2);
            }
        "#;
        
        let result = matcher.analyze_source(test_code);
        assert!(result.confidence > 0.5);
        assert!(result.is_confident(0.5));
        
        // Should detect as unit test
        assert_eq!(result.category, Category::UnitTest);
    }
    
    #[test]
    fn test_category_names() {
        assert_eq!(Category::UnitTest.name(), "Unit Test");
        assert_eq!(Category::PropertyTest.name(), "Property Test");
        assert_eq!(Category::Mock.name(), "Mock/Stub");
        
        assert_eq!(Category::UnitTest.id(), "unit");
        assert_eq!(Category::PropertyTest.id(), "property");
        assert_eq!(Category::Mock.id(), "mock");
    }
}