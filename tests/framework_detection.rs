//! Integration tests for framework detection

use instrument_rs::framework::web::{AxumDetector, FrameworkDetector, HttpMethod};
use syn::parse_quote;

#[test]
fn test_axum_endpoint_detection() {
    let detector = AxumDetector::new();

    let syntax_tree: syn::File = parse_quote! {
        use axum::{Router, routing::{get, post, put, delete}};

        async fn list_users() -> Json<Vec<User>> {
            Json(vec![])
        }

        async fn get_user(Path(id): Path<u32>) -> Json<User> {
            Json(User { id })
        }

        async fn create_user(Json(user): Json<CreateUser>) -> StatusCode {
            StatusCode::CREATED
        }

        async fn update_user(Path(id): Path<u32>, Json(user): Json<UpdateUser>) -> StatusCode {
            StatusCode::OK
        }

        async fn delete_user(Path(id): Path<u32>) -> StatusCode {
            StatusCode::NO_CONTENT
        }

        fn create_app() -> Router {
            Router::new()
                .route("/users", get(list_users))
                .route("/users", post(create_user))
                .route("/users/:id", get(get_user))
                .route("/users/:id", put(update_user))
                .route("/users/:id", delete(delete_user))
        }
    };

    let endpoints = detector.extract_endpoints(&syntax_tree);

    // Should find 5 endpoints
    assert_eq!(endpoints.len(), 5);

    // Check first endpoint
    let first = &endpoints[0];
    assert_eq!(first.method, HttpMethod::Get);
    assert_eq!(first.path, "/users");
    assert_eq!(first.handler, "list_users");

    // Check DELETE endpoint
    let delete_endpoint = endpoints
        .iter()
        .find(|e| e.method == HttpMethod::Delete)
        .expect("Should find DELETE endpoint");
    assert_eq!(delete_endpoint.path, "/users/:id");
    assert_eq!(delete_endpoint.handler, "delete_user");
}

#[test]
fn test_axum_handler_detection() {
    let detector = AxumDetector::new();

    // Test async handler with extractors
    let handler: syn::ItemFn = parse_quote! {
        /// Get a specific user by ID
        async fn get_user(
            Path(id): Path<u32>,
            State(db): State<Database>,
            Query(params): Query<Params>
        ) -> Result<Json<User>, StatusCode> {
            Ok(Json(User { id }))
        }
    };

    let info = detector
        .detect_handler(&handler)
        .expect("Should detect handler");

    assert_eq!(info.name, "get_user");
    assert!(info.is_async);
    assert!(info.return_type.contains("Result"));
    assert_eq!(info.parameters.len(), 3);

    // Check extractors
    let extractor_count = info.parameters.iter().filter(|p| p.is_extractor).count();
    assert_eq!(extractor_count, 3);

    // Check documentation
    assert!(info.documentation.is_some());
    assert!(info.documentation.unwrap().contains("Get a specific user"));
}

#[test]
fn test_nested_router_detection() {
    let detector = AxumDetector::new();

    let syntax_tree: syn::File = parse_quote! {
        use axum::{Router, routing::get};

        fn api_routes() -> Router {
            Router::new()
                .route("/health", get(health_check))
                .nest("/v1", v1_routes())
                .nest("/v2", v2_routes())
        }

        fn v1_routes() -> Router {
            Router::new()
                .route("/users", get(v1_list_users))
                .route("/posts", get(v1_list_posts))
        }
    };

    let endpoints = detector.extract_endpoints(&syntax_tree);

    // Should find endpoints from main router
    let health_endpoint = endpoints
        .iter()
        .find(|e| e.path == "/health")
        .expect("Should find health endpoint");
    assert_eq!(health_endpoint.handler, "health_check");
}

#[test]
fn test_middleware_detection() {
    let detector = AxumDetector::new();

    let router_fn: syn::ItemFn = parse_quote! {
        fn create_app() -> Router {
            Router::new()
                .route("/", get(index))
                .layer(cors())
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
        }
    };

    let router_info = detector
        .analyze_router(&syn::Item::Fn(router_fn))
        .expect("Should analyze router");

    // Should detect middleware layers
    assert_eq!(router_info.middleware.len(), 3);
}

#[test]
fn test_http_method_parsing() {
    use instrument_rs::framework::web::HttpMethod;

    assert_eq!(HttpMethod::from_str("GET"), HttpMethod::Get);
    assert_eq!(HttpMethod::from_str("post"), HttpMethod::Post);
    assert_eq!(HttpMethod::from_str("PUT"), HttpMethod::Put);

    // Test custom method
    match HttpMethod::from_str("CUSTOM") {
        HttpMethod::Custom(method) => assert_eq!(method, "CUSTOM"),
        _ => panic!("Expected Custom variant"),
    }
}
