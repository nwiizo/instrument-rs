//! JSON output formatter for programmatic processing

use crate::reporting::{CombinedReport, CoverageSummary, FileCoverage};
use crate::mutation::MutationSummary;
use crate::Result;
use super::traits::{OutputFormatter, FormatterOptions, OutputFormat};
use serde::{Serialize, Deserialize};
use serde_json;

/// JSON formatter for machine-readable output
pub struct JsonFormatter {
    options: FormatterOptions,
}

impl JsonFormatter {
    /// Create a new JSON formatter with the given options
    pub fn new(options: FormatterOptions) -> Self {
        Self { options }
    }
    
    /// Format JSON with appropriate indentation
    fn format_json<T: Serialize>(&self, value: &T) -> Result<String> {
        let json = if self.options.compact {
            serde_json::to_string(value)?
        } else {
            serde_json::to_string_pretty(value)?
        };
        Ok(json)
    }
}

/// Wrapper for coverage output with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CoverageOutput {
    /// Format version
    version: String,
    
    /// Generation timestamp
    generated_at: String,
    
    /// Coverage summary
    summary: CoverageSummary,
    
    /// Additional metadata
    metadata: OutputMetadata,
}

/// Wrapper for mutation output with metadata
#[derive(Debug, Serialize, Deserialize)]
struct MutationOutput {
    /// Format version
    version: String,
    
    /// Generation timestamp
    generated_at: String,
    
    /// Mutation summary
    summary: MutationSummary,
    
    /// Additional metadata
    metadata: OutputMetadata,
}

/// Wrapper for combined output with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CombinedOutput {
    /// Format version
    version: String,
    
    /// Generation timestamp
    generated_at: String,
    
    /// Combined report
    report: CombinedReport,
    
    /// Additional metadata
    metadata: OutputMetadata,
}

/// Additional metadata for JSON output
#[derive(Debug, Serialize, Deserialize)]
struct OutputMetadata {
    /// Tool name
    tool: String,
    
    /// Tool version
    tool_version: String,
    
    /// Options used for generation
    options: FormatterOptions,
    
    /// Environment information
    environment: EnvironmentInfo,
}

/// Environment information
#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentInfo {
    /// Operating system
    os: String,
    
    /// Architecture
    arch: String,
    
    /// Rust version
    rust_version: String,
}

impl Default for OutputMetadata {
    fn default() -> Self {
        Self {
            tool: "instrument-rs".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            options: FormatterOptions::default(),
            environment: EnvironmentInfo {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                rust_version: "unknown".to_string(), // Would need rustc_version crate for accurate info
            },
        }
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_coverage(&self, summary: &CoverageSummary) -> Result<String> {
        let output = CoverageOutput {
            version: "1.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            summary: summary.clone(),
            metadata: OutputMetadata {
                options: self.options.clone(),
                ..Default::default()
            },
        };
        
        self.format_json(&output)
    }
    
    fn format_mutations(&self, summary: &MutationSummary) -> Result<String> {
        let output = MutationOutput {
            version: "1.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            summary: summary.clone(),
            metadata: OutputMetadata {
                options: self.options.clone(),
                ..Default::default()
            },
        };
        
        self.format_json(&output)
    }
    
    fn format_combined(&self, report: &CombinedReport) -> Result<String> {
        let output = CombinedOutput {
            version: "1.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            report: report.clone(),
            metadata: OutputMetadata {
                options: self.options.clone(),
                ..Default::default()
            },
        };
        
        self.format_json(&output)
    }
    
    fn format_file_coverage(&self, file: &FileCoverage) -> Result<String> {
        // For single file coverage, we'll create a minimal structure
        #[derive(Serialize)]
        struct FileCoverageOutput {
            file: FileCoverage,
            metadata: FileMetadata,
        }
        
        #[derive(Serialize)]
        struct FileMetadata {
            coverage_percent: f64,
            uncovered_lines: Vec<usize>,
            total_branches: usize,
            covered_branches: usize,
        }
        
        let metadata = FileMetadata {
            coverage_percent: file.line_coverage_percent(),
            uncovered_lines: file.uncovered_lines(),
            total_branches: file.branch_coverage.len() * 2,
            covered_branches: file.branch_coverage.iter()
                .map(|b| {
                    (if b.true_count > 0 { 1 } else { 0 }) +
                    (if b.false_count > 0 { 1 } else { 0 })
                })
                .sum(),
        };
        
        let output = FileCoverageOutput {
            file: file.clone(),
            metadata,
        };
        
        self.format_json(&output)
    }
    
    fn format_type(&self) -> OutputFormat {
        OutputFormat::Json
    }
    
    fn options(&self) -> &FormatterOptions {
        &self.options
    }
}

/// Convenience functions for JSON schema generation
pub mod schema {
    use super::*;
    use serde_json::json;
    
    /// Generate JSON schema for coverage output
    pub fn coverage_schema() -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Coverage Output",
            "type": "object",
            "required": ["version", "generated_at", "summary", "metadata"],
            "properties": {
                "version": {
                    "type": "string",
                    "description": "Schema version"
                },
                "generated_at": {
                    "type": "string",
                    "format": "date-time",
                    "description": "ISO 8601 timestamp"
                },
                "summary": {
                    "type": "object",
                    "description": "Coverage summary data"
                },
                "metadata": {
                    "type": "object",
                    "description": "Generation metadata"
                }
            }
        })
    }
    
    /// Generate JSON schema for mutation output
    pub fn mutation_schema() -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Mutation Output",
            "type": "object",
            "required": ["version", "generated_at", "summary", "metadata"],
            "properties": {
                "version": {
                    "type": "string",
                    "description": "Schema version"
                },
                "generated_at": {
                    "type": "string",
                    "format": "date-time",
                    "description": "ISO 8601 timestamp"
                },
                "summary": {
                    "type": "object",
                    "description": "Mutation testing summary data"
                },
                "metadata": {
                    "type": "object",
                    "description": "Generation metadata"
                }
            }
        })
    }
}