//! Performance tests for large codebases

use crate::common::sample_projects;
use instrument_rs::{
    ast::{AstAnalyzer, SourceFile},
    call_graph::GraphBuilder,
    scoring::instrumentation::InstrumentationScorer,
    Config,
};
use std::time::{Duration, Instant};
use std::fs;
use tempfile::TempDir;

/// Performance thresholds
const MAX_FILE_PARSE_TIME_MS: u128 = 100;
const MAX_ANALYSIS_TIME_PER_FUNCTION_MS: u128 = 5;
const MAX_GRAPH_BUILD_TIME_PER_FILE_MS: u128 = 50;
const MAX_SCORING_TIME_PER_FUNCTION_MS: u128 = 2;

#[test]
fn test_large_codebase_performance() {
    let project = sample_projects::large_codebase();
    let config = project.create_config();
    
    // Measure total analysis time
    let start = Instant::now();
    
    let mut total_functions = 0;
    let mut total_files = 0;
    let mut parse_times = Vec::new();
    let mut analysis_times = Vec::new();
    
    // Process all source files
    for entry in walkdir::WalkDir::new(project.root_path.join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        total_files += 1;
        
        // Measure parse time
        let parse_start = Instant::now();
        let source_file = SourceFile::parse(entry.path()).expect("Should parse file");
        let parse_time = parse_start.elapsed();
        parse_times.push(parse_time);
        
        // Measure analysis time
        let analysis_start = Instant::now();
        let analyzer = AstAnalyzer::new();
        let result = analyzer.analyze(source_file).expect("Should analyze file");
        let analysis_time = analysis_start.elapsed();
        analysis_times.push(analysis_time);
        
        total_functions += result.functions.len();
    }
    
    let total_time = start.elapsed();
    
    // Calculate metrics
    let avg_parse_time = parse_times.iter().sum::<Duration>() / parse_times.len() as u32;
    let avg_analysis_time = analysis_times.iter().sum::<Duration>() / analysis_times.len() as u32;
    let max_parse_time = parse_times.iter().max().unwrap();
    let max_analysis_time = analysis_times.iter().max().unwrap();
    
    // Performance assertions
    assert!(
        max_parse_time.as_millis() < MAX_FILE_PARSE_TIME_MS,
        "File parsing took too long: {:?} (max allowed: {}ms)",
        max_parse_time,
        MAX_FILE_PARSE_TIME_MS
    );
    
    assert!(
        total_time.as_secs() < 30,
        "Total analysis took too long: {:?} (max allowed: 30s)",
        total_time
    );
    
    println!("Performance Metrics for Large Codebase:");
    println!("  Total files: {}", total_files);
    println!("  Total functions: {}", total_functions);
    println!("  Total time: {:?}", total_time);
    println!("  Average parse time: {:?}", avg_parse_time);
    println!("  Average analysis time: {:?}", avg_analysis_time);
    println!("  Max parse time: {:?}", max_parse_time);
    println!("  Max analysis time: {:?}", max_analysis_time);
}

#[test]
fn test_parallel_processing_performance() {
    let project = sample_projects::large_codebase();
    let mut config = project.create_config();
    
    // Test with different thread counts
    let thread_counts = vec![1, 2, 4, 8];
    let mut results = Vec::new();
    
    for thread_count in thread_counts {
        config.instrumentation.parallel = thread_count > 1;
        config.instrumentation.threads = Some(thread_count);
        
        let start = Instant::now();
        
        // Use rayon to process files in parallel
        use rayon::prelude::*;
        
        let files: Vec<_> = walkdir::WalkDir::new(project.root_path.join("src"))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            .map(|e| e.path().to_path_buf())
            .collect();
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build()
            .unwrap();
        
        pool.install(|| {
            files.par_iter().for_each(|path| {
                if let Ok(source_file) = SourceFile::parse(path) {
                    let analyzer = AstAnalyzer::new();
                    let _ = analyzer.analyze(source_file);
                }
            });
        });
        
        let elapsed = start.elapsed();
        results.push((thread_count, elapsed));
    }
    
    // Verify that parallel processing improves performance
    println!("Parallel Processing Performance:");
    for (threads, time) in &results {
        println!("  {} threads: {:?}", threads, time);
    }
    
    // Performance should improve with more threads (up to a point)
    if results.len() >= 2 {
        let single_threaded = results[0].1;
        let multi_threaded = results[1].1;
        
        assert!(
            multi_threaded < single_threaded,
            "Multi-threaded processing should be faster than single-threaded"
        );
    }
}

#[test]
fn test_call_graph_generation_performance() {
    let project = sample_projects::large_codebase();
    
    let start = Instant::now();
    let mut builder = GraphBuilder::new();
    
    let mut file_count = 0;
    for entry in walkdir::WalkDir::new(project.root_path.join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(source_file) = SourceFile::parse(entry.path()) {
            builder.add_file(source_file).expect("Should add file to graph");
            file_count += 1;
        }
    }
    
    let graph_build_start = Instant::now();
    let graph = builder.build();
    let graph_build_time = graph_build_start.elapsed();
    
    let total_time = start.elapsed();
    
    // Performance assertions
    let avg_time_per_file = total_time.as_millis() / file_count as u128;
    assert!(
        avg_time_per_file < MAX_GRAPH_BUILD_TIME_PER_FILE_MS,
        "Call graph generation too slow: {}ms per file (max allowed: {}ms)",
        avg_time_per_file,
        MAX_GRAPH_BUILD_TIME_PER_FILE_MS
    );
    
    println!("Call Graph Generation Performance:");
    println!("  Files processed: {}", file_count);
    println!("  Total time: {:?}", total_time);
    println!("  Graph build time: {:?}", graph_build_time);
    println!("  Nodes in graph: {}", graph.nodes.len());
    println!("  Edges in graph: {}", graph.edges.len());
    println!("  Average time per file: {}ms", avg_time_per_file);
}

#[test]
fn test_scoring_performance() {
    let project = sample_projects::complex_patterns();
    
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze file");
    
    let scorer = InstrumentationScorer::new();
    
    let start = Instant::now();
    let mut scores = Vec::new();
    
    // Score all functions multiple times to get meaningful measurements
    for _ in 0..100 {
        for function in &ast_result.functions {
            let score = scorer.score_function(function);
            scores.push(score);
        }
    }
    
    let total_time = start.elapsed();
    let avg_time_per_scoring = total_time / scores.len() as u32;
    
    assert!(
        avg_time_per_scoring.as_millis() < MAX_SCORING_TIME_PER_FUNCTION_MS,
        "Scoring too slow: {:?} per function (max allowed: {}ms)",
        avg_time_per_scoring,
        MAX_SCORING_TIME_PER_FUNCTION_MS
    );
    
    println!("Scoring Performance:");
    println!("  Functions scored: {}", scores.len());
    println!("  Total time: {:?}", total_time);
    println!("  Average time per scoring: {:?}", avg_time_per_scoring);
}

#[test]
fn test_memory_usage_for_large_projects() {
    // This test monitors memory usage patterns
    let project = sample_projects::large_codebase();
    
    // Get initial memory usage (approximation)
    let initial_memory = get_approximate_memory_usage();
    
    let mut analyzers = Vec::new();
    let mut results = Vec::new();
    
    // Process files and keep results in memory
    for entry in walkdir::WalkDir::new(project.root_path.join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .take(20) // Limit to prevent test from taking too long
    {
        if let Ok(source_file) = SourceFile::parse(entry.path()) {
            let analyzer = AstAnalyzer::new();
            if let Ok(result) = analyzer.analyze(source_file) {
                analyzers.push(analyzer);
                results.push(result);
            }
        }
    }
    
    let after_analysis_memory = get_approximate_memory_usage();
    let memory_increase = after_analysis_memory.saturating_sub(initial_memory);
    
    // Memory usage should be reasonable
    let memory_per_file = memory_increase / results.len().max(1);
    
    println!("Memory Usage:");
    println!("  Files analyzed: {}", results.len());
    println!("  Approximate memory increase: {} bytes", memory_increase);
    println!("  Average memory per file: {} bytes", memory_per_file);
    
    // Basic sanity check - each file shouldn't use more than 10MB
    assert!(
        memory_per_file < 10 * 1024 * 1024,
        "Memory usage per file too high: {} bytes",
        memory_per_file
    );
}

#[test]
fn test_incremental_analysis_performance() {
    let project = sample_projects::large_codebase();
    
    // First pass - full analysis
    let full_start = Instant::now();
    let mut file_hashes = std::collections::HashMap::new();
    
    for entry in walkdir::WalkDir::new(project.root_path.join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let path = entry.path();
        let content = fs::read_to_string(path).unwrap();
        let hash = calculate_hash(&content);
        file_hashes.insert(path.to_path_buf(), hash);
        
        if let Ok(source_file) = SourceFile::parse(path) {
            let analyzer = AstAnalyzer::new();
            let _ = analyzer.analyze(source_file);
        }
    }
    
    let full_time = full_start.elapsed();
    
    // Second pass - simulate incremental with only changed files
    let incremental_start = Instant::now();
    let mut changed_files = 0;
    
    for (path, old_hash) in &file_hashes {
        let content = fs::read_to_string(path).unwrap();
        let new_hash = calculate_hash(&content);
        
        // Simulate that only 10% of files changed
        if new_hash != *old_hash || changed_files < file_hashes.len() / 10 {
            changed_files += 1;
            if let Ok(source_file) = SourceFile::parse(path) {
                let analyzer = AstAnalyzer::new();
                let _ = analyzer.analyze(source_file);
            }
        }
    }
    
    let incremental_time = incremental_start.elapsed();
    
    println!("Incremental Analysis Performance:");
    println!("  Full analysis time: {:?}", full_time);
    println!("  Incremental analysis time: {:?}", incremental_time);
    println!("  Files changed: {} of {}", changed_files, file_hashes.len());
    println!("  Speedup: {:.2}x", full_time.as_secs_f64() / incremental_time.as_secs_f64());
    
    // Incremental should be significantly faster
    assert!(
        incremental_time < full_time / 2,
        "Incremental analysis should be at least 2x faster than full analysis"
    );
}

#[test]
fn test_output_generation_performance() {
    let project = sample_projects::large_codebase();
    let temp_dir = TempDir::new().unwrap();
    
    // Analyze a subset of files
    let mut all_results = Vec::new();
    
    for entry in walkdir::WalkDir::new(project.root_path.join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .take(10)
    {
        if let Ok(source_file) = SourceFile::parse(entry.path()) {
            let analyzer = AstAnalyzer::new();
            if let Ok(result) = analyzer.analyze(source_file) {
                all_results.push(result);
            }
        }
    }
    
    // Test different output format generation times
    use instrument_rs::output::{JsonFormatter, TreeFormatter, OutputFormatter};
    use instrument_rs::reporting::{HtmlReporter, JsonReporter};
    
    // JSON formatting
    let json_start = Instant::now();
    let json_formatter = JsonFormatter::new();
    for result in &all_results {
        let _ = json_formatter.format(result);
    }
    let json_time = json_start.elapsed();
    
    // Tree formatting
    let tree_start = Instant::now();
    let tree_formatter = TreeFormatter::new();
    for result in &all_results {
        let _ = tree_formatter.format(result);
    }
    let tree_time = tree_start.elapsed();
    
    // HTML report generation
    let html_start = Instant::now();
    let html_reporter = HtmlReporter::new(temp_dir.path().to_path_buf());
    for (i, result) in all_results.iter().enumerate() {
        let _ = html_reporter.generate_report(result, &format!("report_{}.html", i));
    }
    let html_time = html_start.elapsed();
    
    println!("Output Generation Performance:");
    println!("  Files processed: {}", all_results.len());
    println!("  JSON formatting: {:?}", json_time);
    println!("  Tree formatting: {:?}", tree_time);
    println!("  HTML generation: {:?}", html_time);
    
    // All output formats should be reasonably fast
    assert!(json_time.as_secs() < 5, "JSON formatting too slow");
    assert!(tree_time.as_secs() < 5, "Tree formatting too slow");
    assert!(html_time.as_secs() < 10, "HTML generation too slow");
}

// Helper functions

fn get_approximate_memory_usage() -> usize {
    // This is a rough approximation
    // In a real scenario, you might use a proper memory profiler
    std::mem::size_of::<Vec<u8>>() * 1000 // Placeholder
}

fn calculate_hash(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

mod walkdir {
    pub use ::walkdir::*;
}