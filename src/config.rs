//! Configuration structures for instrument-rs
//!
//! This module defines configuration options for analyzing Rust code
//! and detecting optimal instrumentation points for observability.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The main configuration structure for instrument-rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Detection threshold (0.0-1.0) for identifying instrumentation points
    #[serde(default = "default_threshold")]
    pub threshold: f64,

    /// Maximum call graph depth to trace
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,

    /// Include test functions in analysis
    #[serde(default)]
    pub include_tests: bool,

    /// Web framework to use for endpoint detection
    #[serde(default)]
    pub framework: FrameworkType,

    /// Custom patterns file path
    #[serde(default)]
    pub patterns_file: Option<PathBuf>,

    /// Patterns to exclude from analysis
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,

    /// Source directories to analyze (relative to root)
    #[serde(default = "default_source_dirs")]
    pub source_dirs: Vec<PathBuf>,

    /// Naming convention rules
    #[serde(default)]
    pub naming_rules: NamingRules,
}

/// Naming convention rules for instrumentation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamingRules {
    /// Required span name prefix for endpoints (e.g., "api." or "web.")
    #[serde(default)]
    pub endpoint_prefix: Option<String>,

    /// Required span name prefix for database operations (e.g., "db.")
    #[serde(default)]
    pub database_prefix: Option<String>,

    /// Required span name prefix for external API calls (e.g., "ext.")
    #[serde(default)]
    pub external_prefix: Option<String>,

    /// Required span name prefix for cache operations (e.g., "cache.")
    #[serde(default)]
    pub cache_prefix: Option<String>,

    /// Required attributes for endpoint handlers
    #[serde(default)]
    pub required_endpoint_attrs: Vec<String>,

    /// Required attributes for database operations
    #[serde(default)]
    pub required_database_attrs: Vec<String>,

    /// Forbidden patterns in span names (e.g., passwords, tokens)
    #[serde(default)]
    pub forbidden_patterns: Vec<String>,
}

fn default_threshold() -> f64 {
    0.8
}

fn default_max_depth() -> usize {
    10
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        "target".to_string(),
        "node_modules".to_string(),
        ".git".to_string(),
    ]
}

fn default_source_dirs() -> Vec<PathBuf> {
    vec![PathBuf::from("src")]
}

/// Web framework type for endpoint detection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum FrameworkType {
    /// Auto-detect framework from dependencies
    #[default]
    Auto,
    /// Axum web framework
    Axum,
    /// Actix-web framework
    Actix,
    /// Rocket framework
    Rocket,
    /// Tonic gRPC framework
    Tonic,
}

/// Output format for analysis results
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable tree output with colors
    #[default]
    Human,
    /// JSON output for programmatic consumption
    Json,
    /// Mermaid diagram format for visualization
    Mermaid,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threshold: default_threshold(),
            max_depth: default_max_depth(),
            include_tests: false,
            framework: FrameworkType::Auto,
            patterns_file: None,
            exclude_patterns: default_exclude_patterns(),
            source_dirs: default_source_dirs(),
            naming_rules: NamingRules::default(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to save the configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::Config(format!("Failed to serialize config: {e}")))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl FrameworkType {
    /// Get the display name of the framework
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Axum => "axum",
            Self::Actix => "actix-web",
            Self::Rocket => "rocket",
            Self::Tonic => "tonic",
        }
    }
}
