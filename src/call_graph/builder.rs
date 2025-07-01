//! Call graph builder that traces function calls from endpoints

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use syn::{
    visit::Visit, Expr, ExprCall, ExprMethodCall, Item, ItemFn,
};
use thiserror::Error;
use walkdir::WalkDir;

use super::{CallEdge, CallGraph, CallKind, FunctionNode, NodeKind, SymbolResolver};
use super::edge::CallContext;

/// Errors that can occur during graph building
#[derive(Error, Debug)]
pub enum GraphBuildError {
    /// IO error while reading files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Error parsing Rust syntax
    #[error("Parse error in {file}: {error}")]
    Parse {
        /// File that failed to parse
        file: String,
        /// Parse error details
        error: String,
    },
    
    /// Symbol resolution error
    #[error("Failed to resolve symbol: {0}")]
    SymbolResolution(String),
    
    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Builds call graphs from Rust source code
pub struct GraphBuilder {
    /// The call graph being built
    graph: CallGraph,
    /// Symbol resolver for function references
    resolver: SymbolResolver,
    /// Current file being processed
    current_file: Option<PathBuf>,
    /// Stack of function contexts during traversal
    context_stack: Vec<FunctionContext>,
    /// Set of processed files
    processed_files: HashSet<PathBuf>,
}

/// Context for the current function being analyzed
#[derive(Debug, Clone)]
struct FunctionContext {
    /// Function ID
    function_id: String,
    /// Whether we're in a conditional block
    in_conditional: bool,
    /// Whether we're in a loop
    in_loop: bool,
    /// Current block depth
    block_depth: usize,
}

impl GraphBuilder {
    /// Creates a new graph builder
    pub fn new() -> Self {
        Self {
            graph: CallGraph::new(),
            resolver: SymbolResolver::new(),
            current_file: None,
            context_stack: Vec::new(),
            processed_files: HashSet::new(),
        }
    }
    
    /// Builds a call graph from a directory of Rust source files
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root directory to analyze
    ///
    /// # Returns
    ///
    /// The constructed call graph
    ///
    /// # Errors
    ///
    /// Returns an error if file reading or parsing fails
    pub fn build_from_directory(&mut self, root_path: &Path) -> Result<CallGraph, GraphBuildError> {
        // First pass: collect all function definitions
        self.collect_definitions(root_path)?;
        
        // Second pass: trace function calls
        self.trace_calls(root_path)?;
        
        // Post-process: identify unreachable nodes, external boundaries, etc.
        self.post_process();
        
        Ok(self.graph.clone())
    }
    
    /// Collects all function definitions in the codebase
    fn collect_definitions(&mut self, root_path: &Path) -> Result<(), GraphBuildError> {
        for entry in WalkDir::new(root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.process_file_definitions(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Processes a single file to extract function definitions
    fn process_file_definitions(&mut self, file_path: &Path) -> Result<(), GraphBuildError> {
        if self.processed_files.contains(file_path) {
            return Ok(());
        }
        
        let content = std::fs::read_to_string(file_path)?;
        let file = syn::parse_file(&content).map_err(|e| GraphBuildError::Parse {
            file: file_path.display().to_string(),
            error: e.to_string(),
        })?;
        
        self.current_file = Some(file_path.to_path_buf());
        
        // Extract module path from file path (simplified)
        let module_path = self.extract_module_path(file_path);
        
        // Visit all items to collect definitions
        for item in &file.items {
            self.process_item_definitions(item, module_path.clone(), file_path);
        }
        
        self.processed_files.insert(file_path.to_path_buf());
        Ok(())
    }
    
    /// Processes an item to extract function definitions
    fn process_item_definitions(&mut self, item: &Item, module_path: Vec<String>, file_path: &Path) {
        match item {
            Item::Fn(item_fn) => {
                let node = FunctionNode::from_item_fn(
                    item_fn,
                    module_path,
                    Some(file_path.display().to_string()),
                );
                self.graph.add_node(node);
                self.resolver.process_item(item, &file_path.to_path_buf());
            }
            Item::Mod(item_mod) => {
                let mut new_module_path = module_path;
                new_module_path.push(item_mod.ident.to_string());
                
                if let Some(content) = &item_mod.content {
                    for item in &content.1 {
                        self.process_item_definitions(item, new_module_path.clone(), file_path);
                    }
                }
            }
            Item::Use(_) => {
                self.resolver.process_item(item, &file_path.to_path_buf());
            }
            _ => {}
        }
    }
    
    /// Traces function calls in the codebase
    fn trace_calls(&mut self, root_path: &Path) -> Result<(), GraphBuildError> {
        // Reset processed files for second pass
        self.processed_files.clear();
        
        for entry in WalkDir::new(root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.process_file_calls(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Processes a single file to trace function calls
    fn process_file_calls(&mut self, file_path: &Path) -> Result<(), GraphBuildError> {
        if self.processed_files.contains(file_path) {
            return Ok(());
        }
        
        let content = std::fs::read_to_string(file_path)?;
        let file = syn::parse_file(&content).map_err(|e| GraphBuildError::Parse {
            file: file_path.display().to_string(),
            error: e.to_string(),
        })?;
        
        self.current_file = Some(file_path.to_path_buf());
        
        // Create a call tracer visitor
        let mut tracer = CallTracer::new(self);
        tracer.visit_file(&file);
        
        self.processed_files.insert(file_path.to_path_buf());
        Ok(())
    }
    
    /// Extracts module path from file path
    fn extract_module_path(&self, file_path: &Path) -> Vec<String> {
        // Simplified: extract from src/... path
        let path_str = file_path.display().to_string();
        if let Some(src_idx) = path_str.find("/src/") {
            let module_part = &path_str[src_idx + 5..];
            let module_part = module_part.trim_end_matches(".rs");
            let module_part = module_part.replace("/", "::");
            
            if module_part == "main" || module_part == "lib" {
                vec![]
            } else {
                module_part.split("::").map(String::from).collect()
            }
        } else {
            vec![]
        }
    }
    
    /// Post-processes the graph to identify external boundaries and other properties
    fn post_process(&mut self) {
        // Identify external function calls that don't have nodes in the graph
        let mut external_nodes = Vec::new();
        
        for edge in self.graph.edges().clone() {
            if self.graph.get_node(&edge.to).is_none() {
                // Create external node
                let path: syn::Path = syn::parse_str(&edge.to).unwrap_or_else(|_| {
                    syn::parse_quote!(unknown::function)
                });
                external_nodes.push(FunctionNode::external(&path));
            }
        }
        
        // Add external nodes to the graph
        for node in external_nodes {
            self.graph.add_node(node);
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor for tracing function calls
struct CallTracer<'a> {
    builder: &'a mut GraphBuilder,
}

impl<'a> CallTracer<'a> {
    fn new(builder: &'a mut GraphBuilder) -> Self {
        Self { builder }
    }
    
    /// Gets the current function context
    fn current_function(&self) -> Option<&FunctionContext> {
        self.builder.context_stack.last()
    }
    
    /// Processes a function call
    fn process_call(&mut self, expr: &ExprCall) {
        if let Some(current_fn) = self.current_function() {
            // Try to resolve the called function
            if let Expr::Path(expr_path) = &*expr.func {
                if let Some(resolved) = self.builder.resolver.resolve_path(&expr_path.path) {
                    let kind = if resolved.is_external {
                        CallKind::Direct
                    } else if current_fn.function_id == resolved.full_path {
                        CallKind::Recursive
                    } else {
                        CallKind::Direct
                    };
                    
                    let edge = CallEdge::new(
                        current_fn.function_id.clone(),
                        resolved.full_path,
                        kind,
                    )
                    .with_conditional(current_fn.in_conditional)
                    .with_in_loop(current_fn.in_loop)
                    .with_context(CallContext::Direct);
                    
                    // Add node if it doesn't exist (for external functions)
                    if self.builder.graph.get_node(&edge.to).is_none() && resolved.is_external {
                        let external_node = FunctionNode::external(&expr_path.path);
                        self.builder.graph.add_node(external_node);
                    }
                    
                    // Add edge if both nodes exist
                    if self.builder.graph.get_node(&edge.from).is_some() 
                        && self.builder.graph.get_node(&edge.to).is_some() {
                        self.builder.graph.add_edge(edge);
                    }
                }
            }
        }
        
        // Continue visiting arguments
        self.visit_expr(&expr.func);
        for arg in &expr.args {
            self.visit_expr(arg);
        }
    }
    
    /// Processes a method call
    fn process_method_call(&mut self, expr: &ExprMethodCall) {
        if let Some(current_fn) = self.current_function() {
            // For method calls, we need more sophisticated resolution
            // For now, create a placeholder
            let method_name = expr.method.to_string();
            let edge = CallEdge::new(
                current_fn.function_id.clone(),
                method_name,
                CallKind::Trait,
            )
            .with_conditional(current_fn.in_conditional)
            .with_in_loop(current_fn.in_loop)
            .with_context(CallContext::Method);
            
            // We might not be able to resolve the exact method without type information
            // This is a limitation that could be improved with more sophisticated analysis
        }
        
        // Continue visiting
        self.visit_expr(&expr.receiver);
        for arg in &expr.args {
            self.visit_expr(arg);
        }
    }
}

impl<'a> Visit<'_> for CallTracer<'a> {
    fn visit_item_fn(&mut self, item: &ItemFn) {
        // Extract function ID
        let module_path = self.builder.extract_module_path(
            self.builder.current_file.as_ref().unwrap()
        );
        let function_id = if module_path.is_empty() {
            item.sig.ident.to_string()
        } else {
            format!("{}::{}", module_path.join("::"), item.sig.ident)
        };
        
        // Push function context
        self.builder.context_stack.push(FunctionContext {
            function_id,
            in_conditional: false,
            in_loop: false,
            block_depth: 0,
        });
        
        // Visit function body
        self.visit_block(&item.block);
        
        // Pop function context
        self.builder.context_stack.pop();
    }
    
    fn visit_expr_call(&mut self, expr: &ExprCall) {
        self.process_call(expr);
    }
    
    fn visit_expr_method_call(&mut self, expr: &ExprMethodCall) {
        self.process_method_call(expr);
    }
    
    fn visit_expr_if(&mut self, expr: &syn::ExprIf) {
        // Mark as conditional
        let was_conditional = self.builder.context_stack
            .last()
            .map(|ctx| ctx.in_conditional)
            .unwrap_or(false);
            
        if let Some(ctx) = self.builder.context_stack.last_mut() {
            ctx.in_conditional = true;
        }
        
        self.visit_expr(&expr.cond);
        self.visit_block(&expr.then_branch);
        
        if let Some((_, else_expr)) = &expr.else_branch {
            self.visit_expr(else_expr);
        }
        
        if let Some(ctx) = self.builder.context_stack.last_mut() {
            ctx.in_conditional = was_conditional;
        }
    }
    
    fn visit_expr_while(&mut self, expr: &syn::ExprWhile) {
        self.mark_in_loop(|this| {
            this.visit_expr(&expr.cond);
            this.visit_block(&expr.body);
        });
    }
    
    fn visit_expr_for_loop(&mut self, expr: &syn::ExprForLoop) {
        self.mark_in_loop(|this| {
            this.visit_pat(&expr.pat);
            this.visit_expr(&expr.expr);
            this.visit_block(&expr.body);
        });
    }
    
    fn visit_expr_loop(&mut self, expr: &syn::ExprLoop) {
        self.mark_in_loop(|this| {
            this.visit_block(&expr.body);
        });
    }
}

impl<'a> CallTracer<'a> {
    /// Helper to mark code as being in a loop
    fn mark_in_loop<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let was_in_loop = self.builder.context_stack
            .last()
            .map(|ctx| ctx.in_loop)
            .unwrap_or(false);
            
        if let Some(ctx) = self.builder.context_stack.last_mut() {
            ctx.in_loop = true;
        }
        
        f(self);
        
        if let Some(ctx) = self.builder.context_stack.last_mut() {
            ctx.in_loop = was_in_loop;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_basic_graph_building() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        
        // Create a simple test file
        let main_rs = r#"
            fn main() {
                helper();
                other::function();
            }
            
            fn helper() {
                println!("Hello");
            }
            
            mod other {
                pub fn function() {
                    super::helper();
                }
            }
        "#;
        
        fs::write(src_dir.join("main.rs"), main_rs).unwrap();
        
        let mut builder = GraphBuilder::new();
        let graph = builder.build_from_directory(&src_dir).unwrap();
        
        // Verify nodes
        assert!(graph.get_node("main").is_some());
        assert!(graph.get_node("helper").is_some());
        assert!(graph.get_node("other::function").is_some());
        
        // Verify edges exist
        let edges = graph.edges();
        assert!(edges.iter().any(|e| e.from == "main" && e.to == "helper"));
    }
    
    #[test]
    fn test_external_function_detection() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        
        let code = r#"
            use std::collections::HashMap;
            
            fn create_map() {
                let map = HashMap::new();
                println!("Created map");
            }
        "#;
        
        fs::write(src_dir.join("lib.rs"), code).unwrap();
        
        let mut builder = GraphBuilder::new();
        let graph = builder.build_from_directory(&src_dir).unwrap();
        
        // Should have the internal function
        assert!(graph.get_node("create_map").is_some());
        
        // Should detect external calls
        let external_nodes = graph.nodes_by_kind(NodeKind::External);
        assert!(!external_nodes.is_empty());
    }
}