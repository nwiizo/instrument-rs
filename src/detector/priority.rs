//! Priority scoring for instrumentation points
//!
//! Determines which code locations should be instrumented first.

use super::{Field, InstrumentationKind, InstrumentationPoint, Location, Priority};
use crate::call_graph::CallGraph;
use crate::patterns::MatchResult;

/// Calculate priority and create instrumentation points
pub fn prioritize_points(
    graph: &CallGraph,
    endpoints: &[super::Endpoint],
    patterns: &[MatchResult],
    threshold: f64,
) -> Vec<InstrumentationPoint> {
    let mut points = Vec::new();

    // Add endpoints as high priority
    for endpoint in endpoints {
        let priority = Priority::Critical;
        let suggested_fields = vec![
            Field {
                name: "method".to_string(),
                expression: "method".to_string(),
                is_sensitive: false,
            },
            Field {
                name: "path".to_string(),
                expression: "path".to_string(),
                is_sensitive: false,
            },
        ];

        points.push(InstrumentationPoint {
            location: endpoint.location.clone(),
            kind: InstrumentationKind::Endpoint,
            priority,
            reason: format!("{} endpoint handler", endpoint.method),
            suggested_span_name: format!(
                "{}_{}",
                endpoint.method.to_lowercase(),
                sanitize_path(&endpoint.path)
            ),
            suggested_fields,
        });
    }

    // Add pattern matches
    for pattern in patterns {
        let kind = pattern_to_kind(&pattern.category);
        let priority = calculate_pattern_priority(
            &pattern.category,
            pattern.confidence,
            graph,
            &pattern.function_name,
        );

        if priority.score() as f64 / 4.0 >= threshold {
            let suggested_fields = suggest_fields_for_kind(&kind, &pattern.function_name);

            points.push(InstrumentationPoint {
                location: Location {
                    file: pattern.file.clone(),
                    line: pattern.line,
                    column: 1,
                    function_name: pattern.function_name.clone(),
                },
                kind,
                priority,
                reason: format!(
                    "Matched {} pattern with {:.0}% confidence",
                    pattern.category.name(),
                    pattern.confidence * 100.0
                ),
                suggested_span_name: generate_span_name(&kind, &pattern.function_name),
                suggested_fields,
            });
        }
    }

    // Score based on call graph position
    for node_name in graph.node_names() {
        if let Some(node) = graph.get_node(&node_name) {
            // High connectivity = more important
            let caller_count = node.called_by().len();
            let callee_count = node.calls().len();

            // Skip if already added
            if points.iter().any(|p| p.location.function_name == node_name) {
                continue;
            }

            // Add functions with high connectivity
            if caller_count > 5 || callee_count > 10 {
                let priority = if caller_count > 10 {
                    Priority::High
                } else {
                    Priority::Medium
                };

                if priority.score() as f64 / 4.0 >= threshold {
                    points.push(InstrumentationPoint {
                        location: Location {
                            file: node.file().unwrap_or_default(),
                            line: node.line().unwrap_or(0),
                            column: 1,
                            function_name: node_name.clone(),
                        },
                        kind: InstrumentationKind::BusinessLogic,
                        priority,
                        reason: format!(
                            "High connectivity: {} callers, {} callees",
                            caller_count, callee_count
                        ),
                        suggested_span_name: node_name.clone(),
                        suggested_fields: Vec::new(),
                    });
                }
            }
        }
    }

    // Sort by priority (critical first)
    points.sort_by_key(|p| std::cmp::Reverse(p.priority.score()));

    points
}

fn pattern_to_kind(category: &crate::patterns::Category) -> InstrumentationKind {
    match category {
        crate::patterns::Category::Database => InstrumentationKind::DatabaseCall,
        crate::patterns::Category::HttpClient => InstrumentationKind::ExternalApiCall,
        crate::patterns::Category::ExternalService => InstrumentationKind::ExternalApiCall,
        crate::patterns::Category::Cache => InstrumentationKind::CacheOperation,
        crate::patterns::Category::MessageQueue => InstrumentationKind::MessageQueue,
        crate::patterns::Category::ErrorHandling => InstrumentationKind::ErrorBoundary,
        crate::patterns::Category::BusinessLogic => InstrumentationKind::BusinessLogic,
        _ => InstrumentationKind::BusinessLogic,
    }
}

fn calculate_pattern_priority(
    category: &crate::patterns::Category,
    confidence: f64,
    graph: &CallGraph,
    function_name: &str,
) -> Priority {
    // Base priority from category
    let base_priority = match category {
        crate::patterns::Category::Database
        | crate::patterns::Category::HttpClient
        | crate::patterns::Category::ExternalService => Priority::High,
        crate::patterns::Category::ErrorHandling | crate::patterns::Category::Auth => {
            Priority::High
        }
        crate::patterns::Category::BusinessLogic => Priority::Medium,
        _ => Priority::Low,
    };

    // Boost priority based on call graph position
    if let Some(node) = graph.get_node(function_name) {
        let caller_count = node.called_by().len();
        if caller_count > 5 && base_priority == Priority::Medium {
            return Priority::High;
        }
    }

    // Boost if confidence is very high
    if confidence > 0.95 && base_priority == Priority::Medium {
        return Priority::High;
    }

    base_priority
}

fn suggest_fields_for_kind(kind: &InstrumentationKind, function_name: &str) -> Vec<Field> {
    match kind {
        InstrumentationKind::DatabaseCall => vec![
            Field {
                name: "query".to_string(),
                expression: "query".to_string(),
                is_sensitive: false,
            },
            Field {
                name: "table".to_string(),
                expression: "table".to_string(),
                is_sensitive: false,
            },
        ],
        InstrumentationKind::ExternalApiCall => vec![
            Field {
                name: "url".to_string(),
                expression: "url".to_string(),
                is_sensitive: false,
            },
            Field {
                name: "method".to_string(),
                expression: "method".to_string(),
                is_sensitive: false,
            },
        ],
        InstrumentationKind::CacheOperation => vec![
            Field {
                name: "key".to_string(),
                expression: "key".to_string(),
                is_sensitive: false,
            },
            Field {
                name: "operation".to_string(),
                expression: "op".to_string(),
                is_sensitive: false,
            },
        ],
        InstrumentationKind::ErrorBoundary => vec![Field {
            name: "error".to_string(),
            expression: "err".to_string(),
            is_sensitive: false,
        }],
        _ => Vec::new(),
    }
}

fn generate_span_name(kind: &InstrumentationKind, function_name: &str) -> String {
    let prefix = match kind {
        InstrumentationKind::DatabaseCall => "db",
        InstrumentationKind::ExternalApiCall => "http",
        InstrumentationKind::CacheOperation => "cache",
        InstrumentationKind::MessageQueue => "mq",
        InstrumentationKind::ErrorBoundary => "error",
        InstrumentationKind::BackgroundJob => "job",
        _ => "",
    };

    if prefix.is_empty() {
        function_name.to_string()
    } else {
        format!("{prefix}.{function_name}")
    }
}

fn sanitize_path(path: &str) -> String {
    path.replace('/', "_")
        .replace(':', "")
        .replace('{', "")
        .replace('}', "")
        .trim_matches('_')
        .to_string()
}
