//! instrument-rs: A Rust CLI tool for detecting optimal instrumentation points
//!
//! This library analyzes Rust codebases to identify critical paths that need
//! observability instrumentation (tracing, logging, metrics). It traces execution
//! flows from HTTP/gRPC endpoints to help teams implement comprehensive
//! observability strategies.
//!
//! # Features
//!
//! - **AST-based Analysis**: Deep code analysis using Rust's syntax tree
//! - **Call Graph Construction**: Build and analyze function call relationships
//! - **Pattern Recognition**: Identify business logic, DB calls, external APIs
//! - **Framework Detection**: Auto-detect web frameworks (axum, actix, rocket, tonic)
//! - **Instrumentation Detection**: Find where `#[instrument]` should be added
//! - **Multiple Output Formats**: Human-readable, JSON, Mermaid diagrams
//!
//! # Quick Start
//!
//! ```no_run
//! use instrument_rs::{Analyzer, Config};
//!
//! let config = Config::default();
//! let analyzer = Analyzer::new(config);
//! let result = analyzer.analyze(&["src"])?;
//!
//! println!("Found {} instrumentation points", result.points.len());
//! # Ok::<(), instrument_rs::Error>(())
//! ```
//!
//! # Architecture
//!
//! The library is organized into these key modules:
//!
//! - [`ast`]: AST parsing and analysis
//! - [`call_graph`]: Function call graph construction
//! - [`patterns`]: Pattern matching for code constructs
//! - [`framework`]: Web framework detection
//! - [`detector`]: Instrumentation point detection
//! - [`output`]: Report generation in various formats

#![warn(missing_docs)]
#![warn(clippy::all)]
// Temporarily relax some clippy lints during refactoring
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::collapsible_str_replace)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::single_char_add_str)]
#![allow(clippy::new_without_default)]
#![allow(noop_method_call)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod ast;
pub mod call_graph;
pub mod config;
pub mod dependencies;
#[cfg(test)]
mod dependencies_test;
pub mod detector;
pub mod error;
pub mod fixer;
pub mod framework;
pub mod output;
pub mod patterns;

pub use config::Config;
pub use dependencies::{DetectionContext, ProjectDependencies};
pub use error::{Error, Result};

// Re-export call graph types for convenience
pub use call_graph::{CallGraph, GraphBuilder};

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Analysis result containing all detected information
#[derive(Debug)]
pub struct AnalysisResult {
    /// Detected HTTP/gRPC endpoints
    pub endpoints: Vec<detector::Endpoint>,
    /// Function call graph
    pub call_graph: CallGraph,
    /// Matched patterns (DB calls, external APIs, etc.)
    pub patterns: Vec<patterns::MatchResult>,
    /// Suggested instrumentation points
    pub points: Vec<detector::InstrumentationPoint>,
    /// Existing instrumentation found in code
    pub existing_instrumentation: Vec<detector::ExistingInstrumentation>,
    /// Gaps in instrumentation coverage
    pub gaps: Vec<detector::InstrumentationGap>,
    /// Rule violations found in existing instrumentation
    pub rule_violations: Vec<detector::rules::RuleViolation>,
    /// Project dependencies (for context-aware detection)
    pub dependencies: ProjectDependencies,
    /// Analysis statistics
    pub stats: AnalysisStats,
}

/// Statistics about the analyzed codebase
#[derive(Debug, Default)]
pub struct AnalysisStats {
    /// Total files analyzed
    pub total_files: usize,
    /// Total functions found
    pub total_functions: usize,
    /// Total lines of code
    pub total_lines: usize,
    /// Number of endpoints detected
    pub endpoints_count: usize,
    /// Number of instrumentation points suggested
    pub instrumentation_points: usize,
    /// Number of existing instrumentation found
    pub existing_count: usize,
    /// Number of instrumentation gaps found
    pub gaps_count: usize,
    /// Number of rule violations found
    pub rule_violations_count: usize,
}

/// The main analyzer for detecting instrumentation points
pub struct Analyzer {
    config: Config,
}

impl Analyzer {
    /// Creates a new analyzer with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Analyze the given paths and return detection results
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to analyze (files or directories)
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or parsing fails
    pub fn analyze<P: AsRef<Path>>(&self, paths: &[P]) -> Result<AnalysisResult> {
        // 0. Analyze project dependencies for context-aware detection
        let project_root = paths.first().map(|p| p.as_ref()).unwrap_or(Path::new("."));
        let dependencies = ProjectDependencies::from_manifest(project_root).unwrap_or_default();
        let context = DetectionContext::from_deps(dependencies);

        // 1. Collect all Rust files
        let files = self.collect_files(paths)?;

        // 2. Parse files sequentially (syn types don't implement Send)
        let parsed: Vec<_> = files
            .iter()
            .filter_map(|f| self.parse_file(f).ok())
            .collect();

        // 3. Build call graph
        let mut graph_builder = GraphBuilder::new();
        for source in &parsed {
            graph_builder.add_parsed_file(source)?;
        }
        let call_graph = graph_builder.build()?;

        // 4. Detect framework and endpoints (use deps for framework hint)
        let framework = self.detect_framework_with_context(&parsed, &context);
        let endpoints = self.detect_endpoints(&parsed, &framework);

        // 5. Match patterns with dependency context
        let patterns = self.match_patterns_with_context(&call_graph, &context);

        // 6. Detect instrumentation points
        let points = self.detect_instrumentation_points(&call_graph, &endpoints, &patterns);

        // 7. Detect existing instrumentation
        let existing_instrumentation = detector::existing::detect_existing_instrumentation(&parsed);

        // 8. Detect gaps (instrumentation points without existing instrumentation)
        let gaps = self.detect_gaps(&points, &existing_instrumentation);

        // 9. Check naming convention rules
        let rule_checker = detector::rules::RuleChecker::new(&self.config.naming_rules);
        let mut rule_violations = rule_checker.check_existing(&existing_instrumentation);
        rule_violations.extend(rule_checker.check_points(&points));

        // 10. Compute stats
        let stats = AnalysisStats {
            total_files: parsed.len(),
            total_functions: call_graph.node_count(),
            total_lines: parsed.iter().map(|p| p.line_count()).sum(),
            endpoints_count: endpoints.len(),
            instrumentation_points: points.len(),
            existing_count: existing_instrumentation.len(),
            gaps_count: gaps.len(),
            rule_violations_count: rule_violations.len(),
        };

        // Extract dependencies from context
        let dependencies = context.deps;

        Ok(AnalysisResult {
            endpoints,
            call_graph,
            patterns,
            points,
            existing_instrumentation,
            gaps,
            rule_violations,
            dependencies,
            stats,
        })
    }

    fn collect_files<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for path in paths {
            let path = path.as_ref();
            if path.is_file() {
                if path.extension().is_some_and(|ext| ext == "rs") {
                    files.push(path.to_path_buf());
                }
            } else if path.is_dir() {
                for entry in WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let p = entry.path();
                    if p.extension().is_some_and(|ext| ext == "rs") {
                        // Skip excluded patterns
                        let should_exclude = self
                            .config
                            .exclude_patterns
                            .iter()
                            .any(|pattern| p.to_string_lossy().contains(pattern));
                        if !should_exclude {
                            files.push(p.to_path_buf());
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    fn parse_file(&self, path: &Path) -> Result<ast::SourceFile> {
        ast::SourceFile::parse(path)
    }

    fn detect_framework(&self, parsed: &[ast::SourceFile]) -> framework::DetectedFramework {
        // Check for framework-specific imports
        for file in parsed {
            let source = file.source();
            if source.contains("axum::") || source.contains("use axum") {
                return framework::DetectedFramework::Axum;
            }
            if source.contains("actix_web::") || source.contains("use actix_web") {
                return framework::DetectedFramework::Actix;
            }
            if source.contains("rocket::") || source.contains("#[rocket") {
                return framework::DetectedFramework::Rocket;
            }
            if source.contains("tonic::") || source.contains("use tonic") {
                return framework::DetectedFramework::Tonic;
            }
        }
        framework::DetectedFramework::Unknown
    }

    /// Detect framework using both source analysis and dependency information
    fn detect_framework_with_context(
        &self,
        parsed: &[ast::SourceFile],
        context: &DetectionContext,
    ) -> framework::DetectedFramework {
        // First check dependencies (more reliable)
        if context
            .deps
            .frameworks
            .contains(&dependencies::FrameworkCrate::Axum)
        {
            return framework::DetectedFramework::Axum;
        }
        if context
            .deps
            .frameworks
            .contains(&dependencies::FrameworkCrate::ActixWeb)
        {
            return framework::DetectedFramework::Actix;
        }
        if context
            .deps
            .frameworks
            .contains(&dependencies::FrameworkCrate::Rocket)
        {
            return framework::DetectedFramework::Rocket;
        }
        if context
            .deps
            .frameworks
            .contains(&dependencies::FrameworkCrate::Tonic)
        {
            return framework::DetectedFramework::Tonic;
        }

        // Fall back to source analysis
        self.detect_framework(parsed)
    }

    fn detect_endpoints(
        &self,
        parsed: &[ast::SourceFile],
        framework_type: &framework::DetectedFramework,
    ) -> Vec<detector::Endpoint> {
        detector::endpoint::detect_endpoints(parsed, framework_type)
    }

    fn match_patterns(&self, graph: &CallGraph) -> Vec<patterns::MatchResult> {
        // Pattern matching based on function names in the call graph
        let mut results = Vec::new();

        for node_name in graph.node_names() {
            if let Some(node) = graph.get_node(&node_name) {
                let mut result = patterns::MatchResult::with_location(
                    node.file().unwrap_or_default(),
                    node_name.clone(),
                    node.line().unwrap_or(0),
                );

                // Database patterns
                if matches_db_pattern(&node_name) {
                    result.category = patterns::Category::Database;
                    result.confidence = 0.9;
                    results.push(result);
                    continue;
                }

                // HTTP client patterns
                if matches_http_pattern(&node_name) {
                    result.category = patterns::Category::HttpClient;
                    result.confidence = 0.85;
                    results.push(result);
                    continue;
                }

                // Error handling patterns
                if matches_error_pattern(&node_name) {
                    result.category = patterns::Category::ErrorHandling;
                    result.confidence = 0.8;
                    results.push(result);
                    continue;
                }

                // Business logic patterns
                if matches_business_pattern(&node_name) {
                    result.category = patterns::Category::BusinessLogic;
                    result.confidence = 0.7;
                    results.push(result);
                }
            }
        }

        results
    }

    /// Match patterns with dependency context for smarter detection
    ///
    /// This method uses project dependency information to:
    /// - Avoid false positives (e.g., "get_user" won't be HTTP if no reqwest)
    /// - Boost confidence for known patterns (e.g., DB patterns if sqlx is used)
    fn match_patterns_with_context(
        &self,
        graph: &CallGraph,
        context: &DetectionContext,
    ) -> Vec<patterns::MatchResult> {
        let mut results = Vec::new();

        for node_name in graph.node_names() {
            if let Some(node) = graph.get_node(&node_name) {
                let mut result = patterns::MatchResult::with_location(
                    node.file().unwrap_or_default(),
                    node_name.clone(),
                    node.line().unwrap_or(0),
                );

                // Database patterns - only if project uses a DB crate
                if context.is_likely_db_operation(&node_name) {
                    result.category = patterns::Category::Database;
                    result.confidence = context.db_priority;
                    results.push(result);
                    continue;
                }

                // HTTP client patterns - only if project uses an HTTP client
                if context.is_likely_http_call(&node_name) {
                    result.category = patterns::Category::HttpClient;
                    result.confidence = context.http_priority;
                    results.push(result);
                    continue;
                }

                // Cache patterns - only if project uses a cache crate
                if context.is_likely_cache_operation(&node_name) {
                    result.category = patterns::Category::Cache;
                    result.confidence = context.cache_priority;
                    results.push(result);
                    continue;
                }

                // Error handling patterns (always relevant)
                if matches_error_pattern(&node_name) {
                    result.category = patterns::Category::ErrorHandling;
                    result.confidence = 0.8;
                    results.push(result);
                    continue;
                }

                // Business logic patterns (always relevant)
                if matches_business_pattern(&node_name) {
                    result.category = patterns::Category::BusinessLogic;
                    result.confidence = 0.7;
                    results.push(result);
                    continue;
                }

                // Fallback: use old naive patterns if context check passed
                // This handles cases where deps weren't detected but source shows usage
                if matches_db_pattern(&node_name) && context.deps.has_database() {
                    result.category = patterns::Category::Database;
                    result.confidence = 0.7; // Lower confidence for fallback
                    results.push(result);
                } else if matches_http_pattern(&node_name) && context.deps.has_http_client() {
                    result.category = patterns::Category::HttpClient;
                    result.confidence = 0.6; // Lower confidence for fallback
                    results.push(result);
                }
            }
        }

        results
    }

    fn detect_instrumentation_points(
        &self,
        graph: &CallGraph,
        endpoints: &[detector::Endpoint],
        patterns: &[patterns::MatchResult],
    ) -> Vec<detector::InstrumentationPoint> {
        detector::priority::prioritize_points(graph, endpoints, patterns, self.config.threshold)
    }

    /// Detect gaps between suggested instrumentation points and existing instrumentation
    fn detect_gaps(
        &self,
        points: &[detector::InstrumentationPoint],
        existing: &[detector::ExistingInstrumentation],
    ) -> Vec<detector::InstrumentationGap> {
        let mut gaps = Vec::new();

        for point in points {
            // Check if this point has existing instrumentation
            let has_existing = existing.iter().any(|e| {
                e.location.file == point.location.file
                    && (e.location.line == point.location.line
                        || e.location.line == point.location.line.saturating_sub(1)
                        || e.location.line == point.location.line + 1)
            });

            if !has_existing {
                let severity = match point.priority {
                    detector::Priority::Critical => detector::GapSeverity::Critical,
                    detector::Priority::High => detector::GapSeverity::Major,
                    _ => detector::GapSeverity::Minor,
                };

                gaps.push(detector::InstrumentationGap {
                    location: point.location.clone(),
                    description: format!(
                        "{} ({}) has no instrumentation",
                        point.location.function_name,
                        point.kind.name()
                    ),
                    suggested_fix: format!(
                        "#[instrument(name = \"{}\")]",
                        point.suggested_span_name
                    ),
                    severity,
                });
            }
        }

        gaps
    }
}

// Pattern matching helpers
fn matches_db_pattern(name: &str) -> bool {
    let db_patterns = [
        "query",
        "execute",
        "fetch",
        "insert",
        "update",
        "delete",
        "select",
        "transaction",
        "commit",
        "rollback",
        "connect",
        "pool",
        "database",
        "db_",
        "_db",
        "sql",
        "postgres",
        "mysql",
        "sqlite",
        "redis",
        "mongo",
        "dynamo",
    ];
    let lower = name.to_lowercase();
    db_patterns.iter().any(|p| lower.contains(p))
}

fn matches_http_pattern(name: &str) -> bool {
    let http_patterns = [
        "request",
        "response",
        "http",
        "fetch",
        "call_api",
        "send_request",
        "client",
        "get_",
        "post_",
        "put_",
        "delete_",
        "patch_",
        "api_call",
        "remote",
        "external",
    ];
    let lower = name.to_lowercase();
    http_patterns.iter().any(|p| lower.contains(p))
}

fn matches_error_pattern(name: &str) -> bool {
    let error_patterns = [
        "error",
        "handle_error",
        "map_err",
        "on_error",
        "catch",
        "recover",
        "fallback",
        "retry",
        "validate",
    ];
    let lower = name.to_lowercase();
    error_patterns.iter().any(|p| lower.contains(p))
}

fn matches_business_pattern(name: &str) -> bool {
    let business_patterns = [
        "process",
        "handle",
        "create",
        "calculate",
        "validate",
        "authorize",
        "authenticate",
        "payment",
        "order",
        "checkout",
        "register",
        "login",
        "logout",
        "subscribe",
        "publish",
    ];
    let lower = name.to_lowercase();
    business_patterns.iter().any(|p| lower.contains(p))
}

// Keep backward compatibility alias
/// Alias for `Analyzer` for backward compatibility
#[deprecated(since = "0.2.0", note = "Use `Analyzer` instead")]
pub type Instrumentor = Analyzer;
