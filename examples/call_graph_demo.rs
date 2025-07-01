//! Example demonstrating call graph construction

use instrument_rs::call_graph::{GraphBuilder, CallGraph};
use std::path::Path;

fn main() {
    // Create a graph builder
    let mut builder = GraphBuilder::new();
    
    // Build graph from the current project
    let project_path = Path::new("src");
    
    match builder.build_from_directory(project_path) {
        Ok(graph) => {
            println!("Successfully built call graph!");
            println!("{}", graph);
            
            // Show some statistics
            let stats = graph.stats();
            println!("\nGraph Statistics:");
            println!("  Total nodes: {}", stats.total_nodes);
            println!("  Total edges: {}", stats.total_edges);
            println!("  Endpoints: {}", stats.endpoint_count);
            println!("  Tests: {}", stats.test_count);
            println!("  Internal functions: {}", stats.internal_count);
            println!("  External functions: {}", stats.external_count);
            println!("  Unreachable internal: {}", stats.unreachable_internal);
            println!("  Cycles detected: {}", stats.cycle_count);
            
            // Find all endpoints
            println!("\nEndpoints found:");
            for node in graph.nodes() {
                if matches!(node.kind, instrument_rs::call_graph::NodeKind::Endpoint) {
                    println!("  - {} ({})", node.name, node.fully_qualified_name());
                }
            }
            
            // Find all test functions
            println!("\nTest functions found:");
            for node in graph.nodes() {
                if matches!(node.kind, instrument_rs::call_graph::NodeKind::Test) {
                    println!("  - {} ({})", node.name, node.fully_qualified_name());
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to build call graph: {}", e);
        }
    }
}