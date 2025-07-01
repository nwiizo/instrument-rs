//! Web framework detection and analysis
//!
//! This module provides functionality to detect and analyze web frameworks
//! used in Rust projects, with a focus on extracting HTTP endpoints,
//! routes, and handler signatures.

use crate::Result;
use std::path::Path;
use syn::{Item, ItemFn};

pub mod axum;

pub use axum::AxumDetector;

#[cfg(test)]
mod test_standalone;

/// Trait for detecting and analyzing web frameworks
///
/// Implementors of this trait provide framework-specific logic to:
/// - Detect if a framework is being used in a project
/// - Extract HTTP endpoints and their metadata
/// - Analyze route configurations
/// - Parse handler signatures
pub trait FrameworkDetector: Send + Sync {
    /// Get the name of the framework this detector handles
    ///
    /// # Returns
    ///
    /// The display name of the framework (e.g., "Axum", "Actix-web", "Rocket")
    fn name(&self) -> &'static str;

    /// Detect if this framework is used in the project
    ///
    /// This method examines the project's dependencies and source code
    /// to determine if the framework is being used.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project to analyze
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the framework is detected, `Ok(false)` otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail or if detection encounters issues
    fn detect(&self, project_root: &Path) -> Result<bool>;

    /// Extract HTTP endpoints from the source code
    ///
    /// Analyzes the AST to find all HTTP endpoints defined using this framework.
    ///
    /// # Arguments
    ///
    /// * `syntax_tree` - The parsed syntax tree of a source file
    ///
    /// # Returns
    ///
    /// A vector of detected endpoints
    fn extract_endpoints(&self, syntax_tree: &syn::File) -> Vec<Endpoint>;

    /// Analyze router configuration
    ///
    /// Extracts routing information from router setup code.
    ///
    /// # Arguments
    ///
    /// * `item` - An AST item that might contain router configuration
    ///
    /// # Returns
    ///
    /// Router information if found
    fn analyze_router(&self, item: &Item) -> Option<RouterInfo>;

    /// Detect handler function signatures
    ///
    /// Identifies functions that serve as HTTP request handlers.
    ///
    /// # Arguments
    ///
    /// * `function` - A function item to analyze
    ///
    /// # Returns
    ///
    /// Handler information if the function is a valid handler
    fn detect_handler(&self, function: &ItemFn) -> Option<HandlerInfo>;
}

/// Information about an HTTP endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// HTTP method (GET, POST, etc.)
    pub method: HttpMethod,

    /// Route path (e.g., "/users/:id")
    pub path: String,

    /// Handler function name
    pub handler: String,

    /// Module path to the handler
    pub module_path: Vec<String>,

    /// Line number where the endpoint is defined
    pub line: usize,

    /// Middleware applied to this endpoint
    pub middleware: Vec<String>,

    /// Documentation extracted from the handler
    pub documentation: Option<String>,
}

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    /// GET request
    Get,
    /// POST request
    Post,
    /// PUT request
    Put,
    /// DELETE request
    Delete,
    /// PATCH request
    Patch,
    /// HEAD request
    Head,
    /// OPTIONS request
    Options,
    /// CONNECT request
    Connect,
    /// TRACE request
    Trace,
    /// Custom method
    Custom(&'static str),
}

impl HttpMethod {
    /// Convert from a string representation
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "PATCH" => Self::Patch,
            "HEAD" => Self::Head,
            "OPTIONS" => Self::Options,
            "CONNECT" => Self::Connect,
            "TRACE" => Self::Trace,
            _ => Self::Custom(Box::leak(s.to_string().into_boxed_str())),
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Connect => "CONNECT",
            Self::Trace => "TRACE",
            Self::Custom(s) => s,
        }
    }
}

/// Information about a router configuration
#[derive(Debug, Clone)]
pub struct RouterInfo {
    /// Name of the router variable
    pub name: String,

    /// Base path for all routes in this router
    pub base_path: Option<String>,

    /// Routes defined in this router
    pub routes: Vec<RouteInfo>,

    /// Middleware applied to all routes
    pub middleware: Vec<String>,

    /// Nested routers
    pub nested_routers: Vec<String>,
}

/// Information about a single route
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// HTTP method
    pub method: HttpMethod,

    /// Route path
    pub path: String,

    /// Handler function name
    pub handler: String,

    /// Route-specific middleware
    pub middleware: Vec<String>,
}

/// Information about a handler function
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    /// Function name
    pub name: String,

    /// Whether the handler is async
    pub is_async: bool,

    /// Parameter types
    pub parameters: Vec<HandlerParam>,

    /// Return type
    pub return_type: String,

    /// Extractors used (e.g., Json, Path, Query)
    pub extractors: Vec<String>,

    /// Documentation
    pub documentation: Option<String>,
}

/// Handler function parameter
#[derive(Debug, Clone)]
pub struct HandlerParam {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub ty: String,

    /// Whether this is an extractor
    pub is_extractor: bool,
}

/// Registry for framework detectors
pub struct DetectorRegistry {
    detectors: Vec<Box<dyn FrameworkDetector>>,
}

impl DetectorRegistry {
    /// Create a new detector registry with default detectors
    pub fn new() -> Self {
        Self {
            detectors: vec![
                Box::new(axum::AxumDetector::new()),
                // Future: Add more framework detectors here
            ],
        }
    }

    /// Add a custom detector to the registry
    ///
    /// # Arguments
    ///
    /// * `detector` - The detector to add
    pub fn add_detector(&mut self, detector: Box<dyn FrameworkDetector>) {
        self.detectors.push(detector);
    }

    /// Detect which frameworks are used in a project
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project
    ///
    /// # Returns
    ///
    /// Names of detected frameworks
    ///
    /// # Errors
    ///
    /// Returns an error if detection fails
    pub fn detect_frameworks(&self, project_root: &Path) -> Result<Vec<&'static str>> {
        let mut detected = Vec::new();

        for detector in &self.detectors {
            if detector.detect(project_root)? {
                detected.push(detector.name());
            }
        }

        Ok(detected)
    }

    /// Get a detector by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the framework
    ///
    /// # Returns
    ///
    /// The detector if found
    pub fn get_detector(&self, name: &str) -> Option<&dyn FrameworkDetector> {
        self.detectors
            .iter()
            .find(|d| d.name() == name)
            .map(|d| d.as_ref())
    }
}

impl Default for DetectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(HttpMethod::from_str("GET"), HttpMethod::Get);
        assert_eq!(HttpMethod::from_str("post"), HttpMethod::Post);
        match HttpMethod::from_str("CUSTOM") {
            HttpMethod::Custom(s) => assert_eq!(s, "CUSTOM"),
            _ => panic!("Expected Custom variant"),
        }

        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
    }

    #[test]
    fn test_detector_registry() {
        let registry = DetectorRegistry::new();

        // Should have at least the Axum detector
        assert!(registry.get_detector("Axum").is_some());
    }
}
