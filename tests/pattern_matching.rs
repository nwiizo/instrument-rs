//! Integration tests for the pattern matching functionality.

use instrument_rs::patterns::{Category, Pattern, PatternMatcher, PatternSet};

#[test]
fn test_unit_test_detection() {
    let matcher = PatternMatcher::new();

    // Standard Rust test
    let code = r#"
        #[test]
        fn test_something() {
            assert_eq!(1 + 1, 2);
        }
    "#;

    let result = matcher.analyze_source(code);
    assert!(result.confidence > 0.8);
    assert_eq!(result.category, Category::UnitTest);
    assert!(result.is_confident(0.5));
}

#[test]
fn test_async_test_detection() {
    let matcher = PatternMatcher::new();

    // Tokio async test
    let code = r#"
        #[tokio::test]
        async fn test_async_operation() {
            let result = async_function().await;
            assert!(result.is_ok());
        }
    "#;

    let result = matcher.analyze_source(code);
    assert!(result.confidence > 0.8);
    assert_eq!(result.category, Category::UnitTest);
}

#[test]
#[ignore = "Pattern matcher needs implementation"]
fn test_property_test_detection() {
    let matcher = PatternMatcher::new();

    // QuickCheck property test
    let quickcheck_code = r#"
        #[quickcheck]
        fn prop_addition_commutative(a: i32, b: i32) -> bool {
            a + b == b + a
        }
    "#;

    let result = matcher.analyze_source(quickcheck_code);
    assert!(result.confidence > 0.7);
    assert_eq!(result.category, Category::PropertyTest);

    // Proptest property test
    let proptest_code = r#"
        #[proptest]
        fn test_vector_reverse(#[strategy(0..100)] size: usize) {
            let vec: Vec<i32> = (0..size).collect();
            let reversed: Vec<_> = vec.iter().rev().cloned().collect();
            let double_reversed: Vec<_> = reversed.iter().rev().cloned().collect();
            prop_assert_eq!(vec, double_reversed);
        }
    "#;

    let result = matcher.analyze_source(proptest_code);
    assert!(result.confidence > 0.7);
    assert_eq!(result.category, Category::PropertyTest);
}

#[test]
fn test_mock_detection() {
    let matcher = PatternMatcher::new();

    let code = r#"
        use mockall::*;
        
        #[automock]
        trait MyTrait {
            fn foo(&self) -> i32;
        }
        
        #[test]
        fn test_with_mock() {
            let mut mock = MockMyTrait::new();
            mock.expect_foo()
                .times(1)
                .returning(|| 42);
            
            assert_eq!(mock.foo(), 42);
        }
    "#;

    let result = matcher.analyze_source(code);
    assert!(result.confidence > 0.6);
    assert!(result.frameworks.contains(&"mockall".to_string()));

    // Check that mock patterns contribute to category scores
    let mock_score = result
        .category_scores
        .get(&Category::Mock)
        .copied()
        .unwrap_or(0.0);
    assert!(mock_score > 0.0);
}

#[test]
fn test_framework_detection() {
    let matcher = PatternMatcher::new();

    // Spectral framework
    let spectral_code = r#"
        use spectral::prelude::*;
        
        #[test]
        fn test_with_spectral() {
            let value = 42;
            assert_that!(value).is_equal_to(42);
            assert_that!(vec![1, 2, 3]).contains(2);
        }
    "#;

    let result = matcher.analyze_source(spectral_code);
    assert!(result.frameworks.contains(&"spectral".to_string()));

    // Cucumber framework
    let cucumber_code = r#"
        #[given("a user with name {string}")]
        fn given_user(world: &mut World, name: String) {
            world.user = Some(User { name });
        }
        
        #[when("the user logs in")]
        fn when_login(world: &mut World) {
            world.login_result = login_service.login(&world.user.unwrap());
        }
        
        #[then("the login should succeed")]
        fn then_success(world: &mut World) {
            assert!(world.login_result.is_ok());
        }
    "#;

    let result = matcher.analyze_source(cucumber_code);
    assert!(result.frameworks.contains(&"cucumber".to_string()));
}

#[test]
#[ignore = "Pattern matcher needs implementation"]
fn test_test_module_detection() {
    let matcher = PatternMatcher::new();

    let code = r#"
        #[cfg(test)]
        mod tests {
            use super::*;
            
            #[test]
            fn test_functionality() {
                assert!(true);
            }
        }
    "#;

    let result = matcher.analyze_source(code);
    assert!(result.confidence > 0.9);
    // Should find both module pattern and test attribute
    assert!(result.matches.len() >= 2);
}

#[test]
#[ignore = "Pattern matcher needs implementation"]
fn test_custom_patterns() {
    let mut pattern_set = PatternSet::new();

    // Add custom patterns
    pattern_set
        .function_names
        .push(Pattern::simple("verify_", 0.9).with_description("Custom verify functions"));

    pattern_set
        .assertions
        .push(Pattern::simple("check!", 0.8).with_description("Custom check macro"));

    let matcher = PatternMatcher::with_pattern_set(pattern_set);

    let code = r#"
        fn verify_user_creation() {
            let user = create_user("test");
            check!(user.is_valid());
            check!(user.name == "test");
        }
    "#;

    let result = matcher.analyze_source(code);
    assert!(result.confidence > 0.7);
    assert_eq!(result.matches.len(), 3); // 1 function name + 2 check! macros
}

#[test]
fn test_confidence_threshold() {
    let mut matcher = PatternMatcher::new();
    matcher.set_confidence_threshold(0.7);

    // Code with low confidence
    let low_confidence_code = r#"
        fn maybe_test() {
            println!("This might be a test");
        }
    "#;

    assert!(!matcher.is_test_code(low_confidence_code));

    // Code with high confidence
    let high_confidence_code = r#"
        #[test]
        fn test_something() {
            assert_eq!(1, 1);
        }
    "#;

    assert!(matcher.is_test_code(high_confidence_code));
}

#[test]
fn test_pattern_weight_calculation() {
    let matcher = PatternMatcher::new();

    let code = r#"
        #[test]
        fn test_with_multiple_patterns() {
            assert!(true);
            assert_eq!(1, 1);
            assert_ne!(2, 3);
            
            let result = something();
            result.expect("should work");
        }
    "#;

    let result = matcher.analyze_source(code);

    // Should match multiple patterns with different weights
    assert!(result.matches.len() > 4);

    // Verify weights are properly recorded
    for m in &result.matches {
        assert!(m.weight > 0.0);
        assert!(m.weight <= 1.0);
    }
}

#[test]
fn test_error_handling_patterns() {
    let matcher = PatternMatcher::new();

    let code = r#"
        #[test]
        #[should_panic(expected = "invalid input")]
        fn test_panic_behavior() {
            function_that_panics();
        }
        
        #[test]
        fn test_error_handling() {
            let result: Result<(), Error> = fallible_function();
            assert!(result.is_err());
            
            match result {
                Err(e) => println!("Got expected error: {}", e),
                Ok(_) => panic!("Should have failed"),
            }
        }
    "#;

    let result = matcher.analyze_source(code);

    // Should detect error handling patterns
    let has_error_patterns = result
        .matches
        .iter()
        .any(|m| m.pattern.contains("should_panic") || m.pattern.contains("is_err"));
    assert!(has_error_patterns);
}

#[test]
#[ignore = "Pattern matcher needs implementation"]
fn test_test_utility_detection() {
    let matcher = PatternMatcher::new();

    let code = r#"
        mod test_helpers {
            pub fn setup_test_database() -> TestDb {
                TestDb::new()
            }
            
            pub fn create_fixture() -> Fixture {
                Fixture::default()
            }
            
            pub fn teardown() {
                // cleanup
            }
        }
    "#;

    let result = matcher.analyze_source(code);

    // Should detect utility patterns
    let utility_score = result
        .category_scores
        .get(&Category::TestUtility)
        .copied()
        .unwrap_or(0.0);
    assert!(utility_score > 0.0);
}

#[test]
fn test_top_categories() {
    let matcher = PatternMatcher::new();

    // Code that could match multiple categories
    let code = r#"
        use mockall::*;
        
        #[automock]
        trait Service {
            fn process(&self) -> Result<(), Error>;
        }
        
        #[test]
        fn test_service_mock() {
            let mut mock = MockService::new();
            mock.expect_process()
                .returning(|| Ok(()));
            
            assert!(mock.process().is_ok());
        }
    "#;

    let result = matcher.analyze_source(code);
    let top_cats = result.top_categories(3);

    // Should have multiple categories with scores
    assert!(!top_cats.is_empty());

    // Scores should be in descending order
    for i in 1..top_cats.len() {
        assert!(top_cats[i - 1].1 >= top_cats[i].1);
    }
}
