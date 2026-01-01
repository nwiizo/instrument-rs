//! End-to-end tests for instrument-rs
//!
//! These tests create complete sample projects and verify the full analysis pipeline.

mod common;

use common::TestProject;
use common::sample_projects;
use instrument_rs::dependencies::{DetectionContext, ProjectDependencies};
use instrument_rs::{Analyzer, Config};

// ============================================================================
// Axum Endpoint Detection Tests
// ============================================================================

#[test]
fn test_e2e_axum_endpoint_detection() {
    let project = create_axum_project();
    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];
    let result = analyzer.analyze(&paths).unwrap();

    // Should detect endpoints
    assert!(
        result.stats.endpoints_count > 0,
        "Should detect at least one endpoint"
    );

    // Verify specific endpoints are detected
    let endpoint_handlers: Vec<&str> = result
        .endpoints
        .iter()
        .map(|e| e.handler.as_str())
        .collect();

    assert!(
        endpoint_handlers.contains(&"health_check"),
        "Should detect health_check endpoint"
    );
    assert!(
        endpoint_handlers.contains(&"get_user"),
        "Should detect get_user endpoint"
    );
    assert!(
        endpoint_handlers.contains(&"create_user"),
        "Should detect create_user endpoint"
    );
    assert!(
        endpoint_handlers.contains(&"list_orders"),
        "Should detect list_orders endpoint"
    );
}

#[test]
fn test_e2e_axum_route_paths() {
    let project = create_axum_project();
    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];
    let result = analyzer.analyze(&paths).unwrap();

    // Verify route paths are correctly extracted
    let routes: Vec<(&str, &str)> = result
        .endpoints
        .iter()
        .map(|e| (e.method.as_str(), e.path.as_str()))
        .collect();

    assert!(
        routes.contains(&("GET", "/health")),
        "Should detect GET /health"
    );
    assert!(
        routes.contains(&("GET", "/users/:id")),
        "Should detect GET /users/:id"
    );
    assert!(
        routes.contains(&("POST", "/users")),
        "Should detect POST /users"
    );
}

// ============================================================================
// Dependency Detection Tests
// ============================================================================

#[test]
fn test_e2e_dependency_detection_sqlx() {
    let project = create_database_project();

    let deps = ProjectDependencies::from_manifest(&project.root_path).unwrap();

    assert!(
        !deps.databases.is_empty(),
        "Should detect database dependencies"
    );
    assert!(
        deps.all_deps.contains("sqlx"),
        "Should detect sqlx in all_deps"
    );
}

#[test]
fn test_e2e_dependency_detection_reqwest() {
    let project = create_http_client_project();

    let deps = ProjectDependencies::from_manifest(&project.root_path).unwrap();

    assert!(
        !deps.http_clients.is_empty(),
        "Should detect HTTP client dependencies"
    );
    assert!(
        deps.all_deps.contains("reqwest"),
        "Should detect reqwest in all_deps"
    );
}

#[test]
fn test_e2e_dependency_detection_full_stack() {
    let project = create_full_stack_project();

    let deps = ProjectDependencies::from_manifest(&project.root_path).unwrap();

    // Should detect all categories
    assert!(
        !deps.frameworks.is_empty(),
        "Should detect framework dependencies"
    );
    assert!(
        !deps.databases.is_empty(),
        "Should detect database dependencies"
    );
    assert!(
        !deps.http_clients.is_empty(),
        "Should detect HTTP client dependencies"
    );
    assert!(!deps.caches.is_empty(), "Should detect cache dependencies");
    assert!(
        !deps.observability.is_empty(),
        "Should detect observability dependencies"
    );
}

// ============================================================================
// Detection Context Tests
// ============================================================================

#[test]
fn test_e2e_detection_context_db_operations() {
    let project = create_database_project();

    let deps = ProjectDependencies::from_manifest(&project.root_path).unwrap();
    let ctx = DetectionContext::from_deps(deps);

    // With sqlx in deps, should detect DB operations
    assert!(ctx.is_likely_db_operation("query_users"));
    assert!(ctx.is_likely_db_operation("fetch_order"));
    assert!(ctx.is_likely_db_operation("insert_product"));
    assert!(ctx.is_likely_db_operation("execute_query"));

    // But NOT generic names
    assert!(!ctx.is_likely_db_operation("calculate_total"));
    assert!(!ctx.is_likely_db_operation("validate_input"));
}

#[test]
fn test_e2e_detection_context_http_calls() {
    let project = create_http_client_project();

    let deps = ProjectDependencies::from_manifest(&project.root_path).unwrap();
    let ctx = DetectionContext::from_deps(deps);

    // With reqwest in deps, should detect HTTP calls
    // Patterns: send_request, http_get, http_post, call_api, fetch_from, remote_, _client, api_call
    assert!(ctx.is_likely_http_call("send_request"));
    assert!(ctx.is_likely_http_call("http_get_user"));
    assert!(ctx.is_likely_http_call("api_call_service"));
    assert!(ctx.is_likely_http_call("fetch_from_remote"));

    // But NOT generic getter methods (false positive prevention)
    assert!(!ctx.is_likely_http_call("get_user"));
    assert!(!ctx.is_likely_http_call("get_config"));
    assert!(!ctx.is_likely_http_call("fetch_user")); // Too generic
}

#[test]
fn test_e2e_detection_context_cache_operations() {
    let project = create_cache_project();

    let deps = ProjectDependencies::from_manifest(&project.root_path).unwrap();
    let ctx = DetectionContext::from_deps(deps);

    // With redis in deps, should detect cache operations
    assert!(ctx.is_likely_cache_operation("cache_get"));
    assert!(ctx.is_likely_cache_operation("invalidate_cache"));
    assert!(ctx.is_likely_cache_operation("cache_user_data"));
}

#[test]
fn test_e2e_no_deps_no_detection() {
    let project = create_minimal_project();

    let deps = ProjectDependencies::from_manifest(&project.root_path).unwrap();
    let ctx = DetectionContext::from_deps(deps);

    // Without deps, should NOT match anything
    assert!(!ctx.is_likely_db_operation("query_users"));
    assert!(!ctx.is_likely_http_call("send_request"));
    assert!(!ctx.is_likely_cache_operation("cache_get"));
}

// ============================================================================
// Instrumentation Point Detection Tests
// ============================================================================

#[test]
fn test_e2e_instrumentation_points_basic() {
    let project = create_axum_project();
    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];
    let result = analyzer.analyze(&paths).unwrap();

    // Should have instrumentation points
    assert!(
        result.stats.instrumentation_points > 0,
        "Should detect instrumentation points"
    );

    // All endpoints should have instrumentation points
    assert!(
        result.stats.instrumentation_points >= result.stats.endpoints_count,
        "Should have at least as many instrumentation points as endpoints"
    );
}

#[test]
fn test_e2e_instrumentation_points_with_db() {
    let project = create_database_project();
    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];
    let result = analyzer.analyze(&paths).unwrap();

    // Should detect functions
    assert!(
        result.stats.total_functions > 0,
        "Should detect functions in database project"
    );
}

// ============================================================================
// Analysis Statistics Tests
// ============================================================================

#[test]
fn test_e2e_analysis_stats() {
    let project = sample_projects::simple_library();
    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];
    let result = analyzer.analyze(&paths).unwrap();

    // Verify basic stats
    assert!(
        result.stats.total_files >= 1,
        "Should analyze at least 1 file"
    );
    assert!(
        result.stats.total_functions >= 5,
        "Should detect at least 5 functions (add, subtract, multiply, divide, complex_logic)"
    );
    assert!(result.stats.total_lines > 0, "Should count lines of code");
}

#[test]
fn test_e2e_large_codebase_performance() {
    let project = sample_projects::large_codebase();
    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];

    // Should complete in reasonable time
    let start = std::time::Instant::now();
    let result = analyzer.analyze(&paths).unwrap();
    let duration = start.elapsed();

    // Should analyze many files and functions
    assert!(
        result.stats.total_files >= 50,
        "Should analyze at least 50 module files"
    );
    assert!(
        result.stats.total_functions >= 150,
        "Should detect at least 150 functions (3 per module * 50 modules)"
    );

    // Performance check: should complete within 10 seconds
    assert!(
        duration.as_secs() < 10,
        "Analysis should complete in under 10 seconds, took {:?}",
        duration
    );
}

// ============================================================================
// Existing Instrumentation Detection Tests
// ============================================================================

#[test]
fn test_e2e_existing_instrumentation_detection() {
    let project = create_instrumented_project();
    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];
    let result = analyzer.analyze(&paths).unwrap();

    // Should detect functions with and without instrumentation
    assert!(
        result.stats.total_functions >= 4,
        "Should detect at least 4 functions"
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_e2e_invalid_rust_syntax() {
    let project = TestProject::new();
    project.add_cargo_toml(
        r#"
[package]
name = "invalid-syntax"
version = "0.1.0"
edition = "2021"
"#,
    );

    // Add file with invalid syntax
    project.add_source_file(
        "main.rs",
        r#"
fn main() {
    // This is valid
    println!("Hello");
}

fn broken( {
    // This is invalid syntax - missing closing paren
}
"#,
    );

    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];

    // Should handle gracefully (either skip the file or return partial results)
    // The exact behavior depends on implementation
    let _result = analyzer.analyze(&paths);
    // We just verify it doesn't panic
}

#[test]
fn test_e2e_empty_project() {
    let project = TestProject::new();
    project.add_cargo_toml(
        r#"
[package]
name = "empty-project"
version = "0.1.0"
edition = "2021"
"#,
    );

    // Add empty source file
    project.add_source_file("lib.rs", "// Empty file\n");

    let config = Config::default();
    let analyzer = Analyzer::new(config);

    let src_path = project.root_path.join("src");
    let paths = vec![src_path.to_str().unwrap()];
    let result = analyzer.analyze(&paths).unwrap();

    assert_eq!(
        result.stats.total_functions, 0,
        "Empty project should have 0 functions"
    );
}

// ============================================================================
// Helper Functions - Project Creators
// ============================================================================

fn create_axum_project() -> TestProject {
    let project = TestProject::new();

    project.add_cargo_toml(
        r#"
[package]
name = "axum-test"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
    );

    project.add_source_file(
        "main.rs",
        r#"
use axum::{
    routing::{get, post},
    extract::{Path, State, Json},
    http::StatusCode,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct AppState {
    db: String,
}

#[derive(Serialize)]
pub struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
}

#[derive(Serialize)]
pub struct Order {
    id: u64,
    user_id: u64,
    total: f64,
}

// Endpoint handlers
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn get_user(
    Path(user_id): Path<u64>,
    State(_state): State<Arc<AppState>>,
) -> Result<Json<User>, StatusCode> {
    Ok(Json(User {
        id: user_id,
        name: "Test User".to_string(),
    }))
}

pub async fn create_user(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    Ok(Json(User {
        id: 1,
        name: payload.name,
    }))
}

pub async fn list_orders(
    State(_state): State<Arc<AppState>>,
) -> Json<Vec<Order>> {
    Json(vec![
        Order { id: 1, user_id: 1, total: 99.99 },
    ])
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .route("/orders", get(list_orders))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState { db: "postgres://localhost/test".to_string() });
    let app = create_router(state);
    println!("Server starting...");
}
"#,
    );

    project
}

fn create_database_project() -> TestProject {
    let project = TestProject::new();

    project.add_cargo_toml(
        r#"
[package]
name = "db-test"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
tokio = { version = "1", features = ["full"] }
"#,
    );

    project.add_source_file(
        "lib.rs",
        r#"
pub struct DbPool;

pub struct User {
    pub id: i64,
    pub name: String,
}

pub async fn query_users(pool: &DbPool) -> Vec<User> {
    // Database query
    vec![]
}

pub async fn fetch_order(pool: &DbPool, order_id: i64) -> Option<String> {
    // Fetch single order
    None
}

pub async fn insert_product(pool: &DbPool, name: &str) -> Result<i64, String> {
    // Insert new product
    Ok(1)
}

pub async fn execute_query(pool: &DbPool, sql: &str) -> Result<(), String> {
    // Execute raw query
    Ok(())
}

pub fn calculate_total(items: &[f64]) -> f64 {
    items.iter().sum()
}

pub fn validate_input(input: &str) -> bool {
    !input.is_empty()
}
"#,
    );

    project
}

fn create_http_client_project() -> TestProject {
    let project = TestProject::new();

    project.add_cargo_toml(
        r#"
[package]
name = "http-client-test"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
"#,
    );

    project.add_source_file(
        "lib.rs",
        r#"
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiResponse {
    pub data: String,
}

pub struct HttpClient;

pub async fn call_external_api(client: &HttpClient, url: &str) -> Result<ApiResponse, String> {
    // HTTP call to external API
    Ok(ApiResponse { data: "test".to_string() })
}

pub async fn send_request(client: &HttpClient, endpoint: &str) -> Result<String, String> {
    // Send HTTP request
    Ok("response".to_string())
}

pub async fn fetch_remote_data(client: &HttpClient) -> Vec<String> {
    // Fetch data from remote service
    vec![]
}

// These should NOT be detected as HTTP calls
pub fn get_user(id: u64) -> Option<String> {
    Some(format!("user_{}", id))
}

pub fn get_config() -> String {
    "config".to_string()
}
"#,
    );

    project
}

fn create_cache_project() -> TestProject {
    let project = TestProject::new();

    project.add_cargo_toml(
        r#"
[package]
name = "cache-test"
version = "0.1.0"
edition = "2021"

[dependencies]
redis = "0.27"
tokio = { version = "1", features = ["full"] }
"#,
    );

    project.add_source_file(
        "lib.rs",
        r#"
pub struct CacheClient;

pub async fn cache_get(client: &CacheClient, key: &str) -> Option<String> {
    // Get from cache
    None
}

pub async fn cache_set(client: &CacheClient, key: &str, value: &str) -> Result<(), String> {
    // Set in cache
    Ok(())
}

pub async fn invalidate_cache(client: &CacheClient, pattern: &str) -> Result<u64, String> {
    // Invalidate cache keys
    Ok(0)
}

pub async fn cache_user_data(client: &CacheClient, user_id: u64) -> Result<(), String> {
    // Cache user data
    Ok(())
}
"#,
    );

    project
}

fn create_full_stack_project() -> TestProject {
    let project = TestProject::new();

    project.add_cargo_toml(
        r#"
[package]
name = "full-stack-test"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = "0.7"
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Cache
redis = "0.27"

# Observability
tracing = "0.1"
tracing-subscriber = "0.3"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
    );

    project.add_source_file(
        "lib.rs",
        r#"
pub mod handlers;
pub mod db;
pub mod cache;
pub mod external;
"#,
    );

    project.add_source_file(
        "handlers.rs",
        r#"
use axum::{routing::get, Router, Json};

pub async fn health() -> &'static str {
    "OK"
}

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}
"#,
    );

    project.add_source_file(
        "db.rs",
        r#"
pub struct Pool;

pub async fn query_all(pool: &Pool) -> Vec<String> {
    vec![]
}
"#,
    );

    project.add_source_file(
        "cache.rs",
        r#"
pub struct Redis;

pub async fn get_cached(redis: &Redis, key: &str) -> Option<String> {
    None
}
"#,
    );

    project.add_source_file(
        "external.rs",
        r#"
pub struct Client;

pub async fn call_api(client: &Client) -> Result<String, String> {
    Ok("response".to_string())
}
"#,
    );

    project
}

fn create_minimal_project() -> TestProject {
    let project = TestProject::new();

    project.add_cargo_toml(
        r#"
[package]
name = "minimal-test"
version = "0.1.0"
edition = "2021"

[dependencies]
# No dependencies
"#,
    );

    project.add_source_file(
        "lib.rs",
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn query_users() -> Vec<String> {
    // This function name looks like DB operation but project has no DB deps
    vec![]
}

pub fn send_request() -> String {
    // This function name looks like HTTP call but project has no HTTP deps
    String::new()
}

pub fn cache_get() -> Option<String> {
    // This function name looks like cache operation but project has no cache deps
    None
}
"#,
    );

    project
}

fn create_instrumented_project() -> TestProject {
    let project = TestProject::new();

    project.add_cargo_toml(
        r#"
[package]
name = "instrumented-test"
version = "0.1.0"
edition = "2021"

[dependencies]
tracing = "0.1"
"#,
    );

    project.add_source_file(
        "lib.rs",
        r#"
use tracing::{instrument, info, span, Level};

/// This function is instrumented with #[instrument]
#[instrument(name = "api.get_user", skip(id))]
pub fn get_user_instrumented(id: u64) -> String {
    info!("Getting user {}", id);
    format!("user_{}", id)
}

/// This function uses manual span
pub fn process_order_manual_span(order_id: u64) -> Result<(), String> {
    let span = span!(Level::INFO, "process_order", order_id = order_id);
    let _enter = span.enter();

    info!("Processing order");
    Ok(())
}

/// This function has no instrumentation - gap
pub fn calculate_shipping(weight: f64, distance: f64) -> f64 {
    weight * distance * 0.5
}

/// Another uninstrumented function
pub fn validate_address(address: &str) -> bool {
    !address.is_empty() && address.len() > 5
}
"#,
    );

    project
}
