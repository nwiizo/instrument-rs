//! Call edge representation in the call graph

use std::fmt;
use std::hash::{Hash, Hasher};

/// Represents an edge (function call) in the call graph
#[derive(Debug, Clone)]
pub struct CallEdge {
    /// ID of the calling function
    pub from: String,
    /// ID of the called function
    pub to: String,
    /// Kind of call
    pub kind: CallKind,
    /// File path where the call occurs
    pub file_path: Option<String>,
    /// Line number where the call occurs
    pub line_number: Option<usize>,
    /// Column number where the call occurs
    pub column: Option<usize>,
    /// Whether this call is conditional (e.g., inside an if statement)
    pub is_conditional: bool,
    /// Whether this call is in a loop
    pub is_in_loop: bool,
    /// Call context (e.g., method call, function call, closure)
    pub context: CallContext,
}

impl CallEdge {
    /// Creates a new call edge
    ///
    /// # Arguments
    ///
    /// * `from` - ID of the calling function
    /// * `to` - ID of the called function
    /// * `kind` - The kind of call
    ///
    /// # Returns
    ///
    /// A new CallEdge instance
    pub fn new(from: String, to: String, kind: CallKind) -> Self {
        Self {
            from,
            to,
            kind,
            file_path: None,
            line_number: None,
            column: None,
            is_conditional: false,
            is_in_loop: false,
            context: CallContext::Direct,
        }
    }
    
    /// Sets the location information for this edge
    pub fn with_location(mut self, file_path: String, line: usize, column: usize) -> Self {
        self.file_path = Some(file_path);
        self.line_number = Some(line);
        self.column = Some(column);
        self
    }
    
    /// Sets whether this call is conditional
    pub fn with_conditional(mut self, is_conditional: bool) -> Self {
        self.is_conditional = is_conditional;
        self
    }
    
    /// Sets whether this call is in a loop
    pub fn with_in_loop(mut self, is_in_loop: bool) -> Self {
        self.is_in_loop = is_in_loop;
        self
    }
    
    /// Sets the call context
    pub fn with_context(mut self, context: CallContext) -> Self {
        self.context = context;
        self
    }
    
    /// Returns a unique identifier for this edge
    pub fn id(&self) -> String {
        format!("{} -> {}", self.from, self.to)
    }
    
    /// Returns the weight of this edge for graph analysis
    pub fn weight(&self) -> f64 {
        let base_weight = match self.kind {
            CallKind::Direct => 1.0,
            CallKind::Indirect => 0.8,
            CallKind::Dynamic => 0.5,
            CallKind::Recursive => 1.0,
            CallKind::Trait => 0.7,
            CallKind::Closure => 0.9,
        };
        
        let conditional_factor = if self.is_conditional { 0.8 } else { 1.0 };
        let loop_factor = if self.is_in_loop { 1.5 } else { 1.0 };
        
        base_weight * conditional_factor * loop_factor
    }
}

impl PartialEq for CallEdge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl Eq for CallEdge {}

impl Hash for CallEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
    }
}

impl fmt::Display for CallEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -[{}]-> {}", self.from, self.kind, self.to)
    }
}

/// Represents the kind of function call
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallKind {
    /// Direct function call (e.g., `foo()`)
    Direct,
    /// Indirect function call through a function pointer
    Indirect,
    /// Dynamic dispatch (e.g., trait object method call)
    Dynamic,
    /// Recursive call
    Recursive,
    /// Trait method call
    Trait,
    /// Closure invocation
    Closure,
}

impl fmt::Display for CallKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallKind::Direct => write!(f, "direct"),
            CallKind::Indirect => write!(f, "indirect"),
            CallKind::Dynamic => write!(f, "dynamic"),
            CallKind::Recursive => write!(f, "recursive"),
            CallKind::Trait => write!(f, "trait"),
            CallKind::Closure => write!(f, "closure"),
        }
    }
}

/// Represents the context in which a call occurs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallContext {
    /// Direct function call
    Direct,
    /// Method call on a type
    Method,
    /// Associated function call
    Associated,
    /// Closure or lambda
    Closure,
    /// Macro expansion
    Macro,
    /// Async await point
    Async,
}

impl fmt::Display for CallContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallContext::Direct => write!(f, "direct"),
            CallContext::Method => write!(f, "method"),
            CallContext::Associated => write!(f, "associated"),
            CallContext::Closure => write!(f, "closure"),
            CallContext::Macro => write!(f, "macro"),
            CallContext::Async => write!(f, "async"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_call_edge_creation() {
        let edge = CallEdge::new(
            "module::caller".to_string(),
            "module::callee".to_string(),
            CallKind::Direct,
        );
        
        assert_eq!(edge.from, "module::caller");
        assert_eq!(edge.to, "module::callee");
        assert_eq!(edge.kind, CallKind::Direct);
        assert!(!edge.is_conditional);
        assert!(!edge.is_in_loop);
    }
    
    #[test]
    fn test_edge_with_location() {
        let edge = CallEdge::new(
            "caller".to_string(),
            "callee".to_string(),
            CallKind::Direct,
        )
        .with_location("src/main.rs".to_string(), 42, 10);
        
        assert_eq!(edge.file_path, Some("src/main.rs".to_string()));
        assert_eq!(edge.line_number, Some(42));
        assert_eq!(edge.column, Some(10));
    }
    
    #[test]
    fn test_edge_weight_calculation() {
        let base_edge = CallEdge::new(
            "a".to_string(),
            "b".to_string(),
            CallKind::Direct,
        );
        assert_eq!(base_edge.weight(), 1.0);
        
        let conditional_edge = base_edge.clone().with_conditional(true);
        assert_eq!(conditional_edge.weight(), 0.8);
        
        let loop_edge = base_edge.clone().with_in_loop(true);
        assert_eq!(loop_edge.weight(), 1.5);
        
        let complex_edge = base_edge.clone()
            .with_conditional(true)
            .with_in_loop(true);
        assert_eq!(complex_edge.weight(), 1.2); // 1.0 * 0.8 * 1.5
    }
    
    #[test]
    fn test_edge_equality() {
        let edge1 = CallEdge::new("a".to_string(), "b".to_string(), CallKind::Direct);
        let edge2 = CallEdge::new("a".to_string(), "b".to_string(), CallKind::Indirect);
        let edge3 = CallEdge::new("a".to_string(), "c".to_string(), CallKind::Direct);
        
        assert_eq!(edge1, edge2); // Same from/to, different kind
        assert_ne!(edge1, edge3); // Different to
    }
}