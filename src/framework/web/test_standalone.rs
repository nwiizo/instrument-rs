//! Standalone test module for web framework detection
//!
//! This module tests the web framework detection functionality
//! in isolation from the rest of the codebase.

#[cfg(test)]
mod tests {
    use crate::framework::web::{AxumDetector, FrameworkDetector, HttpMethod};
    use syn::parse_quote;

    #[test]
    fn test_axum_detector_name() {
        let detector = AxumDetector::new();
        assert_eq!(detector.name(), "Axum");
    }

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::from_str("GET"), HttpMethod::Get);
        assert_eq!(HttpMethod::from_str("post"), HttpMethod::Post);
    }

    #[test]
    fn test_axum_extractor_detection() {
        let detector = AxumDetector::new();
        assert!(detector.is_extractor("Json<User>"));
        assert!(detector.is_extractor("Path<u32>"));
        assert!(detector.is_extractor("Query<Params>"));
        assert!(detector.is_extractor("State<AppState>"));
        assert!(!detector.is_extractor("u32"));
        assert!(!detector.is_extractor("String"));
    }

    #[test]
    fn test_simple_endpoint_extraction() {
        let detector = AxumDetector::new();

        let syntax_tree: syn::File = parse_quote! {
            use axum::{Router, routing::get};

            async fn hello() -> &'static str {
                "Hello, World!"
            }

            fn app() -> Router {
                Router::new()
                    .route("/", get(hello))
            }
        };

        let endpoints = detector.extract_endpoints(&syntax_tree);
        assert_eq!(endpoints.len(), 1);

        let endpoint = &endpoints[0];
        assert_eq!(endpoint.method, HttpMethod::Get);
        assert_eq!(endpoint.path, "/");
        assert_eq!(endpoint.handler, "hello");
    }

    #[test]
    fn test_multiple_methods_extraction() {
        let detector = AxumDetector::new();

        let syntax_tree: syn::File = parse_quote! {
            use axum::{Router, routing::{get, post, put, delete}};

            fn create_router() -> Router {
                Router::new()
                    .route("/users", get(list_users))
                    .route("/users", post(create_user))
                    .route("/users/:id", put(update_user))
                    .route("/users/:id", delete(delete_user))
            }
        };

        let endpoints = detector.extract_endpoints(&syntax_tree);
        assert_eq!(endpoints.len(), 4);

        // Verify all methods are captured
        let methods: Vec<HttpMethod> = endpoints.iter().map(|e| e.method).collect();
        assert!(methods.contains(&HttpMethod::Get));
        assert!(methods.contains(&HttpMethod::Post));
        assert!(methods.contains(&HttpMethod::Put));
        assert!(methods.contains(&HttpMethod::Delete));
    }

    #[test]
    fn test_handler_detection_async() {
        let detector = AxumDetector::new();

        let handler: syn::ItemFn = parse_quote! {
            async fn get_user(Path(id): Path<u32>) -> Json<User> {
                Json(User { id })
            }
        };

        let info = detector.detect_handler(&handler).unwrap();
        assert_eq!(info.name, "get_user");
        assert!(info.is_async);
        assert!(info.return_type.contains("Json"));
        assert_eq!(info.parameters.len(), 1);
    }

    #[test]
    fn test_handler_detection_sync() {
        let detector = AxumDetector::new();

        let handler: syn::ItemFn = parse_quote! {
            fn health_check() -> &'static str {
                "OK"
            }
        };

        let info = detector.detect_handler(&handler).unwrap();
        assert_eq!(info.name, "health_check");
        assert!(!info.is_async);
        assert_eq!(info.parameters.len(), 0);
    }

    #[test]
    fn test_handler_with_multiple_extractors() {
        let detector = AxumDetector::new();

        let handler: syn::ItemFn = parse_quote! {
            async fn complex_handler(
                Path(id): Path<u32>,
                Query(params): Query<SearchParams>,
                Json(body): Json<UpdateRequest>,
                Extension(user): Extension<User>
            ) -> Result<Json<Response>, StatusCode> {
                Ok(Json(Response::default()))
            }
        };

        let info = detector.detect_handler(&handler).unwrap();
        assert_eq!(info.parameters.len(), 4);

        let extractor_count = info.parameters.iter().filter(|p| p.is_extractor).count();
        assert_eq!(extractor_count, 4);
    }

    #[test]
    fn test_router_analysis() {
        let detector = AxumDetector::new();

        let router_fn: syn::ItemFn = parse_quote! {
            fn create_app() -> Router {
                Router::new()
                    .route("/", get(index))
                    .route("/users", get(list_users))
                    .route("/users/:id", get(get_user))
                    .nest("/api", api_routes())
                    .layer(cors())
            }
        };

        let router_info = detector.analyze_router(&syn::Item::Fn(router_fn)).unwrap();
        assert_eq!(router_info.routes.len(), 3);
        assert!(!router_info.middleware.is_empty());
    }
}
