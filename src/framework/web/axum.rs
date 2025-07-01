//! Axum web framework detector and analyzer
//!
//! This module implements the `FrameworkDetector` trait for the Axum web framework,
//! providing functionality to detect Axum usage and extract HTTP endpoint information.

use super::{
    Endpoint, FrameworkDetector, HandlerInfo, HandlerParam, HttpMethod, RouteInfo, RouterInfo,
};
use crate::{Error, Result};
use std::fs;
use std::path::Path;
use syn::{
    visit::Visit, Attribute, Expr, ExprMethodCall, Item, ItemFn, Lit, Meta,
    Pat, ReturnType, Signature,
};

/// Detector for the Axum web framework
pub struct AxumDetector {
    /// Known Axum extractor types
    extractors: Vec<String>,
}

impl AxumDetector {
    /// Create a new Axum detector
    pub fn new() -> Self {
        Self {
            extractors: vec![
                "Json".to_string(),
                "Path".to_string(),
                "Query".to_string(),
                "Header".to_string(),
                "Extension".to_string(),
                "Form".to_string(),
                "Bytes".to_string(),
                "String".to_string(),
                "State".to_string(),
                "ConnectInfo".to_string(),
                "Host".to_string(),
                "OriginalUri".to_string(),
                "RawQuery".to_string(),
                "Request".to_string(),
                "TypedHeader".to_string(),
                "WebSocketUpgrade".to_string(),
            ],
        }
    }

    /// Check if a type is an Axum extractor
    pub fn is_extractor(&self, ty: &str) -> bool {
        self.extractors.iter().any(|e| ty.contains(e))
    }

    /// Extract HTTP method from a method call
    fn extract_method_from_call(&self, method_name: &str) -> Option<HttpMethod> {
        match method_name {
            "get" => Some(HttpMethod::Get),
            "post" => Some(HttpMethod::Post),
            "put" => Some(HttpMethod::Put),
            "delete" => Some(HttpMethod::Delete),
            "patch" => Some(HttpMethod::Patch),
            "head" => Some(HttpMethod::Head),
            "options" => Some(HttpMethod::Options),
            "trace" => Some(HttpMethod::Trace),
            _ => None,
        }
    }

    /// Parse handler parameters from function signature
    fn parse_handler_params(&self, sig: &Signature) -> Vec<HandlerParam> {
        let mut params = Vec::new();

        for input in &sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let name = pat_ident.ident.to_string();
                    let ty = quote::quote!(#pat_type.ty).to_string();
                    let is_extractor = self.is_extractor(&ty);

                    params.push(HandlerParam {
                        name,
                        ty,
                        is_extractor,
                    });
                }
            }
        }

        params
    }

    /// Extract return type as string
    fn extract_return_type(&self, ret: &ReturnType) -> String {
        match ret {
            ReturnType::Default => "()".to_string(),
            ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
        }
    }

    /// Extract documentation from attributes
    fn extract_docs(&self, attrs: &[Attribute]) -> Option<String> {
        let mut docs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            docs.push(lit_str.value().trim().to_string());
                        }
                    }
                }
            }
        }

        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }
}

impl FrameworkDetector for AxumDetector {
    fn name(&self) -> &'static str {
        "Axum"
    }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        // Check Cargo.toml for axum dependency
        let cargo_toml_path = project_root.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(false);
        }

        let cargo_content = fs::read_to_string(&cargo_toml_path).map_err(|e| Error::Io(e))?;

        // Check for axum in dependencies
        let cargo_toml: toml::Value = toml::from_str(&cargo_content).map_err(|e| Error::Toml(e))?;

        if let Some(deps) = cargo_toml.get("dependencies") {
            if deps.get("axum").is_some() {
                return Ok(true);
            }
        }

        if let Some(deps) = cargo_toml.get("dev-dependencies") {
            if deps.get("axum").is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn extract_endpoints(&self, syntax_tree: &syn::File) -> Vec<Endpoint> {
        let mut visitor = EndpointVisitor::new(self);
        visitor.visit_file(syntax_tree);
        visitor.endpoints
    }

    fn analyze_router(&self, item: &Item) -> Option<RouterInfo> {
        match item {
            Item::Fn(item_fn) => {
                let mut visitor = RouterVisitor::new(self);
                visitor.visit_item_fn(item_fn);
                visitor.router_info
            }
            Item::Impl(item_impl) => {
                let mut visitor = RouterVisitor::new(self);
                visitor.visit_item_impl(item_impl);
                visitor.router_info
            }
            _ => None,
        }
    }

    fn detect_handler(&self, function: &ItemFn) -> Option<HandlerInfo> {
        let sig = &function.sig;

        // Check if function signature matches Axum handler pattern
        // Handlers typically return impl IntoResponse or a specific response type
        let return_type = self.extract_return_type(&sig.output);
        let is_handler = return_type.contains("IntoResponse")
            || return_type.contains("Response")
            || return_type.contains("Result")
            || return_type.contains("Json")
            || return_type.contains("Html")
            || return_type.contains("StatusCode");

        if !is_handler && return_type != "()" {
            return None;
        }

        let params = self.parse_handler_params(sig);
        let extractors: Vec<String> = params
            .iter()
            .filter(|p| p.is_extractor)
            .map(|p| p.ty.clone())
            .collect();

        Some(HandlerInfo {
            name: sig.ident.to_string(),
            is_async: sig.asyncness.is_some(),
            parameters: params,
            return_type,
            extractors,
            documentation: self.extract_docs(&function.attrs),
        })
    }
}

/// Visitor for finding HTTP endpoints in the AST
struct EndpointVisitor<'a> {
    detector: &'a AxumDetector,
    endpoints: Vec<Endpoint>,
    current_module: Vec<String>,
}

impl<'a> EndpointVisitor<'a> {
    fn new(detector: &'a AxumDetector) -> Self {
        Self {
            detector,
            endpoints: Vec::new(),
            current_module: Vec::new(),
        }
    }

    /// Process a router method call to extract endpoint information
    fn process_router_call(&mut self, expr: &ExprMethodCall) {
        if let Some(method) = self
            .detector
            .extract_method_from_call(&expr.method.to_string())
        {
            // Extract path from first argument
            let path = if let Some(arg) = expr.args.first() {
                if let Expr::Lit(expr_lit) = arg {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        Some(lit_str.value())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Extract handler from second argument
            let handler = if expr.args.len() > 1 {
                if let Some(Expr::Path(expr_path)) = expr.args.iter().nth(1) {
                    Some(quote::quote!(#expr_path).to_string())
                } else {
                    None
                }
            } else {
                None
            };

            if let (Some(path), Some(handler)) = (path, handler) {
                self.endpoints.push(Endpoint {
                    method,
                    path,
                    handler,
                    module_path: self.current_module.clone(),
                    line: 0, // Would need span information for accurate line numbers
                    middleware: Vec::new(),
                    documentation: None,
                });
            }
        }
    }
}

impl<'a> Visit<'_> for EndpointVisitor<'a> {
    fn visit_expr_method_call(&mut self, expr: &ExprMethodCall) {
        // Look for router method calls like router.get("/path", handler)
        self.process_router_call(expr);

        // Continue visiting nested expressions
        syn::visit::visit_expr_method_call(self, expr);
    }

    fn visit_item_mod(&mut self, module: &syn::ItemMod) {
        // Track module path
        self.current_module.push(module.ident.to_string());
        syn::visit::visit_item_mod(self, module);
        self.current_module.pop();
    }
}

/// Visitor for analyzing router configurations
struct RouterVisitor<'a> {
    detector: &'a AxumDetector,
    router_info: Option<RouterInfo>,
    current_router: Option<String>,
}

impl<'a> RouterVisitor<'a> {
    fn new(detector: &'a AxumDetector) -> Self {
        Self {
            detector,
            router_info: None,
            current_router: None,
        }
    }

    /// Process Router::new() calls
    fn process_router_new(&mut self, expr: &ExprMethodCall) {
        if let Expr::Path(path) = &*expr.receiver {
            let path_str = quote::quote!(#path).to_string();
            if path_str.contains("Router") && expr.method == "new" {
                // Found Router::new()
                self.router_info = Some(RouterInfo {
                    name: "router".to_string(),
                    base_path: None,
                    routes: Vec::new(),
                    middleware: Vec::new(),
                    nested_routers: Vec::new(),
                });
            }
        }
    }

    /// Process route registration calls
    fn process_route_registration(&mut self, expr: &ExprMethodCall) {
        if let Some(router_info) = &mut self.router_info {
            if let Some(method) = self
                .detector
                .extract_method_from_call(&expr.method.to_string())
            {
                // Extract path and handler
                if expr.args.len() >= 2 {
                    let path = if let Expr::Lit(expr_lit) = &expr.args[0] {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            Some(lit_str.value())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let handler = if let Expr::Path(expr_path) = &expr.args[1] {
                        Some(quote::quote!(#expr_path).to_string())
                    } else {
                        None
                    };

                    if let (Some(path), Some(handler)) = (path, handler) {
                        router_info.routes.push(RouteInfo {
                            method,
                            path,
                            handler,
                            middleware: Vec::new(),
                        });
                    }
                }
            }

            // Check for nest() calls
            if expr.method == "nest" && expr.args.len() >= 2 {
                if let Expr::Lit(expr_lit) = &expr.args[0] {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        router_info.base_path = Some(lit_str.value());
                    }
                }
            }

            // Check for layer() calls (middleware)
            if expr.method == "layer" {
                router_info.middleware.push("layer".to_string());
            }
        }
    }
}

impl<'a> Visit<'_> for RouterVisitor<'a> {
    fn visit_expr_method_call(&mut self, expr: &ExprMethodCall) {
        self.process_router_new(expr);
        self.process_route_registration(expr);

        syn::visit::visit_expr_method_call(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_axum_detector_creation() {
        let detector = AxumDetector::new();
        assert_eq!(detector.name(), "Axum");
        assert!(detector.is_extractor("Json<User>"));
        assert!(detector.is_extractor("Path<String>"));
        assert!(!detector.is_extractor("String"));
    }

    #[test]
    fn test_method_extraction() {
        let detector = AxumDetector::new();
        assert_eq!(
            detector.extract_method_from_call("get"),
            Some(HttpMethod::Get)
        );
        assert_eq!(
            detector.extract_method_from_call("post"),
            Some(HttpMethod::Post)
        );
        assert_eq!(detector.extract_method_from_call("unknown"), None);
    }

    #[test]
    fn test_handler_detection() {
        let detector = AxumDetector::new();

        let handler: ItemFn = parse_quote! {
            async fn get_user(Path(id): Path<u32>) -> Json<User> {
                Json(User { id })
            }
        };

        let info = detector.detect_handler(&handler).unwrap();
        assert_eq!(info.name, "get_user");
        assert!(info.is_async);
        assert_eq!(info.parameters.len(), 1);
        assert!(info.return_type.contains("Json"));
    }

    #[test]
    fn test_endpoint_extraction() {
        let detector = AxumDetector::new();

        let syntax_tree: syn::File = parse_quote! {
            use axum::{Router, routing::get};

            async fn hello() -> &'static str {
                "Hello, World!"
            }

            fn app() -> Router {
                Router::new()
                    .route("/", get(hello))
                    .route("/users", post(create_user))
            }
        };

        let endpoints = detector.extract_endpoints(&syntax_tree);
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].path, "/");
        assert_eq!(endpoints[0].method, HttpMethod::Get);
        assert_eq!(endpoints[0].handler, "hello");
    }
}
