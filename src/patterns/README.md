# Pattern Matching Module

This module implements the pattern matching functionality described in section 1.2 of the specification. It provides a weighted scoring system for identifying test-related patterns in Rust source code.

## Components

### PatternSet
A collection of patterns organized by category:
- **Function names**: Patterns like `test_`, `should_`, `it_`
- **Attributes**: `#[test]`, `#[tokio::test]`, `#[quickcheck]`, etc.
- **Framework patterns**: Framework-specific patterns (mockall, spectral, cucumber)
- **Assertions**: `assert!`, `assert_eq!`, `expect`, etc.
- **Error handling**: `should_panic`, `is_err()`, `Result<(), _>`
- **Modules**: `mod tests`, `#[cfg(test)]`
- **Imports**: Test-related imports

### Pattern
Individual patterns with:
- Pattern string (simple string or regex)
- Weight (0.0 to 1.0)
- Optional description

### PatternMatcher
The main matching engine that:
- Analyzes functions, modules, and entire files
- Calculates confidence scores
- Determines test categories
- Detects testing frameworks

### MatchResult
Contains:
- Overall confidence score
- Category scores for each test type
- Matched patterns with details
- Detected frameworks
- The most likely category

## Categories

The matcher can identify these test categories:
- `UnitTest`: Standard unit tests
- `IntegrationTest`: Integration tests
- `PropertyTest`: Property-based tests (QuickCheck, Proptest)
- `Benchmark`: Performance benchmarks
- `Fuzz`: Fuzzing tests
- `Mock`: Mock implementations
- `TestUtility`: Test helpers and utilities
- `Example`: Example code
- `Unknown`: Uncategorized test code

## Usage

```rust
use instrument_rs::patterns::{PatternMatcher, PatternSet, Pattern};

// Create a matcher with default patterns
let matcher = PatternMatcher::new();

// Analyze source code
let result = matcher.analyze_source(source_code);

println!("Confidence: {:.2}", result.confidence);
println!("Category: {:?}", result.category);
println!("Frameworks: {:?}", result.frameworks);

// Create custom patterns
let mut pattern_set = PatternSet::with_defaults();
pattern_set.add_pattern(
    "function_names",
    Pattern::simple("scenario_", 0.9)
        .with_description("BDD scenario functions")
);

let custom_matcher = PatternMatcher::with_pattern_set(pattern_set);
```

## Confidence Scoring

The confidence score is calculated based on:
1. Number of patterns matched
2. Weight of each matched pattern
3. Normalization to 0.0-1.0 range

Higher weights indicate stronger indicators of test code.

## Extending

To add support for new test frameworks or patterns:

1. Add patterns to the appropriate category in `PatternSet::with_defaults()`
2. Update category determination logic in `MatchResult::calculate_category_scores()`
3. Add framework-specific patterns to detect the framework

## Performance

- Regex patterns are pre-compiled for efficiency
- Simple string matching is used where possible
- Patterns are organized by category to allow selective matching