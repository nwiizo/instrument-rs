//! Output formatter traits and types

use crate::AnalysisResult;
use crate::Result;

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable tree format
    #[default]
    Tree,
    /// JSON format
    Json,
    /// Mermaid diagram format
    Mermaid,
}

/// Options for output formatters
#[derive(Debug, Clone, Default)]
pub struct FormatterOptions {
    /// Whether to use colors in output
    pub use_colors: bool,
    /// Maximum depth for tree output
    pub max_depth: Option<usize>,
    /// Whether to include source snippets
    pub include_source: bool,
    /// Minimum priority to display
    pub min_priority: Option<u8>,
}

/// Trait for output formatters
pub trait OutputFormatter: Send + Sync {
    /// Format the analysis result
    fn format(&self, result: &AnalysisResult) -> Result<String>;

    /// Get the output format type
    fn format_type(&self) -> OutputFormat;
}
