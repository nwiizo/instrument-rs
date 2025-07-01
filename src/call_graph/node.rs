//! Function node representation in the call graph

use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use syn::{ItemFn, Path};

/// Represents a function node in the call graph
#[derive(Debug, Clone)]
pub struct FunctionNode {
    /// Unique identifier for the function
    pub id: String,
    /// Function name
    pub name: String,
    /// Full module path to the function
    pub module_path: Vec<String>,
    /// Kind of function node
    pub kind: NodeKind,
    /// File path where the function is defined
    pub file_path: Option<String>,
    /// Line number where the function is defined
    pub line_number: Option<usize>,
    /// Function signature
    pub signature: String,
    /// Whether this function is async
    pub is_async: bool,
    /// Whether this function is unsafe
    pub is_unsafe: bool,
    /// Generic parameters if any
    pub generics: Vec<String>,
    /// Attributes on the function
    pub attributes: Vec<String>,
    /// Set of functions this node calls
    pub calls: HashSet<String>,
    /// Set of functions that call this node
    pub called_by: HashSet<String>,
}

impl FunctionNode {
    /// Creates a new function node from a parsed function item
    ///
    /// # Arguments
    ///
    /// * `item` - The parsed function item
    /// * `module_path` - The module path to the function
    /// * `file_path` - Optional file path where the function is defined
    ///
    /// # Returns
    ///
    /// A new FunctionNode instance
    pub fn from_item_fn(
        item: &ItemFn,
        module_path: Vec<String>,
        file_path: Option<String>,
    ) -> Self {
        let name = item.sig.ident.to_string();
        let id = Self::generate_id(&module_path, &name);
        
        let signature = quote::quote!(#item.sig).to_string();
        let is_async = item.sig.asyncness.is_some();
        let is_unsafe = item.sig.unsafety.is_some();
        
        let generics = item.sig.generics.params
            .iter()
            .map(|param| quote::quote!(#param).to_string())
            .collect();
            
        let attributes = item.attrs
            .iter()
            .map(|attr| quote::quote!(#attr).to_string())
            .collect();
            
        // Determine the kind based on attributes and other heuristics
        let kind = Self::determine_kind(&item.attrs, &name);
        
        Self {
            id,
            name,
            module_path,
            kind,
            file_path,
            line_number: None, // To be set later with span information
            signature,
            is_async,
            is_unsafe,
            generics,
            attributes,
            calls: HashSet::new(),
            called_by: HashSet::new(),
        }
    }
    
    /// Creates a new external function node
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the external function
    ///
    /// # Returns
    ///
    /// A new FunctionNode representing an external function
    pub fn external(path: &Path) -> Self {
        let path_str = quote::quote!(#path).to_string();
        let segments: Vec<String> = path.segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect();
            
        let name = segments.last().cloned().unwrap_or_else(|| "unknown".to_string());
        let module_path = if segments.len() > 1 {
            segments[..segments.len() - 1].to_vec()
        } else {
            vec![]
        };
        
        Self {
            id: path_str.clone(),
            name,
            module_path,
            kind: NodeKind::External,
            file_path: None,
            line_number: None,
            signature: path_str,
            is_async: false,
            is_unsafe: false,
            generics: vec![],
            attributes: vec![],
            calls: HashSet::new(),
            called_by: HashSet::new(),
        }
    }
    
    /// Generates a unique ID for a function
    fn generate_id(module_path: &[String], name: &str) -> String {
        if module_path.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", module_path.join("::"), name)
        }
    }
    
    /// Determines the kind of function based on attributes and name
    fn determine_kind(attrs: &[syn::Attribute], name: &str) -> NodeKind {
        // Check for test attribute
        if attrs.iter().any(|attr| attr.path().is_ident("test")) {
            return NodeKind::Test;
        }
        
        // Check for common endpoint attributes
        for attr in attrs {
            if let Some(ident) = attr.path().get_ident() {
                let ident_str = ident.to_string();
                if matches!(ident_str.as_str(), "get" | "post" | "put" | "delete" | "patch" | "head" | "options") {
                    return NodeKind::Endpoint;
                }
            }
        }
        
        // Check for main function
        if name == "main" {
            return NodeKind::Endpoint;
        }
        
        NodeKind::Internal
    }
    
    /// Adds a function call from this node
    pub fn add_call(&mut self, target_id: String) {
        self.calls.insert(target_id);
    }
    
    /// Adds a function that calls this node
    pub fn add_caller(&mut self, caller_id: String) {
        self.called_by.insert(caller_id);
    }
    
    /// Returns the full qualified name of the function
    pub fn fully_qualified_name(&self) -> String {
        self.id.clone()
    }
    
    /// Checks if this node is reachable from any endpoint
    pub fn is_reachable(&self) -> bool {
        !self.called_by.is_empty() || matches!(self.kind, NodeKind::Endpoint | NodeKind::Test)
    }
}

impl PartialEq for FunctionNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FunctionNode {}

impl Hash for FunctionNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for FunctionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.id, self.kind)
    }
}

/// Represents the kind of function node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// An endpoint function (e.g., HTTP handler, main function)
    Endpoint,
    /// A test function
    Test,
    /// An internal function within the codebase
    Internal,
    /// An external function from a dependency
    External,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::Endpoint => write!(f, "endpoint"),
            NodeKind::Test => write!(f, "test"),
            NodeKind::Internal => write!(f, "internal"),
            NodeKind::External => write!(f, "external"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    
    #[test]
    fn test_function_node_from_item() {
        let item: ItemFn = parse_quote! {
            #[get("/users")]
            async fn get_users() -> Result<Vec<User>, Error> {
                Ok(vec![])
            }
        };
        
        let node = FunctionNode::from_item_fn(&item, vec!["api".to_string()], Some("src/api.rs".to_string()));
        
        assert_eq!(node.name, "get_users");
        assert_eq!(node.id, "api::get_users");
        assert!(node.is_async);
        assert!(!node.is_unsafe);
        assert_eq!(node.kind, NodeKind::Endpoint);
    }
    
    #[test]
    fn test_external_node() {
        let path: Path = parse_quote!(std::collections::HashMap::new);
        let node = FunctionNode::external(&path);
        
        assert_eq!(node.name, "new");
        assert_eq!(node.module_path, vec!["std", "collections", "HashMap"]);
        assert_eq!(node.kind, NodeKind::External);
    }
    
    #[test]
    fn test_node_kind_detection() {
        // Test function
        let test_fn: ItemFn = parse_quote! {
            #[test]
            fn test_something() {
                assert_eq!(1, 1);
            }
        };
        let node = FunctionNode::from_item_fn(&test_fn, vec![], None);
        assert_eq!(node.kind, NodeKind::Test);
        
        // Main function
        let main_fn: ItemFn = parse_quote! {
            fn main() {
                println!("Hello");
            }
        };
        let node = FunctionNode::from_item_fn(&main_fn, vec![], None);
        assert_eq!(node.kind, NodeKind::Endpoint);
        
        // Regular function
        let regular_fn: ItemFn = parse_quote! {
            fn helper() -> i32 {
                42
            }
        };
        let node = FunctionNode::from_item_fn(&regular_fn, vec![], None);
        assert_eq!(node.kind, NodeKind::Internal);
    }
}