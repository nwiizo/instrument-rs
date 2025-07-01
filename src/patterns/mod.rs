//! Pattern matching module for identifying code patterns and constructs
//!
//! This module provides a flexible pattern matching system for identifying
//! various code constructs in Rust source files. It supports regex-based
//! matching, AST structure matching, and custom pattern definitions.
//!
//! # Overview
//!
//! The pattern system can identify:
//! - Test functions and test modules
//! - Database operations (queries, transactions)
//! - HTTP client calls and API interactions
//! - Error handling patterns
//! - Authentication and authorization code
//! - Business logic functions
//! - External service calls
//! - Framework-specific patterns
//!
//! # Pattern Categories
//!
//! Patterns are organized into categories:
//! - `Test`: Test functions and test infrastructure
//! - `Database`: Database queries and operations
//! - `HttpClient`: HTTP requests and API calls
//! - `ErrorHandling`: Error creation and propagation
//! - `Authentication`: Auth checks and token handling
//! - `BusinessLogic`: Core business functionality
//! - `ExternalService`: Calls to external systems
//!
//! # Example
//!
//! ```no_run
//! use instrument_rs::patterns::{PatternMatcher, PatternSet};
//!
//! // Load default patterns
//! let patterns = PatternSet::default();
//! let matcher = PatternMatcher::new(patterns);
//!
//! // Match against a function
//! let matches = matcher.match_function("process_payment", &[]);
//! for m in matches {
//!     println!("Found {} pattern with confidence {}", m.category, m.confidence);
//! }
//! ```
//!
//! # Custom Patterns
//!
//! You can define custom patterns in TOML:
//!
//! ```toml
//! [[patterns]]
//! name = "custom_api_call"
//! category = "external_service"
//! regex = "^(call|invoke|request)_.*_api$"
//! confidence = 0.9
//! description = "Custom API call pattern"
//! ```

mod matcher;
mod pattern_set;
mod result;

#[cfg(test)]
mod test_standalone;

pub use matcher::PatternMatcher;
pub use pattern_set::{Pattern, PatternSet};
pub use result::{Category, MatchResult};
