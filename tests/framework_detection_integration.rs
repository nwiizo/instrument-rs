//! Tests for framework detection functionality

use crate::common::sample_projects;
use instrument_rs::framework::detector::FrameworkDetector;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_axum_framework() {
    let project = sample_projects::axum_web_app();
    
    let detector = FrameworkDetector::new();
    
    // Check Cargo.toml for Axum
    let cargo_toml = project.root_path.join("Cargo.toml");
    let framework = detector.detect_from_cargo(&cargo_toml)
        .expect("Should read Cargo.toml");
    
    assert!(framework.is_some(), "Should detect framework from Cargo.toml");
    
    // Should identify Axum patterns
    let main_rs = project.root_path.join("src/main.rs");
    let content = fs::read_to_string(&main_rs).expect("Should read main.rs");
    
    assert!(content.contains("axum::Router"), "Should have Router usage");
    assert!(content.contains("routing::{get, post}"), "Should have routing imports");
    assert!(content.contains("Json"), "Should use Json extractor");
    assert!(content.contains("StatusCode"), "Should use StatusCode");
}

#[test]
fn test_detect_actix_web_framework() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create Actix-web project
    fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "actix-test"
version = "0.1.0"

[dependencies]
actix-web = "4"
actix-rt = "2"
"#,
    ).unwrap();
    
    fs::create_dir_all(project_path.join("src")).unwrap();
    fs::write(
        project_path.join("src/main.rs"),
        r#"
use actix_web::{web, App, HttpResponse, HttpServer, middleware};

async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn create_user(info: web::Json<User>) -> HttpResponse {
    HttpResponse::Ok().json(&info.into_inner())
}

#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    name: String,
}
"#,
    ).unwrap();
    
    let detector = FrameworkDetector::new();
    let cargo_toml = project_path.join("Cargo.toml");
    let framework = detector.detect_from_cargo(&cargo_toml)
        .expect("Should read Cargo.toml");
    
    assert!(framework.is_some(), "Should detect Actix-web framework");
}

#[test]
fn test_detect_rocket_framework() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create Rocket project
    fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "rocket-test"
version = "0.1.0"

[dependencies]
rocket = "0.5"
"#,
    ).unwrap();
    
    fs::create_dir_all(project_path.join("src")).unwrap();
    fs::write(
        project_path.join("src/main.rs"),
        r#"
#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/users", data = "<user>")]
fn create_user(user: Json<User>) -> Json<User> {
    user
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, create_user])
}

#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    name: String,
}

use rocket::serde::json::Json;
"#,
    ).unwrap();
    
    let detector = FrameworkDetector::new();
    let detected = detector.detect_frameworks(project_path)
        .expect("Should detect frameworks");
    
    assert!(detected.web_frameworks.contains(&WebFramework::Rocket),
        "Should detect Rocket framework");
}

#[test]
fn test_detect_warp_framework() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create Warp project
    fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "warp-test"
version = "0.1.0"

[dependencies]
warp = "0.3"
tokio = { version = "1", features = ["full"] }
"#,
    ).unwrap();
    
    fs::create_dir_all(project_path.join("src")).unwrap();
    fs::write(
        project_path.join("src/main.rs"),
        r#"
use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));
    
    let routes = hello
        .or(warp::path::end().map(|| "Hello, World!"))
        .or(api_routes());
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn api_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    users_list()
        .or(users_create())
}

fn users_list() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::get())
        .map(|| warp::reply::json(&vec!["user1", "user2"]))
}

fn users_create() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::post())
        .and(warp::body::json())
        .map(|user: User| warp::reply::json(&user))
}

#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    name: String,
}
"#,
    ).unwrap();
    
    let detector = FrameworkDetector::new();
    let detected = detector.detect_frameworks(project_path)
        .expect("Should detect frameworks");
    
    assert!(detected.web_frameworks.contains(&WebFramework::Warp),
        "Should detect Warp framework");
}

#[test]
fn test_detect_multiple_frameworks() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create project with multiple frameworks (unusual but possible)
    fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "multi-framework"
version = "0.1.0"

[dependencies]
axum = "0.7"
actix-web = "4"
tokio = { version = "1", features = ["full"] }
"#,
    ).unwrap();
    
    fs::create_dir_all(project_path.join("src/bin")).unwrap();
    
    // Axum server
    fs::write(
        project_path.join("src/bin/axum_server.rs"),
        r#"
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Axum server" }));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
"#,
    ).unwrap();
    
    // Actix server
    fs::write(
        project_path.join("src/bin/actix_server.rs"),
        r#"
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().route("/", web::get().to(|| async { "Actix server" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
"#,
    ).unwrap();
    
    let detector = FrameworkDetector::new();
    let detected = detector.detect_frameworks(project_path)
        .expect("Should detect frameworks");
    
    assert!(detected.web_frameworks.len() >= 2,
        "Should detect multiple frameworks");
    assert!(detected.web_frameworks.contains(&WebFramework::Axum),
        "Should detect Axum");
    assert!(detected.web_frameworks.contains(&WebFramework::ActixWeb),
        "Should detect Actix-web");
}

#[test]
fn test_framework_adapter_for_axum() {
    let project = sample_projects::axum_web_app();
    
    let detector = FrameworkDetector::new();
    let detected = detector.detect_frameworks(&project.root_path)
        .expect("Should detect frameworks");
    
    // Get Axum adapter
    let adapter = FrameworkAdapter::for_framework(WebFramework::Axum);
    
    // Test pattern identification
    let main_rs = project.root_path.join("src/main.rs");
    let content = fs::read_to_string(&main_rs).expect("Should read main.rs");
    
    // Should identify route handlers
    let patterns = adapter.identify_patterns(&content);
    assert!(patterns.route_handlers.len() > 0, "Should identify route handlers");
    assert!(patterns.route_handlers.contains(&"root".to_string()));
    assert!(patterns.route_handlers.contains(&"create_user".to_string()));
    assert!(patterns.route_handlers.contains(&"get_user".to_string()));
    
    // Should identify middleware patterns
    assert!(patterns.middleware_usage.is_some() || patterns.extractors.len() > 0,
        "Should identify middleware or extractors");
    
    // Should identify state management
    let handlers_content = fs::read_to_string(project.root_path.join("src/handlers/user.rs"))
        .expect("Should read handlers");
    let handler_patterns = adapter.identify_patterns(&handlers_content);
    assert!(handler_patterns.state_usage.is_some(), "Should identify state usage");
}

#[test]
fn test_framework_specific_instrumentation_points() {
    let project = sample_projects::axum_web_app();
    
    let main_rs = project.root_path.join("src/main.rs");
    let source_file = instrument_rs::ast::SourceFile::parse(&main_rs)
        .expect("Should parse source file");
    let analyzer = instrument_rs::ast::AstAnalyzer::new();
    let ast_result = analyzer.analyze(source_file).expect("Should analyze AST");
    
    // Framework-specific analysis
    let detector = FrameworkDetector::new();
    let detected = detector.detect_frameworks(&project.root_path)
        .expect("Should detect frameworks");
    
    if detected.web_frameworks.contains(&WebFramework::Axum) {
        // Check for Axum-specific patterns in AST
        let route_functions = ast_result.functions.iter()
            .filter(|f| {
                f.is_async && (
                    f.name.contains("handler") ||
                    matches!(f.name.as_str(), "root" | "create_user" | "get_user" | "health_check")
                )
            })
            .count();
        
        assert!(route_functions >= 4, "Should identify at least 4 route handler functions");
        
        // Check for specific Axum types in function signatures
        let functions_with_extractors = ast_result.functions.iter()
            .filter(|f| {
                f.parameters.iter().any(|p| 
                    p.contains("Json") || 
                    p.contains("Path") || 
                    p.contains("State")
                )
            })
            .count();
        
        assert!(functions_with_extractors > 0, "Should identify functions using Axum extractors");
    }
}

#[test]
fn test_detect_test_framework() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create project with test frameworks
    fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "test-framework-detection"
version = "0.1.0"

[dev-dependencies]
tokio-test = "0.4"
proptest = "1.0"
criterion = "0.5"
mockall = "0.11"
"#,
    ).unwrap();
    
    fs::create_dir_all(project_path.join("tests")).unwrap();
    fs::write(
        project_path.join("tests/integration_test.rs"),
        r#"
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_addition_commutative(a: i32, b: i32) {
        assert_eq!(a + b, b + a);
    }
}

#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}

async fn async_function() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
"#,
    ).unwrap();
    
    let detector = FrameworkDetector::new();
    let detected = detector.detect_frameworks(project_path)
        .expect("Should detect frameworks");
    
    // Should detect test frameworks
    assert!(detected.test_frameworks.len() > 0, "Should detect test frameworks");
    assert!(detected.test_frameworks.iter().any(|tf| tf.name == "proptest"),
        "Should detect proptest");
    assert!(detected.test_frameworks.iter().any(|tf| tf.name == "tokio-test"),
        "Should detect tokio-test");
}

#[test]
fn test_no_framework_detection() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create minimal project without frameworks
    fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "no-framework"
version = "0.1.0"
edition = "2021"
"#,
    ).unwrap();
    
    fs::create_dir_all(project_path.join("src")).unwrap();
    fs::write(
        project_path.join("src/main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
}
"#,
    ).unwrap();
    
    let detector = FrameworkDetector::new();
    let detected = detector.detect_frameworks(project_path)
        .expect("Should complete detection");
    
    assert!(detected.web_frameworks.is_empty(), "Should not detect any web frameworks");
    assert!(detected.test_frameworks.is_empty(), "Should not detect any test frameworks");
}