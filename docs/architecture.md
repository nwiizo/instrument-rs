# instrument-rs Architecture

This document describes the high-level architecture and design decisions of instrument-rs.

## Overview

instrument-rs is designed as a modular, extensible system for code analysis and instrumentation. The architecture follows a pipeline pattern where code flows through various analysis stages, each adding information that downstream components can use.

```
Source Code → Parser → AST → Analysis Pipeline → Reports/Instrumented Code
                              ├── Call Graph Builder
                              ├── Pattern Matcher
                              ├── Framework Detector
                              ├── Instrumentation Scorer
                              └── Mutation Generator
```

## Core Components

### 1. AST Module (`src/ast/`)

The AST module provides the foundation for all code analysis. It uses the `syn` crate to parse Rust source code into an Abstract Syntax Tree.

**Key Components:**
- `SourceFile`: Represents a parsed file with metadata
- `analyzer`: Core analysis functionality
- `visitor`: AST traversal utilities
- `helpers`: Common AST manipulation functions

**Design Decisions:**
- Uses `syn` with full features for complete Rust syntax support
- Maintains source location information for accurate reporting
- Preserves original source for display in reports

### 2. Call Graph Module (`src/call_graph/`)

Builds a directed graph of function calls to understand code flow and dependencies.

**Key Components:**
- `GraphBuilder`: Constructs the call graph from AST
- `Node`: Represents functions/methods
- `Edge`: Represents function calls with metadata
- `Resolver`: Handles cross-module and cross-crate resolution

**Features:**
- Tracks call sites with source locations
- Handles trait implementations and generics
- Supports async function analysis
- Provides cycle detection

### 3. Pattern Matching (`src/patterns/`)

A flexible pattern matching system for identifying code constructs of interest.

**Key Components:**
- `PatternSet`: Collection of patterns to match
- `Matcher`: Pattern matching engine
- `Result`: Match results with confidence scores

**Pattern Types:**
- Function name patterns (regex-based)
- AST structure patterns
- Call chain patterns
- Attribute patterns

### 4. Framework Detection (`src/framework/`)

Auto-detects web frameworks and their specific patterns.

**Supported Frameworks:**
- Axum: Handler functions, middleware, routers
- Actix-web: Handlers, services, app configuration
- Rocket: Route handlers, guards, fairings
- Tonic: gRPC services and methods

**Design:**
- Pluggable architecture for adding new frameworks
- Framework-specific endpoint detection
- Middleware and interceptor recognition

### 5. Instrumentation Module (`src/instrumentation/`)

Handles code transformation for adding instrumentation.

**Components:**
- `Coverage`: Adds coverage tracking
- `Mutation`: Applies mutations for testing
- `Transform`: Generic code transformation infrastructure

**Instrumentation Types:**
- Line coverage tracking
- Branch coverage tracking
- Function entry/exit tracing
- Performance timing

### 6. Mutation Testing (`src/mutation/`)

Implements mutation testing to evaluate test suite quality.

**Mutation Operators:**
- Arithmetic: `+` → `-`, `*` → `/`
- Comparison: `<` → `>`, `==` → `!=`
- Logical: `&&` → `||`, `!` negation
- Constant: Modify literal values
- Statement deletion
- Return value modification

**Design:**
- Deterministic mutation selection with seeds
- Parallel mutation execution
- Timeout handling for infinite loops
- Minimal performance overhead

### 7. Scoring System (`src/scoring/`)

Evaluates code quality and instrumentation effectiveness.

**Scoring Dimensions:**
- Coverage completeness
- Critical path coverage
- Error handling instrumentation
- Performance hotspot coverage
- External service call tracking

### 8. Output Formats (`src/output/`)

Provides multiple output formats for different use cases.

**Formats:**
- **Human**: Readable terminal output with colors
- **JSON**: Machine-readable for CI/CD integration
- **Mermaid**: Interactive diagrams for documentation
- **DOT**: Graphviz format for detailed graphs
- **HTML**: Rich reports with source code

### 9. Reporting (`src/reporting/`)

Generates comprehensive reports combining all analysis results.

**Report Types:**
- Coverage reports with line-by-line details
- Mutation testing reports with killed/survived mutations
- Instrumentation quality reports
- Call graph visualizations
- Pattern match summaries

## Data Flow

1. **Input Phase**: Source files are discovered and parsed
2. **Analysis Phase**: Multiple analyzers run in parallel:
   - Call graph construction
   - Pattern matching
   - Framework detection
   - Existing instrumentation detection
3. **Scoring Phase**: Results are scored and prioritized
4. **Transformation Phase**: Code modifications are applied
5. **Output Phase**: Reports are generated in requested formats

## Configuration System

The configuration system (`src/config.rs`) provides:
- TOML-based configuration files
- Hierarchical configuration (project → user → system)
- Environment variable overrides
- Sensible defaults for all options

## Error Handling

Uses a consistent error handling approach:
- `thiserror` for error type definitions
- `Result<T, Error>` for fallible operations
- Detailed error messages with context
- Recovery strategies for non-fatal errors

## Performance Considerations

1. **Parallel Processing**: Uses `rayon` for parallel file analysis
2. **Incremental Analysis**: Supports analyzing only changed files
3. **Caching**: File hashes prevent re-analyzing unchanged code
4. **Memory Efficiency**: Streaming processing for large codebases

## Extensibility Points

The architecture provides several extension points:

1. **Custom Patterns**: Users can define their own patterns
2. **Framework Adapters**: New frameworks can be added
3. **Mutation Operators**: Custom mutations can be implemented
4. **Output Formats**: New report formats can be added
5. **Scoring Algorithms**: Pluggable scoring systems

## Security Considerations

1. **Sandboxed Execution**: Mutation tests run in isolated environments
2. **Path Validation**: All file paths are validated and sanitized
3. **Resource Limits**: Timeouts and memory limits prevent DoS
4. **No Code Execution**: Analysis never executes user code directly

## Future Architecture Considerations

### Planned Enhancements

1. **Language Server Protocol (LSP)**: Real-time analysis in editors
2. **Distributed Analysis**: Support for analyzing massive codebases
3. **Cloud Integration**: Direct integration with observability platforms
4. **AI-Powered Suggestions**: ML-based instrumentation recommendations

### Scalability

The current architecture scales to:
- Codebases with 100k+ files
- Functions with complex call graphs (1000+ nodes)
- Parallel analysis on 32+ cores

### Integration Points

Designed to integrate with:
- CI/CD systems (GitHub Actions, GitLab CI, Jenkins)
- Observability platforms (DataDog, New Relic, Jaeger)
- Testing frameworks (cargo test, nextest)
- Code quality tools (clippy, rustfmt)

## Design Principles

1. **Modularity**: Each component has a single responsibility
2. **Composability**: Components can be combined in different ways
3. **Performance**: Fast enough for CI/CD integration
4. **Accuracy**: Prefer false negatives over false positives
5. **Usability**: Clear output with actionable recommendations
6. **Extensibility**: Easy to add new features without breaking existing ones