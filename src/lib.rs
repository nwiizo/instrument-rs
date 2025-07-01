//! instrument-rs: A comprehensive Rust library for code instrumentation and analysis
//!
//! This library provides powerful tools for analyzing Rust codebases and automatically
//! instrumenting them for observability, testing, and quality metrics.
//!
//! # Features
//!
//! - **AST-based Analysis**: Deep code analysis using Rust's syntax tree
//! - **Call Graph Construction**: Build and analyze function call relationships
//! - **Pattern Recognition**: Identify code patterns and critical paths
//! - **Framework Detection**: Auto-detect web and testing frameworks
//! - **Mutation Testing**: Evaluate test suite effectiveness
//! - **Coverage Tracking**: Instrument code for test coverage
//! - **Multiple Output Formats**: JSON, HTML, Mermaid, DOT
//!
//! # Quick Start
//!
//! ```no_run
//! use instrument_rs::{Instrumentor, Config};
//!
//! // Create default configuration
//! let config = Config::default();
//!
//! // Create instrumentor
//! let instrumentor = Instrumentor::new(config);
//!
//! // Run analysis
//! instrumentor.run()?;
//! # Ok::<(), instrument_rs::Error>(())
//! ```
//!
//! # Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`ast`]: AST parsing and analysis
//! - [`call_graph`]: Function call graph construction
//! - [`patterns`]: Pattern matching for code constructs
//! - [`framework`]: Web and test framework detection
//! - [`instrumentation`]: Code transformation for coverage/tracing
//! - [`mutation`]: Mutation testing implementation
//! - [`scoring`]: Code quality and instrumentation scoring
//! - [`output`]: Report generation in various formats
//!
//! # Example: Building a Call Graph
//!
//! ```no_run
//! use instrument_rs::call_graph::GraphBuilder;
//! use std::path::PathBuf;
//!
//! let mut builder = GraphBuilder::new();
//! builder.add_source_file(PathBuf::from("src/main.rs"))?;
//! 
//! let graph = builder.build()?;
//! println!("Found {} functions with {} calls", 
//!     graph.node_count(), 
//!     graph.edge_count()
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod ast;
pub mod call_graph;
pub mod config;
pub mod error;
pub mod framework;
pub mod instrumentation;
pub mod mutation;
pub mod output;
pub mod patterns;
pub mod reporting;
pub mod scoring;

pub use config::Config;
pub use error::{Error, Result};

// Re-export call graph types for convenience
pub use call_graph::{CallGraph, GraphBuilder};

/// The main entry point for the instrumentation library
pub struct Instrumentor {
    config: Config,
}

impl Instrumentor {
    /// Creates a new instrumentor with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the instrumentation process
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use instrument_rs::{Instrumentor, Config};
    ///
    /// let config = Config::default();
    /// let instrumentor = Instrumentor::new(config);
    /// ```
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the instrumentation process on the configured project
    ///
    /// # Errors
    ///
    /// Returns an error if the instrumentation process fails
    pub fn run(&self) -> Result<()> {
        // TODO: Implement the main instrumentation logic
        Ok(())
    }
}
