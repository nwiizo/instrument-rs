//! Error types for the instrument-rs library
//!
//! This module provides error types for code analysis and instrumentation detection.

use thiserror::Error;

/// The main error type for instrument-rs operations
#[derive(Error, Debug)]
pub enum Error {
    /// I/O related errors (file reading, writing, etc.)
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// AST parsing and syntax errors
    #[error("parse error: {0}")]
    Parse(String),

    /// Configuration errors
    #[error("configuration error: {0}")]
    Config(String),

    /// Framework detection errors
    #[error("framework detection error: {0}")]
    FrameworkDetection(String),

    /// Call graph construction errors
    #[error("call graph error: {0}")]
    CallGraph(String),

    /// Pattern matching errors
    #[error("pattern matching error: {0}")]
    PatternMatching(String),

    /// Serialization/deserialization errors
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML parsing errors
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Generic errors
    #[error("{0}")]
    Generic(String),
}

impl Error {
    /// Create a parse error with the given message
    #[must_use]
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    /// Create a call graph error with the given message
    #[must_use]
    pub fn call_graph(message: impl Into<String>) -> Self {
        Self::CallGraph(message.into())
    }

    /// Create a framework detection error with the given message
    #[must_use]
    pub fn framework(message: impl Into<String>) -> Self {
        Self::FrameworkDetection(message.into())
    }

    /// Check if the error is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Io(e) => {
                matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::Interrupted
                        | std::io::ErrorKind::TimedOut
                )
            }
            _ => false,
        }
    }
}

/// Type alias for Results using our Error type
pub type Result<T> = std::result::Result<T, Error>;

// Implement From for GraphBuildError
impl From<crate::call_graph::GraphBuildError> for Error {
    fn from(err: crate::call_graph::GraphBuildError) -> Self {
        Error::CallGraph(err.to_string())
    }
}
