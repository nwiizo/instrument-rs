//! Example demonstrating the instrumentation scoring system

use instrument_rs::scoring::{
    InstrumentationPriority, InstrumentationScorer, ScoringFactor,
};
use std::collections::HashMap;

fn main() {
    println!("=== Instrumentation Scoring Demo ===\n");
    
    let mut scorer = InstrumentationScorer::new();
    
    // Example functions to score
    let functions = vec![
        ("process_payment", 15, true, 3, true),
        ("validate_user_input", 8, true, 0, true),
        ("calculate_tax", 12, false, 1, false),
        ("format_string", 2, false, 0, false),
        ("authenticate_user", 10, true, 2, true),
        ("save_to_database", 5, true, 1, true),
        ("complex_algorithm", 35, false, 0, false),
        ("send_notification", 6, true, 2, true),
    ];
    
    println!("Scoring {} functions...\n", functions.len());
    
    let mut results = Vec::new();
    
    for (name, complexity, has_error_handling, external_calls, is_public) in &functions {
        let score = scorer.score_function(
            name,
            *complexity,
            *has_error_handling,
            *external_calls,
            *is_public,
        );
        
        results.push((name.to_string(), score));
    }
    
    // Sort by score (highest first)
    results.sort_by(|a, b| b.1.overall_score.partial_cmp(&a.1.overall_score).unwrap());
    
    // Display results
    println!("{:<25} {:>10} {:>15} {}", "Function", "Score", "Priority", "Reasoning");
    println!("{}", "-".repeat(80));
    
    for (name, score) in &results {
        let priority_str = format!("{:?}", score.priority);
        println!(
            "{:<25} {:>10.2} {:>15} {}",
            name,
            score.overall_score,
            priority_str,
            score.reasoning.join("; ")
        );
    }
    
    println!("\n=== Priority Summary ===");
    
    let mut priority_counts = HashMap::new();
    for (_, score) in &results {
        *priority_counts.entry(score.priority).or_insert(0) += 1;
    }
    
    for priority in [
        InstrumentationPriority::Critical,
        InstrumentationPriority::High,
        InstrumentationPriority::Medium,
        InstrumentationPriority::Low,
        InstrumentationPriority::Minimal,
    ] {
        let count = priority_counts.get(&priority).unwrap_or(&0);
        println!("{:?}: {} functions", priority, count);
    }
    
    println!("\n=== Factor Analysis ===");
    
    // Show how different factors contribute
    if let Some((name, score)) = results.first() {
        println!("\nTop scoring function: {}", name);
        for (factor, value) in &score.factor_scores {
            println!("  {}: {:.2}", factor.name(), value);
        }
    }
    
    println!("\n=== Custom Weight Example ===");
    
    // Create a scorer that heavily weights external calls
    let mut api_focused_scorer = InstrumentationScorer::new();
    api_focused_scorer.set_weight(ScoringFactor::ExternalCalls, 0.6);
    api_focused_scorer.set_weight(ScoringFactor::BusinessCriticality, 0.2);
    api_focused_scorer.set_weight(ScoringFactor::ErrorHandling, 0.1);
    api_focused_scorer.set_weight(ScoringFactor::Complexity, 0.1);
    
    println!("\nRescoring with API-focused weights...");
    
    for (name, complexity, has_error_handling, external_calls, is_public) in &functions[0..3] {
        let original = scorer.score_function(name, *complexity, *has_error_handling, *external_calls, *is_public);
        let api_focused = api_focused_scorer.score_function(name, *complexity, *has_error_handling, *external_calls, *is_public);
        
        println!(
            "{}: Original={:.2}, API-focused={:.2} ({})",
            name,
            original.overall_score,
            api_focused.overall_score,
            if api_focused.overall_score > original.overall_score { "↑" } else { "↓" }
        );
    }
}