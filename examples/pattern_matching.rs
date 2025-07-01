//! Example demonstrating the PatternMatcher functionality.

use instrument_rs::patterns::{Pattern, PatternMatcher, PatternSet};

fn main() {
    // Create a pattern matcher with default patterns
    let matcher = PatternMatcher::new();

    // Example 1: Basic unit test detection
    println!("=== Example 1: Unit Test Detection ===");
    let unit_test_code = r#"
        #[test]
        fn test_addition() {
            assert_eq!(2 + 2, 4);
        }
    "#;

    let result = matcher.analyze_source(unit_test_code);
    println!("Confidence: {:.2}", result.confidence);
    println!("Category: {:?}", result.category);
    println!("Matched patterns:");
    for m in &result.matches {
        println!("  - {} (weight: {:.2})", m.pattern, m.weight);
    }
    println!();

    // Example 2: Property test detection
    println!("=== Example 2: Property Test Detection ===");
    let property_test_code = r#"
        use quickcheck::quickcheck;
        
        #[quickcheck]
        fn prop_reverse_reverse_is_identity(xs: Vec<i32>) -> bool {
            let rev1: Vec<_> = xs.iter().cloned().rev().collect();
            let rev2: Vec<_> = rev1.iter().cloned().rev().collect();
            xs == rev2
        }
    "#;

    let result = matcher.analyze_source(property_test_code);
    println!("Confidence: {:.2}", result.confidence);
    println!("Category: {:?}", result.category);
    println!("Top 3 categories:");
    for (cat, score) in result.top_categories(3) {
        println!("  - {:?}: {:.2}", cat, score);
    }
    println!();

    // Example 3: Framework detection
    println!("=== Example 3: Framework Detection ===");
    let framework_test_code = r#"
        use mockall::*;
        use spectral::prelude::*;
        
        #[automock]
        trait Database {
            fn get_user(&self, id: u64) -> Option<User>;
        }
        
        #[test]
        fn test_user_service() {
            let mut mock = MockDatabase::new();
            mock.expect_get_user()
                .with(predicate::eq(42))
                .times(1)
                .returning(|_| Some(User { id: 42, name: "Test".into() }));
                
            assert_that!(service.get_user(42)).is_some();
        }
    "#;

    let result = matcher.analyze_source(framework_test_code);
    println!("Confidence: {:.2}", result.confidence);
    println!("Category: {:?}", result.category);
    println!("Detected frameworks: {:?}", result.frameworks);
    println!();

    // Example 4: Custom patterns
    println!("=== Example 4: Custom Patterns ===");
    let mut custom_pattern_set = PatternSet::with_defaults();

    // Add custom patterns for a hypothetical testing DSL
    custom_pattern_set.add_pattern(
        "function_names",
        Pattern::simple("scenario_", 0.9).with_description("Custom scenario-based test functions"),
    );

    custom_pattern_set.add_pattern(
        "framework_patterns",
        Pattern::simple("verify!", 0.8).with_description("Custom verification macro"),
    );

    let custom_matcher = PatternMatcher::with_pattern_set(custom_pattern_set);

    let custom_test_code = r#"
        fn scenario_user_login() {
            // Setup
            let user = create_test_user();
            
            // Action
            let result = login_service.authenticate(&user);
            
            // Verify
            verify!(result.is_ok());
            verify!(result.unwrap().token.is_some());
        }
    "#;

    let result = custom_matcher.analyze_source(custom_test_code);
    println!("Confidence: {:.2}", result.confidence);
    println!("Matched custom patterns:");
    for m in &result.matches {
        if m.pattern.contains("scenario") || m.pattern.contains("verify") {
            println!("  - {} (weight: {:.2})", m.pattern, m.weight);
        }
    }
    println!();

    // Example 5: Test utility detection
    println!("=== Example 5: Test Utility Detection ===");
    let test_utility_code = r#"
        mod test_helpers {
            use super::*;
            
            pub fn setup_test_database() -> TestDb {
                let db = TestDb::new();
                db.migrate();
                db
            }
            
            pub fn create_test_user(name: &str) -> User {
                User {
                    id: rand::random(),
                    name: name.to_string(),
                    email: format!("{}@test.com", name),
                }
            }
            
            pub fn teardown_test_env() {
                // Cleanup logic
            }
        }
    "#;

    let result = matcher.analyze_source(test_utility_code);
    println!("Confidence: {:.2}", result.confidence);
    println!("Category: {:?}", result.category);
    println!("Utility indicators found:");
    for m in &result.matches {
        if m.matched_text.contains("helper")
            || m.matched_text.contains("setup")
            || m.matched_text.contains("teardown")
        {
            println!("  - {}", m.matched_text);
        }
    }
}
