//! Example: Analyze a Rust source file with the AST analyzer

use instrument_rs::ast::{AstAnalyzer, SourceFile};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <rust_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    // Parse the source file
    println!("Parsing {}...", file_path);
    let source_file = SourceFile::parse(file_path)?;

    // Analyze the AST
    let analyzer = AstAnalyzer::new();
    let result = analyzer.analyze(source_file)?;

    // Print analysis results
    println!("\n=== Analysis Results ===");
    println!("Functions: {}", result.functions.len());
    println!("Test functions: {}", result.test_functions.len());
    println!("Modules: {}", result.modules.len());
    println!("Total elements: {}", result.elements.len());

    println!("\n=== Functions ===");
    for func in &result.functions {
        println!("\n{} ({})", func.name, func.full_path);
        println!("  - Parameters: {}", func.param_count);
        println!("  - Async: {}", func.is_async);
        println!("  - Generic: {}", func.is_generic);
        println!(
            "  - Return type: {}",
            func.return_type.as_ref().unwrap_or(&"()".to_string())
        );
        println!("  - Complexity:");
        println!("    - Cyclomatic: {}", func.complexity.cyclomatic);
        println!("    - Lines of code: {}", func.complexity.lines_of_code);
        println!("    - Max nesting: {}", func.complexity.max_nesting_depth);
        println!("  - Error handling:");
        println!(
            "    - Result returns: {}",
            func.error_handling.result_returns
        );
        println!("    - unwrap() calls: {}", func.error_handling.unwrap_calls);
        println!(
            "    - ? operators: {}",
            func.error_handling.question_mark_ops
        );

        if !func.calls.is_empty() {
            println!("  - Calls:");
            for call in &func.calls {
                println!(
                    "    - {} ({})",
                    call.callee,
                    if call.is_method { "method" } else { "function" }
                );
            }
        }
    }

    println!("\n=== Test Functions ===");
    for test in &result.test_functions {
        println!("- {} ({})", test.name, test.full_path);
    }

    Ok(())
}
