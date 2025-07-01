//! Call graph data structure and operations

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use super::{CallEdge, FunctionNode, NodeKind};

/// Represents a complete call graph
#[derive(Debug, Clone)]
pub struct CallGraph {
    /// All nodes in the graph, indexed by their ID
    nodes: HashMap<String, FunctionNode>,
    /// All edges in the graph
    edges: Vec<CallEdge>,
    /// Adjacency list for efficient traversal (from -> to)
    adjacency: HashMap<String, HashSet<String>>,
    /// Reverse adjacency list (to -> from)
    reverse_adjacency: HashMap<String, HashSet<String>>,
}

impl CallGraph {
    /// Creates a new empty call graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }
    
    /// Adds a node to the graph
    ///
    /// # Arguments
    ///
    /// * `node` - The function node to add
    ///
    /// # Returns
    ///
    /// `true` if the node was added, `false` if it already existed
    pub fn add_node(&mut self, node: FunctionNode) -> bool {
        let id = node.id.clone();
        if self.nodes.contains_key(&id) {
            false
        } else {
            self.nodes.insert(id.clone(), node);
            self.adjacency.entry(id.clone()).or_default();
            self.reverse_adjacency.entry(id).or_default();
            true
        }
    }
    
    /// Adds an edge to the graph
    ///
    /// # Arguments
    ///
    /// * `edge` - The call edge to add
    ///
    /// # Panics
    ///
    /// Panics if either the source or target node doesn't exist in the graph
    pub fn add_edge(&mut self, edge: CallEdge) {
        assert!(
            self.nodes.contains_key(&edge.from),
            "Source node {} not found in graph",
            edge.from
        );
        assert!(
            self.nodes.contains_key(&edge.to),
            "Target node {} not found in graph",
            edge.to
        );
        
        // Update adjacency lists
        self.adjacency
            .entry(edge.from.clone())
            .or_default()
            .insert(edge.to.clone());
        self.reverse_adjacency
            .entry(edge.to.clone())
            .or_default()
            .insert(edge.from.clone());
            
        // Update node call relationships
        if let Some(from_node) = self.nodes.get_mut(&edge.from) {
            from_node.add_call(edge.to.clone());
        }
        if let Some(to_node) = self.nodes.get_mut(&edge.to) {
            to_node.add_caller(edge.from.clone());
        }
        
        self.edges.push(edge);
    }
    
    /// Gets a node by its ID
    pub fn get_node(&self, id: &str) -> Option<&FunctionNode> {
        self.nodes.get(id)
    }
    
    /// Gets a mutable reference to a node by its ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut FunctionNode> {
        self.nodes.get_mut(id)
    }
    
    /// Gets all nodes in the graph
    pub fn nodes(&self) -> impl Iterator<Item = &FunctionNode> {
        self.nodes.values()
    }
    
    /// Gets all edges in the graph
    pub fn edges(&self) -> &[CallEdge] {
        &self.edges
    }
    
    /// Gets all nodes of a specific kind
    pub fn nodes_by_kind(&self, kind: NodeKind) -> Vec<&FunctionNode> {
        self.nodes
            .values()
            .filter(|node| node.kind == kind)
            .collect()
    }
    
    /// Gets all direct callees of a function
    pub fn get_callees(&self, id: &str) -> Option<&HashSet<String>> {
        self.adjacency.get(id)
    }
    
    /// Gets all direct callers of a function
    pub fn get_callers(&self, id: &str) -> Option<&HashSet<String>> {
        self.reverse_adjacency.get(id)
    }
    
    /// Finds all reachable nodes from a given starting node using BFS
    ///
    /// # Arguments
    ///
    /// * `start_id` - The ID of the starting node
    ///
    /// # Returns
    ///
    /// A set of all reachable node IDs
    pub fn find_reachable(&self, start_id: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        if self.nodes.contains_key(start_id) {
            queue.push_back(start_id.to_string());
            visited.insert(start_id.to_string());
        }
        
        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = self.adjacency.get(&current) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        visited
    }
    
    /// Finds all nodes that can reach a given target node
    ///
    /// # Arguments
    ///
    /// * `target_id` - The ID of the target node
    ///
    /// # Returns
    ///
    /// A set of all node IDs that can reach the target
    pub fn find_reaching(&self, target_id: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        if self.nodes.contains_key(target_id) {
            queue.push_back(target_id.to_string());
            visited.insert(target_id.to_string());
        }
        
        while let Some(current) = queue.pop_front() {
            if let Some(callers) = self.reverse_adjacency.get(&current) {
                for caller in callers {
                    if visited.insert(caller.clone()) {
                        queue.push_back(caller.clone());
                    }
                }
            }
        }
        
        visited
    }
    
    /// Finds the shortest path between two nodes using BFS
    ///
    /// # Arguments
    ///
    /// * `from` - The starting node ID
    /// * `to` - The target node ID
    ///
    /// # Returns
    ///
    /// The shortest path as a vector of node IDs, or None if no path exists
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) {
            return None;
        }
        
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();
        
        queue.push_back(from.to_string());
        visited.insert(from.to_string());
        
        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to.to_string();
                
                while node != from {
                    path.push(node.clone());
                    node = parent.get(&node).unwrap().clone();
                }
                path.push(from.to_string());
                path.reverse();
                
                return Some(path);
            }
            
            if let Some(neighbors) = self.adjacency.get(&current) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        None
    }
    
    /// Detects cycles in the graph using DFS
    ///
    /// # Returns
    ///
    /// A vector of cycles, where each cycle is represented as a vector of node IDs
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_find_cycles(
                    node_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }
        
        cycles
    }
    
    /// Helper function for cycle detection using DFS
    fn dfs_find_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(neighbors) = self.adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_find_cycles(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                        cycles.push(path[start_idx..].to_vec());
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node);
    }
    
    /// Computes statistics about the graph
    pub fn stats(&self) -> GraphStats {
        let endpoint_count = self.nodes_by_kind(NodeKind::Endpoint).len();
        let test_count = self.nodes_by_kind(NodeKind::Test).len();
        let internal_count = self.nodes_by_kind(NodeKind::Internal).len();
        let external_count = self.nodes_by_kind(NodeKind::External).len();
        
        let reachable_from_endpoints: HashSet<_> = self
            .nodes_by_kind(NodeKind::Endpoint)
            .iter()
            .flat_map(|node| self.find_reachable(&node.id))
            .collect();
            
        let unreachable_internal = self
            .nodes_by_kind(NodeKind::Internal)
            .iter()
            .filter(|node| !reachable_from_endpoints.contains(&node.id))
            .count();
            
        let cycles = self.find_cycles();
        
        GraphStats {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            endpoint_count,
            test_count,
            internal_count,
            external_count,
            unreachable_internal,
            cycle_count: cycles.len(),
            max_in_degree: self.reverse_adjacency.values().map(|s| s.len()).max().unwrap_or(0),
            max_out_degree: self.adjacency.values().map(|s| s.len()).max().unwrap_or(0),
        }
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CallGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CallGraph {{")?;
        writeln!(f, "  Nodes: {}", self.nodes.len())?;
        writeln!(f, "  Edges: {}", self.edges.len())?;
        
        let stats = self.stats();
        writeln!(f, "  Endpoints: {}", stats.endpoint_count)?;
        writeln!(f, "  Tests: {}", stats.test_count)?;
        writeln!(f, "  Internal: {}", stats.internal_count)?;
        writeln!(f, "  External: {}", stats.external_count)?;
        writeln!(f, "  Unreachable: {}", stats.unreachable_internal)?;
        writeln!(f, "  Cycles: {}", stats.cycle_count)?;
        
        write!(f, "}}")
    }
}

/// Statistics about a call graph
#[derive(Debug, Clone)]
pub struct GraphStats {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Number of endpoint nodes
    pub endpoint_count: usize,
    /// Number of test nodes
    pub test_count: usize,
    /// Number of internal nodes
    pub internal_count: usize,
    /// Number of external nodes
    pub external_count: usize,
    /// Number of unreachable internal nodes
    pub unreachable_internal: usize,
    /// Number of cycles detected
    pub cycle_count: usize,
    /// Maximum in-degree (number of callers)
    pub max_in_degree: usize,
    /// Maximum out-degree (number of callees)
    pub max_out_degree: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_graph::CallKind;
    
    fn create_test_graph() -> CallGraph {
        let mut graph = CallGraph::new();
        
        // Add nodes
        let mut main = FunctionNode {
            id: "main".to_string(),
            name: "main".to_string(),
            module_path: vec![],
            kind: NodeKind::Endpoint,
            file_path: None,
            line_number: None,
            signature: "fn main()".to_string(),
            is_async: false,
            is_unsafe: false,
            generics: vec![],
            attributes: vec![],
            calls: HashSet::new(),
            called_by: HashSet::new(),
        };
        
        let mut foo = main.clone();
        foo.id = "foo".to_string();
        foo.name = "foo".to_string();
        foo.kind = NodeKind::Internal;
        
        let mut bar = main.clone();
        bar.id = "bar".to_string();
        bar.name = "bar".to_string();
        bar.kind = NodeKind::Internal;
        
        let mut baz = main.clone();
        baz.id = "baz".to_string();
        baz.name = "baz".to_string();
        baz.kind = NodeKind::Internal;
        
        graph.add_node(main);
        graph.add_node(foo);
        graph.add_node(bar);
        graph.add_node(baz);
        
        // Add edges
        graph.add_edge(CallEdge::new("main".to_string(), "foo".to_string(), CallKind::Direct));
        graph.add_edge(CallEdge::new("foo".to_string(), "bar".to_string(), CallKind::Direct));
        graph.add_edge(CallEdge::new("bar".to_string(), "baz".to_string(), CallKind::Direct));
        
        graph
    }
    
    #[test]
    fn test_graph_construction() {
        let graph = create_test_graph();
        
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);
        
        assert!(graph.get_node("main").is_some());
        assert!(graph.get_node("foo").is_some());
        assert!(graph.get_node("nonexistent").is_none());
    }
    
    #[test]
    fn test_reachability() {
        let graph = create_test_graph();
        
        let reachable = graph.find_reachable("main");
        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains("main"));
        assert!(reachable.contains("foo"));
        assert!(reachable.contains("bar"));
        assert!(reachable.contains("baz"));
        
        let reachable_from_bar = graph.find_reachable("bar");
        assert_eq!(reachable_from_bar.len(), 2);
        assert!(reachable_from_bar.contains("bar"));
        assert!(reachable_from_bar.contains("baz"));
    }
    
    #[test]
    fn test_reaching() {
        let graph = create_test_graph();
        
        let reaching_baz = graph.find_reaching("baz");
        assert_eq!(reaching_baz.len(), 4);
        
        let reaching_foo = graph.find_reaching("foo");
        assert_eq!(reaching_foo.len(), 2);
        assert!(reaching_foo.contains("main"));
        assert!(reaching_foo.contains("foo"));
    }
    
    #[test]
    fn test_path_finding() {
        let graph = create_test_graph();
        
        let path = graph.find_path("main", "baz").unwrap();
        assert_eq!(path, vec!["main", "foo", "bar", "baz"]);
        
        let no_path = graph.find_path("baz", "main");
        assert!(no_path.is_none());
    }
    
    #[test]
    fn test_cycle_detection() {
        let mut graph = create_test_graph();
        
        // Add a cycle
        graph.add_edge(CallEdge::new("baz".to_string(), "foo".to_string(), CallKind::Direct));
        
        let cycles = graph.find_cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
        assert!(cycles[0].contains(&"foo".to_string()));
        assert!(cycles[0].contains(&"bar".to_string()));
        assert!(cycles[0].contains(&"baz".to_string()));
    }
    
    #[test]
    fn test_graph_stats() {
        let graph = create_test_graph();
        let stats = graph.stats();
        
        assert_eq!(stats.total_nodes, 4);
        assert_eq!(stats.total_edges, 3);
        assert_eq!(stats.endpoint_count, 1);
        assert_eq!(stats.internal_count, 3);
        assert_eq!(stats.unreachable_internal, 0);
        assert_eq!(stats.cycle_count, 0);
    }
}