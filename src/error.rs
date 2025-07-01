//! Error types for the instrument-rs library

use thiserror::Error;

/// The main error type for instrument-rs operations
#[derive(Error, Debug)]
pub enum Error {
    /// IO-related errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parsing errors when analyzing Rust code
    #[error("Parse error: {0}")]
    Parse(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Instrumentation errors
    #[error("Instrumentation error: {0}")]
    Instrumentation(String),

    /// Framework detection errors
    #[error("Framework detection error: {0}")]
    FrameworkDetection(String),

    /// Mutation generation errors
    #[error("Mutation error: {0}")]
    Mutation(String),

    /// Reporting errors
    #[error("Reporting error: {0}")]
    Reporting(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML parsing errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Generic errors
    #[error("{0}")]
    Generic(String),
}

/// Type alias for Results using our Error type
pub type Result<T> = std::result::Result<T, Error>;
