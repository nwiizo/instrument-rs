//! Call graph construction for analyzing function dependencies and call relationships
//!
//! This module provides functionality to build and analyze call graphs from Rust code,
//! tracking function calls from endpoints through to external boundaries.
//!
//! # Overview
//!
//! A call graph is a directed graph where:
//! - Nodes represent functions, methods, or closures
//! - Edges represent function calls with metadata about the call site
//!
//! The call graph builder can:
//! - Trace execution paths from entry points (e.g., HTTP handlers)
//! - Identify critical paths through the codebase
//! - Detect cycles and recursive calls
//! - Resolve cross-module and cross-crate dependencies
//! - Track async function calls and spawned tasks
//!
//! # Example
//!
//! ```ignore
//! use instrument_rs::call_graph::{GraphBuilder, CallGraph};
//! use std::path::PathBuf;
//!
//! // Build a call graph from source files
//! let mut builder = GraphBuilder::new();
//! let graph = builder.build_from_directory(&PathBuf::from("src"))?;
//!
//! // Analyze the graph
//! println!("Total functions: {}", graph.node_count());
//! println!("Total calls: {}", graph.edge_count());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Architecture
//!
//! The call graph system consists of several components:
//!
//! - `GraphBuilder`: Constructs the graph from source files
//! - `CallGraph`: The graph data structure with analysis methods
//! - `FunctionNode`: Represents functions in the graph
//! - `CallEdge`: Represents function calls with metadata
//! - `SymbolResolver`: Resolves function references across modules

mod builder;
mod edge;
mod graph;
mod node;
mod resolver;

pub use builder::{GraphBuildError, GraphBuilder};
pub use edge::{CallEdge, CallKind};
pub use graph::{CallGraph, GraphStats};
pub use node::{FunctionNode, NodeKind};
pub use resolver::{ResolvedSymbol, SymbolResolver};

/// Result type for call graph operations
pub type Result<T> = std::result::Result<T, GraphBuildError>;
