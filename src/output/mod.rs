//! Output formatters for displaying analysis results in various formats
//!
//! This module provides different output formatters to present the analysis results
//! in human-readable and machine-readable formats.

use crate::Result;
use std::io::Write;
use std::path::Path;

pub mod tree;
pub mod json;
pub mod mermaid;
pub mod traits;
pub mod utils;

#[cfg(test)]
mod test;

pub use traits::{OutputFormatter, FormatterOptions, OutputFormat};
pub use tree::TreeFormatter;
pub use json::JsonFormatter;
pub use mermaid::MermaidFormatter;

/// Factory for creating output formatters
pub struct FormatterFactory;

impl FormatterFactory {
    /// Create a formatter for the specified output format
    ///
    /// # Arguments
    ///
    /// * `format` - The desired output format
    /// * `options` - Formatter options
    ///
    /// # Returns
    ///
    /// A boxed formatter implementing the OutputFormatter trait
    pub fn create(format: OutputFormat, options: FormatterOptions) -> Box<dyn OutputFormatter> {
        match format {
            OutputFormat::Tree => Box::new(TreeFormatter::new(options)),
            OutputFormat::Json => Box::new(JsonFormatter::new(options)),
            OutputFormat::Mermaid => Box::new(MermaidFormatter::new(options)),
        }
    }

    /// Create multiple formatters at once
    ///
    /// # Arguments
    ///
    /// * `formats` - List of desired output formats
    /// * `options` - Formatter options to apply to all formatters
    ///
    /// # Returns
    ///
    /// A vector of boxed formatters
    pub fn create_multiple(
        formats: &[OutputFormat],
        options: FormatterOptions,
    ) -> Vec<Box<dyn OutputFormatter>> {
        formats
            .iter()
            .map(|&format| Self::create(format, options.clone()))
            .collect()
    }
}

/// Helper function to write formatted output to a file or stdout
///
/// # Arguments
///
/// * `output` - The formatted output string
/// * `path` - Optional path to write to (None means stdout)
///
/// # Errors
///
/// Returns an error if writing fails
pub fn write_output(output: &str, path: Option<&Path>) -> Result<()> {
    match path {
        Some(p) => {
            std::fs::write(p, output)?;
            Ok(())
        }
        None => {
            let mut stdout = std::io::stdout();
            stdout.write_all(output.as_bytes())?;
            stdout.flush()?;
            Ok(())
        }
    }
}