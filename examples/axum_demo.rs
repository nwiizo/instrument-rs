//! Minimal demonstration of Axum framework detection
//!
//! Run with: cargo run --example axum_demo

use syn::parse_quote;

fn main() {
    println!("Axum Framework Detection Demo");
    println!("==============================\n");

    // Sample Axum code to analyze
    let sample_code: syn::File = parse_quote! {
        use axum::{
            Router,
            routing::{get, post},
            extract::{Path, Query, Json},
            response::IntoResponse,
            http::StatusCode,
        };
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct User {
            id: u32,
            name: String,
            email: String,
        }

        /// Get all users
        async fn list_users() -> Json<Vec<User>> {
            Json(vec![
                User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
                User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
            ])
        }

        /// Get a specific user by ID
        async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, StatusCode> {
            if id == 1 {
                Ok(Json(User {
                    id: 1,
                    name: "Alice".into(),
                    email: "alice@example.com".into()
                }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }

        /// Create a new user
        async fn create_user(Json(user): Json<User>) -> (StatusCode, Json<User>) {
            (StatusCode::CREATED, Json(user))
        }

        /// Health check endpoint
        async fn health() -> &'static str {
            "OK"
        }

        pub fn app() -> Router {
            Router::new()
                .route("/health", get(health))
                .route("/users", get(list_users).post(create_user))
                .route("/users/:id", get(get_user))
        }
    };

    // Manual endpoint extraction demonstration
    println!("Analyzing Axum routes...\n");

    // Walk through the AST and find route definitions
    use syn::visit::Visit;

    struct RouteVisitor {
        routes: Vec<(String, String, String)>, // (method, path, handler)
    }

    impl<'ast> Visit<'ast> for RouteVisitor {
        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            let method_name = node.method.to_string();

            // Check if this is a route method
            if matches!(
                method_name.as_str(),
                "get" | "post" | "put" | "delete" | "patch"
            ) {
                if let Some(syn::Expr::Lit(expr_lit)) = node.args.first() {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let path = lit_str.value();
                        let handler = if node.args.len() > 1 {
                            if let Some(syn::Expr::Path(expr_path)) = node.args.iter().nth(1) {
                                quote::quote!(#expr_path).to_string()
                            } else {
                                "unknown".to_string()
                            }
                        } else {
                            "unknown".to_string()
                        };

                        self.routes
                            .push((method_name.to_uppercase(), path, handler));
                    }
                }
            }

            // Continue visiting
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    let mut visitor = RouteVisitor { routes: Vec::new() };
    visitor.visit_file(&sample_code);

    println!("Found {} routes:", visitor.routes.len());
    for (method, path, handler) in &visitor.routes {
        println!("  {} {} -> {}", method, path, handler);
    }

    println!("\nAnalyzing handler functions...\n");

    // Find and analyze handler functions
    for item in &sample_code.items {
        if let syn::Item::Fn(item_fn) = item {
            let name = &item_fn.sig.ident;
            let is_async = item_fn.sig.asyncness.is_some();
            let params = item_fn.sig.inputs.len();

            // Extract documentation
            let mut docs = Vec::new();
            for attr in &item_fn.attrs {
                if attr.path().is_ident("doc") {
                    if let syn::Meta::NameValue(meta) = &attr.meta {
                        if let syn::Expr::Lit(expr_lit) = &meta.value {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                docs.push(lit_str.value());
                            }
                        }
                    }
                }
            }

            println!("Handler: {}", name);
            println!("  Async: {}", is_async);
            println!("  Parameters: {}", params);
            if !docs.is_empty() {
                println!("  Documentation: {}", docs.join(" "));
            }

            // Check for extractors
            for input in &item_fn.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    let ty_str = quote::quote!(#pat_type.ty).to_string();
                    if ty_str.contains("Path")
                        || ty_str.contains("Query")
                        || ty_str.contains("Json")
                        || ty_str.contains("Extension")
                    {
                        println!("  Extractor: {}", ty_str);
                    }
                }
            }

            println!();
        }
    }

    println!("\nDemo complete!");
}
