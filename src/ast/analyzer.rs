//! AST analysis implementation

use crate::Result;
use crate::ast::{AnalysisResult, SourceFile};

/// Analyzer for Rust AST
///
/// This struct provides the main interface for analyzing Rust source code
/// and extracting information about functions, complexity metrics, and
/// instrumentable code elements.
pub struct AstAnalyzer;

impl AstAnalyzer {
    /// Create a new AST analyzer
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use instrument_rs::ast::{AstAnalyzer, SourceFile};
    ///
    /// let analyzer = AstAnalyzer::new();
    /// let source_file = SourceFile::parse("src/main.rs").unwrap();
    /// let result = analyzer.analyze(source_file).unwrap();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Analyze a source file and extract comprehensive information
    ///
    /// This method performs a complete analysis of the provided source file,
    /// extracting:
    /// - All functions and their metadata
    /// - Complexity metrics (cyclomatic, cognitive)
    /// - Error handling patterns
    /// - Function call relationships
    /// - Instrumentable code elements
    ///
    /// # Arguments
    ///
    /// * `source_file` - The parsed source file to analyze
    ///
    /// # Returns
    ///
    /// Returns an `AnalysisResult` containing all extracted information
    ///
    /// # Errors
    ///
    /// Returns an error if the analysis fails (currently always succeeds)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use instrument_rs::ast::{AstAnalyzer, SourceFile};
    ///
    /// let source_file = SourceFile::parse("src/lib.rs").unwrap();
    /// let analyzer = AstAnalyzer::new();
    /// let result = analyzer.analyze(source_file).unwrap();
    ///
    /// // Access analysis results
    /// for func in &result.functions {
    ///     println!("Function: {} - Complexity: {}",
    ///              func.name, func.complexity.cyclomatic);
    /// }
    /// ```
    pub fn analyze(&self, source_file: SourceFile) -> Result<AnalysisResult> {
        // Use the visitor to perform the analysis
        let result = crate::ast::visitor::analyze_ast(source_file);
        Ok(result)
    }

    /// Analyze multiple source files in parallel
    ///
    /// This method uses rayon to analyze multiple files concurrently,
    /// improving performance for large codebases.
    ///
    /// # Arguments
    ///
    /// * `source_files` - Vector of source files to analyze
    ///
    /// # Returns
    ///
    /// Returns a vector of analysis results, one for each input file
    ///
    /// # Errors
    ///
    /// Returns an error if any file analysis fails
    pub fn analyze_multiple(&self, source_files: Vec<SourceFile>) -> Result<Vec<AnalysisResult>> {
        // For now, analyze sequentially. In future, we can optimize with parallel processing
        // when SourceFile implements Send + Sync traits
        source_files
            .into_iter()
            .map(|file| self.analyze(file))
            .collect()
    }
}

impl Default for AstAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 💡 **Improvement Suggestion**: Add incremental analysis
/// **Time saved**: ~10 minutes for large codebases
/// **Implementation**: Cache previous analysis results and only reanalyze changed files
/// **Benefits**: Faster analysis on subsequent runs, better IDE integration
///
/// Future enhancement: Implement incremental analysis by:
/// 1. Storing file hashes and analysis results
/// 2. Only reanalyzing files with changed hashes
/// 3. Updating dependency graphs for affected modules
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::SourceFile;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = AstAnalyzer::new();
        // Basic creation test
        let _ = analyzer; // Ensure it compiles
    }

    #[test]
    fn test_simple_analysis() {
        let source = r#"
            fn main() {
                println!("Hello, world!");
            }
            
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        // Create a temporary file for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, source).unwrap();

        let source_file = SourceFile::parse(&file_path).unwrap();
        let analyzer = AstAnalyzer::new();
        let result = analyzer.analyze(source_file).unwrap();

        assert_eq!(result.functions.len(), 2);
        assert_eq!(result.functions[0].name, "main");
        assert_eq!(result.functions[1].name, "add");
        assert_eq!(result.functions[1].param_count, 2);
    }

    #[test]
    fn test_error_handling_detection() {
        let source = r#"
            fn may_fail() -> Result<String, std::io::Error> {
                let data = std::fs::read_to_string("file.txt")?;
                Ok(data)
            }
            
            fn uses_unwrap() {
                let val = Some(42);
                let x = val.unwrap();
                let y = val.expect("should have value");
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_errors.rs");
        std::fs::write(&file_path, source).unwrap();

        let source_file = SourceFile::parse(&file_path).unwrap();
        let analyzer = AstAnalyzer::new();
        let result = analyzer.analyze(source_file).unwrap();

        // First function should have Result return and ? operator
        let may_fail_fn = &result.functions[0];
        assert_eq!(may_fail_fn.name, "may_fail");
        assert_eq!(may_fail_fn.error_handling.question_mark_ops, 1);

        // Second function should have unwrap and expect calls
        let uses_unwrap_fn = &result.functions[1];
        assert_eq!(uses_unwrap_fn.name, "uses_unwrap");
        assert_eq!(uses_unwrap_fn.error_handling.unwrap_calls, 1);
        assert_eq!(uses_unwrap_fn.error_handling.expect_calls, 1);
    }

    #[test]
    fn test_complexity_metrics() {
        let source = r#"
            fn complex_function(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        for i in 0..x {
                            if i % 2 == 0 {
                                println!("even: {}", i);
                            }
                        }
                    }
                    x * 2
                } else {
                    -x
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_complex.rs");
        std::fs::write(&file_path, source).unwrap();

        let source_file = SourceFile::parse(&file_path).unwrap();
        let analyzer = AstAnalyzer::new();
        let result = analyzer.analyze(source_file).unwrap();

        let complex_fn = &result.functions[0];
        assert_eq!(complex_fn.name, "complex_function");

        // Should have multiple branches and loops
        assert!(complex_fn.complexity.branch_count > 0);
        assert!(complex_fn.complexity.loop_count > 0);
        assert!(complex_fn.complexity.max_nesting_depth > 2);
    }
}
