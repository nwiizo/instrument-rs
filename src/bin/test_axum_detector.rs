//! Standalone test binary for Axum framework detection
//! 
//! This demonstrates the web framework detection functionality
//! without depending on the rest of the instrument-rs codebase.

use syn::parse_quote;

fn main() {
    println!("Axum Framework Detection Test");
    println!("=============================\n");
    
    // Test HTTP method parsing
    test_http_methods();
    
    // Test endpoint extraction
    test_endpoint_extraction();
    
    // Test handler detection
    test_handler_detection();
    
    println!("\nAll tests completed successfully!");
}

fn test_http_methods() {
    println!("Testing HTTP method parsing:");
    
    let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "CUSTOM"];
    
    for method in methods {
        println!("  {} -> {:?}", method, parse_http_method(method));
    }
    println!();
}

fn parse_http_method(s: &str) -> &'static str {
    match s.to_uppercase().as_str() {
        "GET" => "GET",
        "POST" => "POST",
        "PUT" => "PUT",
        "DELETE" => "DELETE",
        "PATCH" => "PATCH",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        "CONNECT" => "CONNECT",
        "TRACE" => "TRACE",
        _ => "CUSTOM",
    }
}

fn test_endpoint_extraction() {
    println!("Testing endpoint extraction:");
    
    let code: syn::File = parse_quote! {
        use axum::{Router, routing::{get, post}};
        
        async fn health() -> &'static str { "OK" }
        async fn list_users() -> Json<Vec<User>> { Json(vec![]) }
        async fn create_user(Json(user): Json<User>) -> StatusCode { StatusCode::CREATED }
        
        fn app() -> Router {
            Router::new()
                .route("/health", get(health))
                .route("/users", get(list_users).post(create_user))
        }
    };
    
    let endpoints = extract_endpoints(&code);
    
    println!("  Found {} endpoints:", endpoints.len());
    for (method, path, handler) in endpoints {
        println!("    {} {} -> {}", method, path, handler);
    }
    println!();
}

fn extract_endpoints(file: &syn::File) -> Vec<(String, String, String)> {
    use syn::visit::Visit;
    
    struct EndpointExtractor {
        endpoints: Vec<(String, String, String)>,
    }
    
    impl<'ast> Visit<'ast> for EndpointExtractor {
        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            let method_name = node.method.to_string();
            
            if matches!(method_name.as_str(), "get" | "post" | "put" | "delete" | "patch") {
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
                        
                        self.endpoints.push((method_name.to_uppercase(), path, handler));
                    }
                }
            }
            
            syn::visit::visit_expr_method_call(self, node);
        }
    }
    
    let mut extractor = EndpointExtractor { endpoints: Vec::new() };
    extractor.visit_file(file);
    extractor.endpoints
}

fn test_handler_detection() {
    println!("Testing handler detection:");
    
    let handlers: Vec<syn::ItemFn> = vec![
        parse_quote! {
            /// Get user by ID
            async fn get_user(Path(id): Path<u32>) -> Json<User> {
                Json(User { id })
            }
        },
        parse_quote! {
            fn health_check() -> &'static str {
                "OK"
            }
        },
        parse_quote! {
            async fn complex_handler(
                Path(id): Path<u32>,
                Query(params): Query<SearchParams>,
                Json(body): Json<UpdateRequest>
            ) -> Result<Json<Response>, StatusCode> {
                Ok(Json(Response::default()))
            }
        },
    ];
    
    for handler in handlers {
        analyze_handler(&handler);
    }
}

fn analyze_handler(handler: &syn::ItemFn) {
    let name = &handler.sig.ident;
    let is_async = handler.sig.asyncness.is_some();
    let params = handler.sig.inputs.len();
    
    println!("\n  Handler: {}", name);
    println!("    Async: {}", is_async);
    println!("    Parameters: {}", params);
    
    // Extract documentation
    for attr in &handler.attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        println!("    Documentation: {}", lit_str.value().trim());
                    }
                }
            }
        }
    }
    
    // Check for extractors
    for input in &handler.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let ty_str = quote::quote!(#pat_type.ty).to_string();
            if ty_str.contains("Path") || ty_str.contains("Query") || 
               ty_str.contains("Json") || ty_str.contains("State") {
                println!("    Extractor: {}", ty_str);
            }
        }
    }
}