//! Common traits and structures for output formatters

use crate::reporting::{CombinedReport, CoverageSummary, FileCoverage};
use crate::mutation::MutationSummary;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Available output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Human-readable tree view
    Tree,
    /// JSON format for programmatic processing
    Json,
    /// Mermaid diagram for visualization
    Mermaid,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Tree => write!(f, "tree"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Mermaid => write!(f, "mermaid"),
        }
    }
}

/// Options for controlling formatter behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterOptions {
    /// Include detailed file-level information
    pub include_files: bool,
    
    /// Include line-by-line coverage data
    pub include_lines: bool,
    
    /// Include source code snippets
    pub include_source: bool,
    
    /// Use color in output (for terminal formatters)
    pub use_color: bool,
    
    /// Maximum depth for tree views
    pub max_depth: Option<usize>,
    
    /// Sort files by coverage (lowest first)
    pub sort_by_coverage: bool,
    
    /// Only show items below this coverage threshold
    pub coverage_threshold: Option<f64>,
    
    /// Compact output mode
    pub compact: bool,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self {
            include_files: true,
            include_lines: false,
            include_source: false,
            use_color: true,
            max_depth: None,
            sort_by_coverage: false,
            coverage_threshold: None,
            compact: false,
        }
    }
}

/// Trait for output formatters
pub trait OutputFormatter: Send + Sync {
    /// Format a coverage summary
    ///
    /// # Arguments
    ///
    /// * `summary` - The coverage summary to format
    ///
    /// # Returns
    ///
    /// The formatted output as a string
    fn format_coverage(&self, summary: &CoverageSummary) -> Result<String>;
    
    /// Format a mutation summary
    ///
    /// # Arguments
    ///
    /// * `summary` - The mutation summary to format
    ///
    /// # Returns
    ///
    /// The formatted output as a string
    fn format_mutations(&self, summary: &MutationSummary) -> Result<String>;
    
    /// Format a combined report
    ///
    /// # Arguments
    ///
    /// * `report` - The combined report to format
    ///
    /// # Returns
    ///
    /// The formatted output as a string
    fn format_combined(&self, report: &CombinedReport) -> Result<String>;
    
    /// Format a single file's coverage
    ///
    /// # Arguments
    ///
    /// * `file` - The file coverage to format
    ///
    /// # Returns
    ///
    /// The formatted output as a string
    fn format_file_coverage(&self, file: &FileCoverage) -> Result<String>;
    
    /// Get the output format type
    fn format_type(&self) -> OutputFormat;
    
    /// Get the formatter options
    fn options(&self) -> &FormatterOptions;
}

/// Common formatting data structure for internal use
#[derive(Debug, Clone)]
pub struct FormattedNode {
    /// Node label/name
    pub label: String,
    
    /// Node value (e.g., coverage percentage)
    pub value: Option<String>,
    
    /// Node type for styling
    pub node_type: NodeType,
    
    /// Child nodes
    pub children: Vec<FormattedNode>,
    
    /// Additional metadata
    pub metadata: NodeMetadata,
}

/// Type of node for styling purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Root/project node
    Root,
    /// Directory node
    Directory,
    /// File node
    File,
    /// Function/method node
    Function,
    /// Line node
    Line,
    /// Summary node
    Summary,
    /// Error node
    Error,
}

/// Additional metadata for nodes
#[derive(Debug, Clone, Default)]
pub struct NodeMetadata {
    /// Coverage percentage (0-100)
    pub coverage: Option<f64>,
    
    /// Number of mutations
    pub mutations: Option<usize>,
    
    /// Number of survived mutations
    pub survived_mutations: Option<usize>,
    
    /// Line numbers for line nodes
    pub line_range: Option<(usize, usize)>,
    
    /// Path for file/directory nodes
    pub path: Option<String>,
    
    /// Whether this node represents a test file
    pub is_test: bool,
    
    /// Custom key-value pairs
    pub custom: std::collections::HashMap<String, String>,
}

impl FormattedNode {
    /// Create a new formatted node
    pub fn new(label: String, node_type: NodeType) -> Self {
        Self {
            label,
            value: None,
            node_type,
            children: Vec::new(),
            metadata: NodeMetadata::default(),
        }
    }
    
    /// Add a child node
    pub fn add_child(&mut self, child: FormattedNode) {
        self.children.push(child);
    }
    
    /// Set the node value
    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }
    
    /// Set the coverage percentage
    pub fn with_coverage(mut self, coverage: f64) -> Self {
        self.metadata.coverage = Some(coverage);
        self
    }
    
    /// Sort children by coverage (ascending)
    pub fn sort_by_coverage(&mut self) {
        self.children.sort_by(|a, b| {
            let a_cov = a.metadata.coverage.unwrap_or(100.0);
            let b_cov = b.metadata.coverage.unwrap_or(100.0);
            a_cov.partial_cmp(&b_cov).unwrap()
        });
        
        // Recursively sort children
        for child in &mut self.children {
            child.sort_by_coverage();
        }
    }
    
    /// Filter nodes by coverage threshold
    pub fn filter_by_coverage(&mut self, threshold: f64) {
        self.children.retain(|child| {
            child.metadata.coverage.map_or(true, |cov| cov < threshold)
        });
        
        // Recursively filter children
        for child in &mut self.children {
            child.filter_by_coverage(threshold);
        }
    }
}