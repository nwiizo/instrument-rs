//! AST analysis and manipulation utilities
//!
//! This module provides the foundation for all code analysis in instrument-rs.
//! It wraps the `syn` crate to parse Rust source code and provides utilities
//! for traversing and analyzing the Abstract Syntax Tree (AST).
//!
//! # Overview
//!
//! The AST module is responsible for:
//! - Parsing Rust source files into ASTs
//! - Providing visitor patterns for tree traversal
//! - Extracting information about functions, types, and other code elements
//! - Maintaining source location information for accurate reporting
//!
//! # Example
//!
//! ```no_run
//! use instrument_rs::ast::SourceFile;
//! use std::path::PathBuf;
//!
//! // Parse a Rust source file
//! let path = PathBuf::from("src/main.rs");
//! let source = std::fs::read_to_string(&path).unwrap();
//! let ast = syn::parse_file(&source).unwrap();
//!
//! let source_file = SourceFile::new(path, ast, source);
//! ```

use std::path::PathBuf;
use syn::File;

pub mod analyzer;
pub mod helpers;
pub mod visitor;

/// Represents a parsed Rust source file with metadata
///
/// This struct combines the parsed AST with additional metadata needed
/// for analysis and reporting. It maintains the relationship between
/// the AST nodes and their source locations.
///
/// # Example
///
/// ```no_run
/// # use instrument_rs::ast::SourceFile;
/// # use std::path::PathBuf;
/// # let source = String::new();
/// # let ast = syn::parse_file(&source).unwrap();
/// let source_file = SourceFile {
///     path: PathBuf::from("src/lib.rs"),
///     syntax_tree: ast,
///     source: source.clone(),
///     content_hash: SourceFile::calculate_hash(&source),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Path to the source file
    pub path: PathBuf,

    /// The parsed AST
    pub syntax_tree: File,

    /// Original source code
    pub source: String,

    /// SHA256 hash of the source content
    pub content_hash: String,
}

impl SourceFile {
    /// Creates a new SourceFile instance
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the source file
    /// * `syntax_tree` - Parsed AST
    /// * `source` - Original source code
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use instrument_rs::ast::SourceFile;
    /// # use std::path::PathBuf;
    /// # let source = String::new();
    /// # let ast = syn::parse_file(&source).unwrap();
    /// let source_file = SourceFile::new(
    ///     PathBuf::from("src/main.rs"),
    ///     ast,
    ///     source
    /// );
    /// ```
    pub fn new(path: PathBuf, syntax_tree: File, source: String) -> Self {
        let content_hash = Self::calculate_hash(&source);
        Self {
            path,
            syntax_tree,
            source,
            content_hash,
        }
    }

    /// Calculates SHA256 hash of the source content
    ///
    /// This is used for cache invalidation and change detection.
    ///
    /// # Arguments
    ///
    /// * `content` - Source code to hash
    ///
    /// # Returns
    ///
    /// Hexadecimal string representation of the SHA256 hash
    pub fn calculate_hash(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Information about a code element that can be instrumented
#[derive(Debug, Clone)]
pub struct InstrumentableElement {
    /// Unique identifier for this element
    pub id: String,

    /// Type of the element
    pub kind: ElementKind,

    /// Location in the source file
    pub location: Location,

    /// Parent element ID (if any)
    pub parent_id: Option<String>,

    /// Whether this element is in test code
    pub is_test: bool,
}

impl Default for InstrumentableElement {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: ElementKind::Function,
            location: Location::point(1, 1),
            parent_id: None,
            is_test: false,
        }
    }
}

/// Types of code elements that can be instrumented
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementKind {
    /// Function or method
    Function,
    /// Closure or lambda
    Closure,
    /// If expression
    IfExpression,
    /// Match expression
    MatchExpression,
    /// Match arm
    MatchArm,
    /// Loop (for, while, loop)
    Loop,
    /// Return statement
    Return,
    /// Expression statement
    Statement,
    /// Assignment
    Assignment,
    /// Binary operation
    BinaryOp,
    /// Unary operation
    UnaryOp,
    /// Method call
    MethodCall,
    /// Function call
    FunctionCall,
}

/// Location information for code elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Starting line (1-indexed)
    pub start_line: usize,

    /// Starting column (1-indexed)
    pub start_column: usize,

    /// Ending line (1-indexed)
    pub end_line: usize,

    /// Ending column (1-indexed)
    pub end_column: usize,
}

/// Analysis result for a source file
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// The analyzed source file
    pub source_file: SourceFile,

    /// All instrumentable elements found
    pub elements: Vec<InstrumentableElement>,

    /// Function definitions
    pub functions: Vec<FunctionInfo>,

    /// Test functions
    pub test_functions: Vec<FunctionInfo>,

    /// Module structure
    pub modules: Vec<ModuleInfo>,
}

/// Information about a function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Element ID
    pub id: String,

    /// Function name
    pub name: String,

    /// Full path to the function (module::function)
    pub full_path: String,

    /// Whether it's async
    pub is_async: bool,

    /// Whether it's unsafe
    pub is_unsafe: bool,

    /// Whether it's a test function
    pub is_test: bool,

    /// Whether it's generic
    pub is_generic: bool,

    /// Number of parameters
    pub param_count: usize,

    /// Return type (as string)
    pub return_type: Option<String>,

    /// Functions called from this function
    pub calls: Vec<CallInfo>,

    /// Error handling patterns detected
    pub error_handling: ErrorHandlingInfo,

    /// Complexity metrics
    pub complexity: ComplexityMetrics,

    /// Location in source
    pub location: Location,
}

/// Information about a module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Module name
    pub name: String,

    /// Full path to the module
    pub path: Vec<String>,

    /// Whether it's a test module
    pub is_test: bool,

    /// Location in source
    pub location: Location,
}

impl SourceFile {
    /// Parse a Rust source file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the source file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn parse(path: impl Into<PathBuf>) -> crate::Result<Self> {
        let path = path.into();
        let source = std::fs::read_to_string(&path)?;
        let syntax_tree = syn::parse_file(&source).map_err(|e| {
            crate::Error::Parse(format!("Failed to parse {}: {}", path.display(), e))
        })?;

        let content_hash = Self::calculate_hash(&source);

        Ok(Self {
            path,
            syntax_tree,
            source,
            content_hash,
        })
    }

}

/// Information about a function call
#[derive(Debug, Clone)]
pub struct CallInfo {
    /// Name of the called function
    pub callee: String,

    /// Whether it's a method call
    pub is_method: bool,

    /// Location of the call
    pub location: Location,
}

/// Error handling patterns detected in a function
#[derive(Debug, Clone, Default)]
pub struct ErrorHandlingInfo {
    /// Number of Result returns
    pub result_returns: usize,

    /// Number of Option returns
    pub option_returns: usize,

    /// Number of unwrap calls
    pub unwrap_calls: usize,

    /// Number of expect calls
    pub expect_calls: usize,

    /// Number of ? operators
    pub question_mark_ops: usize,

    /// Number of match expressions on Result/Option
    pub error_matches: usize,

    /// Number of if let expressions on Result/Option
    pub error_if_lets: usize,
}

/// Complexity metrics for a function
#[derive(Debug, Clone, Default)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity
    pub cyclomatic: usize,

    /// Cognitive complexity
    pub cognitive: usize,

    /// Lines of code (excluding comments and blank lines)
    pub lines_of_code: usize,

    /// Number of statements
    pub statement_count: usize,

    /// Maximum nesting depth
    pub max_nesting_depth: usize,

    /// Number of branches (if, match, etc.)
    pub branch_count: usize,

    /// Number of loops
    pub loop_count: usize,
}

impl Location {
    /// Create a new location
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }

    /// Create a location for a single point
    pub fn point(line: usize, column: usize) -> Self {
        Self {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column,
        }
    }

    /// Create a location from a syn span (if line/column info available)
    pub fn from_span(span: proc_macro2::Span) -> Option<Self> {
        // Note: This requires the "proc-macro2" feature with span locations
        // In practice, we'll need to use the source map to get accurate locations
        None // Placeholder for now
    }
}

// Re-export commonly used items from submodules
pub use analyzer::AstAnalyzer;
pub use helpers::{AnalysisStats, CallGraphAnalyzer, ComplexityAnalyzer, FunctionFilter};
pub use visitor::analyze_ast;
