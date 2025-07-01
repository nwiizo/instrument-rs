# Instrumentation Scoring System

The instrumentation scoring system prioritizes which code elements should be instrumented based on multiple factors, as specified in section 4 of the project specification.

## Overview

The scoring system evaluates functions and code blocks across four main factors:

1. **Business Criticality (35% weight)** - How important is this function to core business logic?
2. **Error Handling (25% weight)** - Does the function properly handle errors?
3. **External Calls (25% weight)** - Does the function interact with external systems?
4. **Complexity (15% weight)** - How complex is the function's logic?

## Usage

### Basic Scoring

```rust
use instrument_rs::scoring::InstrumentationScorer;

let scorer = InstrumentationScorer::new();

let score = scorer.score_function(
    "process_payment",  // function name
    15,                 // complexity
    true,               // has error handling
    2,                  // external call count
    true,               // is public
);

println!("Score: {:.2}", score.overall_score);
println!("Priority: {:?}", score.priority);
```

### Custom Weights

You can customize the weights for different factors:

```rust
let mut scorer = InstrumentationScorer::new();

// Heavily weight external calls for API-focused analysis
scorer.set_weight(ScoringFactor::ExternalCalls, 0.6);
scorer.set_weight(ScoringFactor::BusinessCriticality, 0.2);
scorer.set_weight(ScoringFactor::ErrorHandling, 0.1);
scorer.set_weight(ScoringFactor::Complexity, 0.1);
```

### AST-Based Analysis

For analyzing entire files:

```rust
use instrument_rs::scoring::InstrumentationAnalyzer;
use syn::parse_file;

let code = std::fs::read_to_string("src/main.rs")?;
let file = parse_file(&code)?;

let mut analyzer = InstrumentationAnalyzer::new();
let report = analyzer.generate_report(&file);

// Get high-priority functions
let critical_functions = analyzer.get_high_priority_functions(
    &file,
    InstrumentationPriority::High
);
```

## Priority Levels

Functions are assigned one of five priority levels based on their score:

| Priority | Score Range | Description |
|----------|-------------|-------------|
| Critical | 80-100 | Must instrument - high business impact |
| High | 60-79 | Should instrument - significant risk or complexity |
| Medium | 40-59 | Consider instrumenting - moderate complexity |
| Low | 20-39 | Optional - simple logic with minimal impact |
| Minimal | 0-19 | Rarely needs instrumentation - trivial code |

## Scoring Factors

### Business Criticality

Functions are scored higher if they:
- Contain keywords like "payment", "auth", "security", "transaction"
- Are public APIs
- Handle validation or verification
- Process important business data

### Error Handling

The system evaluates:
- Presence of Result return types
- Use of ? operator
- Try/catch blocks
- Match expressions on Result/Option

### External Calls

Detects and counts:
- HTTP/API calls
- Database operations
- File system operations
- Network requests
- Third-party service integrations

### Complexity

Based on:
- Cyclomatic complexity
- Number of branches (if/match)
- Loop nesting depth
- Overall function length

## Integration Example

```rust
// Analyze a codebase and instrument high-priority functions
let mut analyzer = InstrumentationAnalyzer::new();
let scores = analyzer.analyze_and_score(&file);

for (function_name, score) in scores {
    if score.priority >= InstrumentationPriority::High {
        // Apply instrumentation to this function
        println!("Instrumenting {}: {}", function_name, score.reasoning.join(", "));
    }
}
```

## Configuration

The scoring system can be configured through `ScoringConfig`:

```rust
use instrument_rs::scoring::ScoringConfig;
use std::collections::HashMap;

let mut weights = HashMap::new();
weights.insert(ScoringFactor::BusinessCriticality, 0.4);
weights.insert(ScoringFactor::ErrorHandling, 0.3);
weights.insert(ScoringFactor::ExternalCalls, 0.2);
weights.insert(ScoringFactor::Complexity, 0.1);

let config = ScoringConfig {
    weights,
    thresholds: Default::default(),
    critical_patterns: vec!["payment".to_string(), "auth".to_string()],
    external_patterns: vec!["http".to_string(), "database".to_string()],
};

let scorer = InstrumentationScorer::with_config(config);
```

## Best Practices

1. **Start with defaults** - The default weights are well-balanced for most applications
2. **Adjust for your domain** - Financial apps might weight error handling higher
3. **Use patterns** - Add domain-specific patterns to identify critical functions
4. **Review scores** - Periodically review scoring results to refine weights
5. **Combine with coverage** - Use alongside coverage data for comprehensive analysis