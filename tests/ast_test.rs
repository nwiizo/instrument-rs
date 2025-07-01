#[cfg(test)]
mod ast_integration_tests {
    use instrument_rs::ast::{AstAnalyzer, SourceFile};
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_basic_ast_analysis() {
        let source = r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn complex_function(x: i32) -> Result<i32, String> {
    if x > 0 {
        if x > 10 {
            Ok(x * 2)
        } else {
            Ok(x)
        }
    } else {
        Err("negative value".to_string())
    }
}

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}
"#;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, source).unwrap();
        
        let source_file = SourceFile::parse(&file_path).unwrap();
        let analyzer = AstAnalyzer::new();
        let result = analyzer.analyze(source_file).unwrap();
        
        // Check functions
        assert_eq!(result.functions.len(), 3);
        assert_eq!(result.test_functions.len(), 1);
        
        // Check function names
        let func_names: Vec<&str> = result.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(func_names.contains(&"main"));
        assert!(func_names.contains(&"add"));
        assert!(func_names.contains(&"complex_function"));
        
        // Check test function
        assert_eq!(result.test_functions[0].name, "test_add");
        
        // Check complexity
        let complex_fn = result.functions.iter().find(|f| f.name == "complex_function").unwrap();
        assert!(complex_fn.complexity.branch_count > 0);
        assert!(complex_fn.complexity.cyclomatic > 1);
        
        // Check error handling
        assert_eq!(complex_fn.error_handling.result_returns, 1);
        
        println!("Analysis complete:");
        println!("Functions: {}", result.functions.len());
        println!("Test functions: {}", result.test_functions.len());
        println!("Elements: {}", result.elements.len());
    }
}