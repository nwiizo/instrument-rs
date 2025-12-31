//! Endpoint detection from web frameworks
//!
//! Detects HTTP handlers and gRPC service methods from various frameworks.

use super::{Endpoint, Location};
use crate::ast::SourceFile;
use crate::framework::DetectedFramework;

/// Detect endpoints from parsed source files
pub fn detect_endpoints(files: &[SourceFile], framework: &DetectedFramework) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();

    for file in files {
        match framework {
            DetectedFramework::Axum => {
                endpoints.extend(detect_axum_endpoints(file));
            }
            DetectedFramework::Actix => {
                endpoints.extend(detect_actix_endpoints(file));
            }
            DetectedFramework::Rocket => {
                endpoints.extend(detect_rocket_endpoints(file));
            }
            DetectedFramework::Tonic => {
                endpoints.extend(detect_tonic_endpoints(file));
            }
            DetectedFramework::Unknown => {
                // Try all frameworks
                endpoints.extend(detect_axum_endpoints(file));
                endpoints.extend(detect_actix_endpoints(file));
                endpoints.extend(detect_rocket_endpoints(file));
                endpoints.extend(detect_tonic_endpoints(file));
            }
        }
    }

    endpoints
}

fn detect_axum_endpoints(file: &SourceFile) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();
    let source = file.source();

    // Look for Router::new().route() patterns
    // and handler functions with axum extractors
    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();

        // Match .route("/path", method(handler))
        if let Some(captures) = parse_axum_route(line) {
            let handler = captures.handler;
            endpoints.push(Endpoint {
                method: captures.method,
                path: captures.path,
                handler: handler.clone(),
                location: Location {
                    file: file.path().to_path_buf(),
                    line: line_num + 1,
                    column: 1,
                    function_name: handler,
                },
                framework: "axum".to_string(),
            });
        }
    }

    endpoints
}

struct RouteCapture {
    method: String,
    path: String,
    handler: String,
}

fn parse_axum_route(line: &str) -> Option<RouteCapture> {
    // Simple pattern matching for .route("/path", get(handler))
    if !line.contains(".route(") {
        return None;
    }

    // Extract path
    let path_start = line.find('"')? + 1;
    let path_end = line[path_start..].find('"')? + path_start;
    let path = line[path_start..path_end].to_string();

    // Extract method and handler
    let methods = ["get", "post", "put", "delete", "patch", "head", "options"];
    for method in methods {
        let pattern = format!("{method}(");
        if let Some(pos) = line.find(&pattern) {
            let handler_start = pos + pattern.len();
            let handler_end = line[handler_start..].find(')')? + handler_start;
            let handler = line[handler_start..handler_end].trim().to_string();
            return Some(RouteCapture {
                method: method.to_uppercase(),
                path,
                handler,
            });
        }
    }

    None
}

fn detect_actix_endpoints(file: &SourceFile) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();
    let source = file.source();

    // Look for #[get("/path")], #[post("/path")] etc.
    let methods = ["get", "post", "put", "delete", "patch", "head"];

    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();

        for method in methods {
            let pattern = format!("#[{method}(");
            if line.starts_with(&pattern) {
                if let Some(path) = extract_path_from_attribute(line) {
                    // Try to find the handler name on the next non-empty line
                    let handler = find_next_function_name(source, line_num)
                        .unwrap_or_else(|| "unknown".to_string());
                    endpoints.push(Endpoint {
                        method: method.to_uppercase(),
                        path,
                        handler: handler.clone(),
                        location: Location {
                            file: file.path().to_path_buf(),
                            line: line_num + 1,
                            column: 1,
                            function_name: handler,
                        },
                        framework: "actix-web".to_string(),
                    });
                }
            }
        }
    }

    endpoints
}

fn detect_rocket_endpoints(file: &SourceFile) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();
    let source = file.source();

    // Look for #[get("/path")], #[post("/path")] etc.
    let methods = ["get", "post", "put", "delete", "patch", "head"];

    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();

        for method in methods {
            // Rocket uses same attribute syntax as actix
            let pattern = format!("#[{method}(");
            if line.starts_with(&pattern) {
                if let Some(path) = extract_path_from_attribute(line) {
                    let handler = find_next_function_name(source, line_num)
                        .unwrap_or_else(|| "unknown".to_string());
                    endpoints.push(Endpoint {
                        method: method.to_uppercase(),
                        path,
                        handler: handler.clone(),
                        location: Location {
                            file: file.path().to_path_buf(),
                            line: line_num + 1,
                            column: 1,
                            function_name: handler,
                        },
                        framework: "rocket".to_string(),
                    });
                }
            }
        }
    }

    endpoints
}

fn detect_tonic_endpoints(file: &SourceFile) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();
    let source = file.source();

    // Look for #[tonic::async_trait] impl blocks and gRPC methods
    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();

        // Look for async fn inside impl blocks that look like gRPC services
        if line.starts_with("async fn ") && line.contains("Request<") {
            if let Some(fn_name) = extract_function_name(line) {
                endpoints.push(Endpoint {
                    method: "gRPC".to_string(),
                    path: format!("/{fn_name}"),
                    handler: fn_name.clone(),
                    location: Location {
                        file: file.path().to_path_buf(),
                        line: line_num + 1,
                        column: 1,
                        function_name: fn_name,
                    },
                    framework: "tonic".to_string(),
                });
            }
        }
    }

    endpoints
}

fn extract_path_from_attribute(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

fn find_next_function_name(source: &str, current_line: usize) -> Option<String> {
    for line in source.lines().skip(current_line + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with("pub async fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("fn ")
        {
            return extract_function_name(trimmed);
        }
        // Skip attributes and comments
        if !trimmed.starts_with('#') && !trimmed.starts_with("//") && !trimmed.is_empty() {
            break;
        }
    }
    None
}

fn extract_function_name(line: &str) -> Option<String> {
    let line = line
        .trim_start_matches("pub ")
        .trim_start_matches("async ")
        .trim_start_matches("fn ");

    let end = line.find('(')?;
    let name = line[..end].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
