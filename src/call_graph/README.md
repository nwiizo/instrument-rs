# Call Graph Module

This module implements call graph construction for Rust code analysis, as specified in section 3 of the instrument-rs specification.

## Overview

The call graph module provides functionality to:
- Build comprehensive call graphs from Rust source code
- Trace function calls from endpoints (main, HTTP handlers, tests)
- Identify external function boundaries
- Detect cycles and analyze reachability
- Resolve symbols across modules

## Components

### Core Types

- **`CallGraph`**: The main graph data structure containing nodes and edges
- **`FunctionNode`**: Represents a function in the graph with metadata
- **`CallEdge`**: Represents a function call relationship
- **`GraphBuilder`**: Constructs call graphs from source directories
- **`SymbolResolver`**: Resolves function references and imports

### Node Types

Functions are classified into four categories:
- **Endpoint**: Entry points like `main()` or HTTP handlers
- **Test**: Test functions marked with `#[test]`
- **Internal**: Regular functions within the codebase
- **External**: Functions from external crates or std library

### Edge Types

Function calls are categorized as:
- **Direct**: Regular function calls
- **Indirect**: Function pointer calls
- **Dynamic**: Trait object method calls
- **Recursive**: Self-referential calls
- **Trait**: Trait method calls
- **Closure**: Closure invocations

## Usage

```rust
use instrument_rs::call_graph::{GraphBuilder, NodeKind};

// Build a call graph from a directory
let mut builder = GraphBuilder::new();
let graph = builder.build_from_directory("src")?;

// Analyze the graph
let stats = graph.stats();
println!("Total functions: {}", stats.total_nodes);
println!("Unreachable functions: {}", stats.unreachable_internal);

// Find all endpoints
for node in graph.nodes_by_kind(NodeKind::Endpoint) {
    println!("Endpoint: {}", node.name);
}

// Check reachability from main
let reachable = graph.find_reachable("main");
println!("Functions reachable from main: {}", reachable.len());

// Detect cycles
let cycles = graph.find_cycles();
if !cycles.is_empty() {
    println!("Warning: {} cycles detected", cycles.len());
}
```

## Implementation Details

### Two-Pass Analysis

1. **Definition Collection**: First pass collects all function definitions
2. **Call Tracing**: Second pass traces function calls and builds edges

### Symbol Resolution

The resolver handles:
- Fully qualified paths
- Use statements and imports
- Relative paths within modules
- External crate detection

### Graph Analysis

The graph supports:
- Breadth-first search for reachability
- Cycle detection using DFS
- Shortest path finding
- In/out degree analysis

## Limitations

- Method resolution without full type information is approximate
- Macro-generated code may not be fully captured
- Dynamic dispatch through trait objects is estimated
- Async function calls are treated as regular calls

## Future Enhancements

- Integration with type information for better method resolution
- Support for macro expansion
- Call frequency analysis from test execution
- Integration with mutation testing for targeted instrumentation