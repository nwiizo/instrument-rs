#[cfg(test)]
mod test {
    use instrument_rs::ast::{AstAnalyzer, SourceFile};
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_ast_analyzer() {
        let source = r#"
fn main() {
    println!("Hello");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, source).unwrap();
        
        let source_file = SourceFile::parse(&file_path).unwrap();
        let analyzer = AstAnalyzer::new();
        let result = analyzer.analyze(source_file).unwrap();
        
        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].name, "main");
        assert_eq!(result.functions[1].name, "add");
        assert_eq!(result.functions[1].param_count, 2);
        
        println!("Test passed! Found {} functions", result.functions.len());
    }
}