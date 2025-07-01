//! Example demonstrating Axum framework detection
//!
//! This example shows how to use the framework detection system
//! to identify Axum usage in a project and extract endpoint information.

use instrument_rs::framework::web::{DetectorRegistry, FrameworkDetector};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a detector registry
    let registry = DetectorRegistry::new();

    // Get the current project path
    let project_path = Path::new(".");

    // Detect frameworks in the project
    println!("Detecting web frameworks in project...");
    let detected_frameworks = registry.detect_frameworks(project_path)?;

    if detected_frameworks.is_empty() {
        println!("No web frameworks detected.");
    } else {
        println!("Detected frameworks: {:?}", detected_frameworks);

        // If Axum is detected, analyze some example code
        if detected_frameworks.contains(&"Axum") {
            println!("\nAxum framework detected! Analyzing endpoints...");

            // Example: Parse a sample Axum router
            let sample_code = r#"
                use axum::{Router, routing::{get, post}, Json};
                use serde::{Deserialize, Serialize};
                
                #[derive(Serialize)]
                struct User {
                    id: u32,
                    name: String,
                }
                
                /// Get user by ID
                async fn get_user(Path(id): Path<u32>) -> Json<User> {
                    Json(User {
                        id,
                        name: "John Doe".to_string(),
                    })
                }
                
                /// Create a new user
                async fn create_user(Json(user): Json<User>) -> StatusCode {
                    StatusCode::CREATED
                }
                
                /// Health check endpoint
                async fn health() -> &'static str {
                    "OK"
                }
                
                pub fn create_router() -> Router {
                    Router::new()
                        .route("/health", get(health))
                        .route("/users/:id", get(get_user))
                        .route("/users", post(create_user))
                        .layer(tower_http::trace::TraceLayer::new_for_http())
                }
            "#;

            // Parse the code
            let syntax_tree = syn::parse_file(sample_code)?;

            // Get the Axum detector
            if let Some(axum_detector) = registry.get_detector("Axum") {
                // Extract endpoints
                let endpoints = axum_detector.extract_endpoints(&syntax_tree);

                println!("\nFound {} endpoints:", endpoints.len());
                for endpoint in &endpoints {
                    println!(
                        "  {} {} -> {}",
                        endpoint.method.as_str(),
                        endpoint.path,
                        endpoint.handler
                    );
                }

                // Analyze handler functions
                println!("\nAnalyzing handlers...");
                for item in &syntax_tree.items {
                    if let syn::Item::Fn(item_fn) = item {
                        if let Some(handler_info) = axum_detector.detect_handler(item_fn) {
                            println!("\nHandler: {}", handler_info.name);
                            println!("  Async: {}", handler_info.is_async);
                            println!("  Return type: {}", handler_info.return_type);
                            if !handler_info.extractors.is_empty() {
                                println!("  Extractors: {:?}", handler_info.extractors);
                            }
                            if let Some(docs) = &handler_info.documentation {
                                println!("  Documentation: {}", docs);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
