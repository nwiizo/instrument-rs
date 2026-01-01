//! Common test utilities and helpers for integration tests
#![allow(dead_code)]

use instrument_rs::Config;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a temporary directory with a sample Rust project
pub struct TestProject {
    pub temp_dir: TempDir,
    pub root_path: PathBuf,
}

impl TestProject {
    /// Create a new test project with default structure
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();

        // Create default project structure
        fs::create_dir_all(root_path.join("src")).unwrap();
        fs::create_dir_all(root_path.join("tests")).unwrap();

        Self {
            temp_dir,
            root_path,
        }
    }

    /// Add a source file to the project
    pub fn add_source_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.root_path.join("src").join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
        file_path
    }

    /// Add a test file to the project
    pub fn add_test_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.root_path.join("tests").join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
        file_path
    }

    /// Add a Cargo.toml file
    pub fn add_cargo_toml(&self, content: &str) -> PathBuf {
        let file_path = self.root_path.join("Cargo.toml");
        fs::write(&file_path, content).unwrap();
        file_path
    }

    /// Create a configuration for this test project
    pub fn create_config(&self) -> Config {
        Config::default()
    }
}

/// Sample Rust project generators
pub mod sample_projects {
    use super::*;

    /// Create a simple library project with basic functions
    pub fn simple_library() -> TestProject {
        let project = TestProject::new();

        // Add Cargo.toml
        project.add_cargo_toml(
            r#"
[package]
name = "test-lib"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        );

        // Add lib.rs
        project.add_source_file(
            "lib.rs",
            r#"
//! A simple library for testing

/// Adds two numbers together
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtracts b from a
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiplies two numbers
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Divides a by b, returns None if b is zero
pub fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

/// A more complex function with multiple branches
pub fn complex_logic(x: i32, y: i32) -> Result<i32, String> {
    if x < 0 {
        Err("x must be non-negative".to_string())
    } else if y < 0 {
        Err("y must be non-negative".to_string())
    } else if x > 100 || y > 100 {
        Err("values too large".to_string())
    } else {
        Ok(x * y + x + y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }
    
    #[test]
    fn test_divide() {
        assert_eq!(divide(10, 2), Some(5));
        assert_eq!(divide(10, 0), None);
    }
}
"#,
        );

        // Add integration test
        project.add_test_file(
            "integration_test.rs",
            r#"
use test_lib::*;

#[test]
fn test_complex_logic_integration() {
    assert!(complex_logic(5, 10).is_ok());
    assert!(complex_logic(-1, 5).is_err());
    assert!(complex_logic(5, -1).is_err());
    assert!(complex_logic(101, 5).is_err());
}
"#,
        );

        project
    }

    /// Create an Axum web application project
    pub fn axum_web_app() -> TestProject {
        let project = TestProject::new();

        // Add Cargo.toml
        project.add_cargo_toml(
            r#"
[package]
name = "test-axum-app"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
axum-test = "0.1"
"#,
        );

        // Add main.rs
        project.add_source_file(
            "main.rs",
            r#"
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
    extract::Path,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = create_app();
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub fn create_app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .route("/health", get(health_check))
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1,
        name: payload.name,
        email: payload.email,
    };
    
    (StatusCode::CREATED, Json(user))
}

async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, StatusCode> {
    if id == 1 {
        Ok(Json(User {
            id,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    
    #[tokio::test]
    async fn test_health_check() {
        let app = create_app();
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }
}
"#,
        );

        // Add handlers module
        project.add_source_file(
            "handlers/mod.rs",
            r#"
pub mod user;
pub mod admin;

use axum::response::IntoResponse;
use axum::http::StatusCode;

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Page not found")
}
"#,
        );

        project.add_source_file(
            "handlers/user.rs",
            r#"
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserProfile {
    pub id: u64,
    pub username: String,
    pub created_at: String,
}

pub async fn get_profile(State(state): State<AppState>) -> Json<UserProfile> {
    Json(UserProfile {
        id: 1,
        username: "testuser".to_string(),
        created_at: "2024-01-01".to_string(),
    })
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: String, // Simplified for testing
}
"#,
        );

        project
    }

    /// Create a project with complex code patterns
    pub fn complex_patterns() -> TestProject {
        let project = TestProject::new();

        project.add_cargo_toml(
            r#"
[package]
name = "complex-patterns"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
"#,
        );

        project.add_source_file(
            "lib.rs",
            r#"
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// A trait with async methods
#[async_trait]
pub trait DataStore: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: String, value: String) -> Result<(), StoreError>;
    async fn delete(&self, key: &str) -> Result<bool, StoreError>;
}

#[derive(Debug, Clone)]
pub enum StoreError {
    ConnectionError(String),
    InvalidKey(String),
    InternalError(String),
}

/// In-memory implementation
pub struct MemoryStore {
    data: Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DataStore for MemoryStore {
    async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }
    
    async fn set(&self, key: String, value: String) -> Result<(), StoreError> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey("Key cannot be empty".to_string()));
        }
        
        let mut data = self.data.write().await;
        data.insert(key, value);
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<bool, StoreError> {
        let mut data = self.data.write().await;
        Ok(data.remove(key).is_some())
    }
}

/// Generic processor with complex branching
pub struct DataProcessor<T: DataStore> {
    store: T,
    cache: HashMap<String, String>,
}

impl<T: DataStore> DataProcessor<T> {
    pub fn new(store: T) -> Self {
        Self {
            store,
            cache: HashMap::new(),
        }
    }
    
    pub async fn process(&mut self, input: &str) -> Result<String, ProcessError> {
        // Complex branching logic
        match input.len() {
            0 => return Err(ProcessError::EmptyInput),
            1..=10 => {
                if let Some(cached) = self.cache.get(input) {
                    return Ok(cached.clone());
                }
            }
            11..=50 => {
                let result = self.store.get(input).await;
                if let Some(value) = result {
                    self.cache.insert(input.to_string(), value.clone());
                    return Ok(value);
                }
            }
            _ => {
                return Err(ProcessError::InputTooLarge);
            }
        }
        
        // Process and store
        let processed = input.to_uppercase();
        self.store.set(input.to_string(), processed.clone()).await
            .map_err(|e| ProcessError::StoreError(format!("{:?}", e)))?;
        
        Ok(processed)
    }
}

#[derive(Debug)]
pub enum ProcessError {
    EmptyInput,
    InputTooLarge,
    StoreError(String),
}

/// Recursive function for testing
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

/// Iterator implementation
pub struct FibIterator {
    curr: u64,
    next: u64,
}

impl FibIterator {
    pub fn new() -> Self {
        Self { curr: 0, next: 1 }
    }
}

impl Iterator for FibIterator {
    type Item = u64;
    
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr = self.next;
        self.next = current + self.next;
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_store() {
        let store = MemoryStore::new();
        
        assert!(store.get("key1").await.is_none());
        
        store.set("key1".to_string(), "value1".to_string()).await.unwrap();
        assert_eq!(store.get("key1").await, Some("value1".to_string()));
        
        assert!(store.delete("key1").await.unwrap());
        assert!(store.get("key1").await.is_none());
    }
    
    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }
}
"#,
        );

        project
    }

    /// Create a large project for performance testing
    pub fn large_codebase() -> TestProject {
        let project = TestProject::new();

        project.add_cargo_toml(
            r#"
[package]
name = "large-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        );

        // Generate many module files
        for i in 0..50 {
            let module_content = format!(
                r#"
//! Module {i} documentation

/// Function {i}_a
pub fn function_{i}_a(x: i32) -> i32 {{
    if x > 0 {{
        x * 2
    }} else {{
        x * -2
    }}
}}

/// Function {i}_b with more complexity
pub fn function_{i}_b(x: i32, y: i32) -> Result<i32, String> {{
    match (x, y) {{
        (0, 0) => Err("both zero".to_string()),
        (0, _) => Ok(y),
        (_, 0) => Ok(x),
        (x, y) if x > y => Ok(x - y),
        (x, y) if x < y => Ok(y - x),
        _ => Ok(0),
    }}
}}

/// Function {i}_c with loops
pub fn function_{i}_c(n: usize) -> Vec<i32> {{
    let mut result = Vec::with_capacity(n);
    for i in 0..n {{
        if i % 2 == 0 {{
            result.push(i as i32);
        }} else {{
            result.push(-(i as i32));
        }}
    }}
    result
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_function_{i}_a() {{
        assert_eq!(function_{i}_a(5), 10);
        assert_eq!(function_{i}_a(-5), 10);
    }}
    
    #[test]
    fn test_function_{i}_b() {{
        assert!(function_{i}_b(0, 0).is_err());
        assert_eq!(function_{i}_b(5, 3).unwrap(), 2);
    }}
}}
"#,
                i = i
            );

            project.add_source_file(&format!("module_{}.rs", i), &module_content);
        }

        // Create lib.rs that exports all modules
        let mut lib_content = String::from("//! Large codebase for performance testing\n\n");
        for i in 0..50 {
            lib_content.push_str(&format!("pub mod module_{};\n", i));
        }

        project.add_source_file("lib.rs", &lib_content);

        project
    }
}

/// Helper function to create a sample config file
pub fn create_sample_config_file(path: &Path, config_type: &str) -> PathBuf {
    let content = match config_type {
        "minimal" => {
            r#"
[project]
root_dir = "."
source_dirs = ["src"]
test_dirs = ["tests"]

[instrumentation]
mode = "coverage"
"#
        }
        "full" => {
            r#"
[project]
root_dir = "."
source_dirs = ["src", "lib"]
test_dirs = ["tests", "benches"]
exclude_patterns = ["target/**", "**/*.rs.bk", "**/generated/**"]
target_dir = "target"

[instrumentation]
mode = "combined"
preserve_originals = true
output_dir = "target/instrumented"
parallel = true
threads = 4

[mutation]
operators = [
    "arithmetic_operator_replacement",
    "comparison_operator_replacement",
    "logical_operator_replacement",
    "statement_deletion",
]
max_mutations_per_file = 50
timeout_seconds = 60
seed = 12345

[reporting]
formats = ["html", "json", "console"]
output_dir = "target/reports"
include_source = true
coverage_threshold = 85.0
mutation_threshold = 70.0
"#
        }
        "invalid" => {
            r#"
[project]
# Missing required fields

[instrumentation
# Invalid TOML syntax
"#
        }
        _ => panic!("Unknown config type: {}", config_type),
    };

    let file_path = path.join(format!("{}_config.toml", config_type));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Assertion helpers
pub mod assertions {
    use std::path::Path;

    /// Assert that a file exists
    pub fn assert_file_exists(path: &Path) {
        assert!(path.exists(), "Expected file to exist: {:?}", path);
    }

    /// Assert that a file contains specific content
    pub fn assert_file_contains(path: &Path, expected: &str) {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read file: {:?}", path));
        assert!(
            content.contains(expected),
            "File {:?} does not contain expected content: {}",
            path,
            expected
        );
    }

    /// Assert that a directory exists and contains files
    pub fn assert_dir_not_empty(path: &Path) {
        assert!(path.is_dir(), "Expected directory: {:?}", path);
        let entries = std::fs::read_dir(path)
            .unwrap_or_else(|_| panic!("Failed to read directory: {:?}", path));
        assert!(entries.count() > 0, "Directory is empty: {:?}", path);
    }
}
