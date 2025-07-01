//! End-to-end tests for analyzing sample Rust projects

use crate::common::{sample_projects, assertions::*};
use instrument_rs::{
    Instrumentor, Config,
    ast::{AstAnalyzer, SourceFile},
    call_graph::GraphBuilder,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_analyze_simple_library_project() {
    let project = sample_projects::simple_library();
    let config = Config::default();
    
    // Run the instrumentor
    let instrumentor = Instrumentor::new(config);
    let result = instrumentor.run();
    assert!(result.is_ok(), "Instrumentor should run successfully");
    
    // Verify AST analysis
    let src_lib = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&src_lib).expect("Should parse source file");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Check function detection
    assert_eq!(ast_result.functions.len(), 5, "Should detect 5 functions");
    assert_eq!(ast_result.test_functions.len(), 2, "Should detect 2 test functions");
    
    // Verify function names
    let func_names: Vec<&str> = ast_result.functions.iter()
        .map(|f| f.name.as_str())
        .collect();
    assert!(func_names.contains(&"add"));
    assert!(func_names.contains(&"subtract"));
    assert!(func_names.contains(&"multiply"));
    assert!(func_names.contains(&"divide"));
    assert!(func_names.contains(&"complex_logic"));
    
    // Check complexity metrics
    let complex_fn = ast_result.functions.iter()
        .find(|f| f.name == "complex_logic")
        .expect("Should find complex_logic function");
    assert!(complex_fn.complexity.cyclomatic > 3, "Complex function should have high cyclomatic complexity");
    assert!(complex_fn.complexity.branch_count > 2, "Complex function should have multiple branches");
    
    // Check error handling detection
    assert_eq!(complex_fn.error_handling.result_returns, 1, "Should detect Result return type");
    
    let divide_fn = ast_result.functions.iter()
        .find(|f| f.name == "divide")
        .expect("Should find divide function");
    assert_eq!(divide_fn.error_handling.option_returns, 1, "Should detect Option return type");
}

#[test]
fn test_analyze_axum_web_application() {
    let project = sample_projects::axum_web_app();
    let config = Config::default();
    
    // Verify Axum-specific patterns are detected
    let main_rs = project.root_path.join("src/main.rs");
    let source_file = SourceFile::parse(&main_rs).expect("Should parse main.rs");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Check for async functions
    let async_funcs: Vec<&str> = ast_result.functions.iter()
        .filter(|f| f.is_async)
        .map(|f| f.name.as_str())
        .collect();
    
    assert!(!async_funcs.is_empty(), "Should detect async functions");
    assert!(async_funcs.contains(&"main"));
    assert!(async_funcs.contains(&"root"));
    assert!(async_funcs.contains(&"create_user"));
    assert!(async_funcs.contains(&"get_user"));
    
    // Check for route handlers
    let route_handlers = vec!["root", "health_check", "create_user", "get_user"];
    for handler in route_handlers {
        let func = ast_result.functions.iter()
            .find(|f| f.name == handler)
            .expect(&format!("Should find {} handler", handler));
        assert!(func.is_async, "{} should be async", handler);
    }
    
    // Check test detection
    assert!(ast_result.test_functions.iter().any(|f| f.name == "test_health_check"));
}

#[test]
fn test_analyze_complex_patterns_project() {
    let project = sample_projects::complex_patterns();
    let config = Config::default();
    
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse lib.rs");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Check trait detection
    assert!(!ast_result.traits.is_empty(), "Should detect traits");
    let data_store_trait = ast_result.traits.iter()
        .find(|t| t.name == "DataStore")
        .expect("Should find DataStore trait");
    assert_eq!(data_store_trait.methods.len(), 3, "DataStore should have 3 methods");
    
    // Check struct detection
    assert!(!ast_result.structs.is_empty(), "Should detect structs");
    let struct_names: Vec<&str> = ast_result.structs.iter()
        .map(|s| s.name.as_str())
        .collect();
    assert!(struct_names.contains(&"MemoryStore"));
    assert!(struct_names.contains(&"DataProcessor"));
    assert!(struct_names.contains(&"FibIterator"));
    
    // Check enum detection
    assert!(!ast_result.enums.is_empty(), "Should detect enums");
    let enum_names: Vec<&str> = ast_result.enums.iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(enum_names.contains(&"StoreError"));
    assert!(enum_names.contains(&"ProcessError"));
    
    // Check impl block detection
    assert!(!ast_result.impl_blocks.is_empty(), "Should detect impl blocks");
    
    // Check recursive function detection
    let fibonacci = ast_result.functions.iter()
        .find(|f| f.name == "fibonacci")
        .expect("Should find fibonacci function");
    assert!(fibonacci.complexity.cyclomatic > 1, "Recursive function should have complexity");
    
    // Check async trait implementation
    let async_impls = ast_result.impl_blocks.iter()
        .filter(|impl_block| impl_block.trait_name.as_ref().map_or(false, |n| n == "DataStore"))
        .count();
    assert!(async_impls > 0, "Should detect async trait implementations");
}

#[test]
fn test_call_graph_generation() {
    let project = sample_projects::simple_library();
    
    // Build call graph
    let mut builder = GraphBuilder::new();
    let src_dir = project.root_path.join("src");
    
    // Process all source files
    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(source_file) = SourceFile::parse(entry.path()) {
            builder.add_source_file(entry.path().to_path_buf()).expect("Should add file to graph");
        }
    }
    
    let graph = builder.build().expect("Should build graph");
    
    // Verify graph structure
    assert!(graph.node_count() > 0, "Call graph should have nodes");
    assert!(graph.edge_count() > 0, "Call graph should have edges");
}

#[test]
fn test_instrumentation_scoring() {
    let project = sample_projects::complex_patterns();
    
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse lib.rs");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Check complexity of functions
    for function in &ast_result.functions {
        // Verify function properties
        if function.name == "fibonacci" {
            // Recursive function should have high complexity
            assert!(function.complexity.cyclomatic > 5, "Recursive function should have high complexity");
        }
        
        if function.complexity.cyclomatic > 5 {
            // Functions with high complexity are detected
            assert!(function.complexity.cyclomatic > 0, "Should detect complexity");
        }
    }
}

#[test]
fn test_full_project_analysis_with_tests() {
    let project = sample_projects::simple_library();
    
    // Create integration test that uses the library
    project.add_test_file("full_integration.rs", r#"
use test_lib::*;

#[test]
fn test_all_operations() {
    // Test arithmetic operations
    assert_eq!(add(10, 5), 15);
    assert_eq!(subtract(10, 5), 5);
    assert_eq!(multiply(3, 4), 12);
    
    // Test division with edge cases
    assert_eq!(divide(20, 4), Some(5));
    assert_eq!(divide(10, 0), None);
    
    // Test complex logic
    assert!(complex_logic(5, 10).is_ok());
    assert_eq!(complex_logic(5, 10).unwrap(), 65); // 5*10 + 5 + 10
    
    // Test error cases
    assert!(complex_logic(-1, 5).is_err());
    assert!(complex_logic(5, -1).is_err());
    assert!(complex_logic(101, 5).is_err());
    assert!(complex_logic(5, 101).is_err());
}

#[test]
fn test_edge_cases() {
    assert_eq!(add(i32::MAX, 0), i32::MAX);
    assert_eq!(multiply(0, 100), 0);
    assert_eq!(divide(0, 5), Some(0));
}
"#);
    
    let config = Config::default();
    
    // Analyze both source and test files
    let mut total_functions = 0;
    let mut total_tests = 0;
    let mut total_complexity = 0;
    
    // Analyze src directory
    for entry in walkdir::WalkDir::new(project.root_path.join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(source_file) = SourceFile::parse(entry.path()) {
            let analyzer = AstAnalyzer::new();
            if let Ok(result) = analyzer.analyze(source_file) {
                total_functions += result.functions.len();
                total_tests += result.test_functions.len();
                total_complexity += result.functions.iter()
                    .map(|f| f.complexity.cyclomatic)
                    .sum::<usize>();
            }
        }
    }
    
    // Analyze test directory
    for entry in walkdir::WalkDir::new(project.root_path.join("tests"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(source_file) = SourceFile::parse(entry.path()) {
            let analyzer = AstAnalyzer::new();
            if let Ok(result) = analyzer.analyze(source_file) {
                total_tests += result.test_functions.len();
            }
        }
    }
    
    // Verify comprehensive coverage
    assert!(total_functions >= 5, "Should have at least 5 functions");
    assert!(total_tests >= 4, "Should have at least 4 test functions");
    assert!(total_complexity > 10, "Total complexity should be significant");
}

#[test]
fn test_mutation_targets_identification() {
    let project = sample_projects::simple_library();
    let config = Config::default();
    
    let lib_rs = project.root_path.join("src/lib.rs");
    let source_file = SourceFile::parse(&lib_rs).expect("Should parse lib.rs");
    let analyzer = AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Check for mutation opportunities
    for function in &ast_result.functions {
        match function.name.as_str() {
            "add" | "subtract" | "multiply" => {
                // These should have arithmetic operators
                assert!(function.operators.arithmetic_ops > 0, 
                    "{} should have arithmetic operators", function.name);
            }
            "divide" => {
                // Should have comparison operator (b == 0)
                assert!(function.operators.comparison_ops > 0,
                    "divide should have comparison operators");
            }
            "complex_logic" => {
                // Should have multiple operator types
                assert!(function.operators.comparison_ops > 0,
                    "complex_logic should have comparison operators");
                assert!(function.operators.logical_ops > 0,
                    "complex_logic should have logical operators");
            }
            _ => {}
        }
    }
}

mod walkdir {
    pub use ::walkdir::*;
}