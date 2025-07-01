# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`instrument-rs` is a Rust CLI tool for analyzing code and detecting optimal instrumentation points for observability (tracing, logging, metrics). It traces execution flows from HTTP/gRPC endpoints to identify critical paths that need monitoring, helping teams implement comprehensive observability strategies.

## CLI Usage

### Basic Commands

```bash
# Analyze current directory
instrument-rs .

# Trace from endpoints (most common usage)
instrument-rs . --trace-from-endpoints

# Generate visual call graph
instrument-rs . --trace-from-endpoints --format mermaid > flow.md

# Analyze with specific framework
instrument-rs . --framework axum --trace-from-endpoints

# Filter to specific paths
instrument-rs . --trace-from-endpoints --filter-path "payment|order"

# Generate JSON report for CI/CD
instrument-rs . --format json > report.json
```

### Advanced Options

```bash
# Adjust detection sensitivity
instrument-rs . --threshold 0.9

# Include test endpoints in analysis  
instrument-rs . --trace-from-endpoints --include-tests

# Limit call graph depth
instrument-rs . --trace-from-endpoints --max-depth 5

# Use custom patterns
instrument-rs . --patterns custom-patterns.yml
```

## Development Commands

### Building and Testing

```bash
# Build the project
cargo build --release

# Run the CLI
cargo run -- . --trace-from-endpoints

# Run all tests
cargo test

# Run tests with output visible
cargo test -- --nocapture

# Run specific test file
cargo test --test ast_test
cargo test --test framework_detection
cargo test --test pattern_matching

# Run with all warnings
cargo clippy -- -D warnings

# Format code
cargo fmt

# Generate and open documentation
cargo doc --no-deps --open
```

### Development Workflow

```bash
# Before committing (as per global CLAUDE.md)
cargo fmt && cargo clippy && cargo test

# Check for unused dependencies
cargo machete

# Update dependencies
cargo update
```

## Architecture Overview

### Core Components

1. **CLI Entry Point (`src/main.rs`)**
   - Command-line argument parsing
   - Framework auto-detection
   - Orchestrates analysis pipeline

2. **Analyzer Module (`src/analyzer/`)**
   - `ast.rs`: AST parsing and traversal using `syn`
   - `call_graph.rs`: Builds function call graphs from AST
   - `patterns.rs`: Pattern matching for business logic detection
   - Identifies critical paths and external service boundaries

3. **Framework Detection (`src/frameworks/`)**
   - `axum.rs`: Detects Axum handlers and middleware
   - `actix.rs`: Actix-web endpoint detection
   - `rocket.rs`: Rocket route detection
   - `tonic.rs`: gRPC service detection
   - Extensible trait-based design for new frameworks

4. **Pattern System (`src/patterns/`)**
   - Business logic patterns (payment, order, user, etc.)
   - Infrastructure patterns (database, cache, queue, etc.)
   - External service call detection
   - Customizable via YAML configuration

5. **Detector (`src/detector.rs`)**
   - Instrumentation point identification
   - Existing instrumentation quality analysis
   - Critical path prioritization
   - Cost-benefit analysis for instrumentation

6. **Output Formatting (`src/output.rs`)**
   - Human-readable tree output
   - JSON for programmatic consumption
   - Mermaid/DOT for visualization
   - CI/CD integration formats

### Key Design Patterns

1. **Visitor Pattern**: Used extensively in AST analysis for traversing syntax trees
2. **Builder Pattern**: Expected for complex configuration objects
3. **Result-based Error Handling**: Using `thiserror` for custom errors
4. **Trait-based Extensibility**: Reporter traits, detector traits for framework detection

### Module Dependencies

```
main.rs (CLI entry point)
├── analyzer/
│   ├── ast.rs (syntax tree analysis)
│   ├── call_graph.rs (function relationships)
│   └── patterns.rs (pattern matching)
├── frameworks/
│   ├── mod.rs (framework trait)
│   ├── axum.rs
│   ├── actix.rs
│   ├── rocket.rs
│   └── tonic.rs
├── patterns/
│   └── default.yml (pattern definitions)
├── detector.rs (instrumentation detection)
├── output.rs (formatting)
└── error.rs (error types)
```

### Pattern Categories

The tool recognizes these pattern categories for identifying critical code:

**Business Logic Patterns**
- Payment processing: `process_payment`, `charge_card`, `refund`
- Order management: `create_order`, `fulfill_order`, `ship_order`
- User operations: `authenticate_user`, `register_user`
- Inventory: `check_stock`, `reserve_inventory`

**Infrastructure Patterns**
- Database: `execute_query`, `fetch_*`, `insert_*`, `update_*`
- Cache: `get_from_cache`, `set_cache`, `invalidate_cache`
- Queue: `publish_message`, `enqueue_*`, `dequeue_*`
- External APIs: `call_api`, `http_client`, `send_request`

## Current Development Focus

Based on TODO.md, the project is actively implementing:

### High Priority
1. **Existing Instrumentation Detection**
   - Detect `#[instrument]` macros and manual span creation
   - Analyze logging patterns and quality
   - Identify gaps in current observability

2. **Differential Analysis System**
   - Generate migration steps from current to ideal state
   - Impact analysis for proposed changes
   - Gradual rollout plans

3. **Coverage Analysis**
   - Calculate instrumentation coverage by module
   - Identify critical paths lacking observability
   - Generate gap analysis reports

### Medium Priority
1. **Performance Impact Estimation**
   - Predict instrumentation overhead
   - Hot path analysis
   - Alternative implementation suggestions

2. **Cost Optimization**
   - Telemetry volume prediction
   - Cost calculation for major providers (DataDog, CloudWatch)
   - Sampling recommendations

3. **Team Standardization**
   - Enforce naming conventions
   - Required field validation
   - Security compliance checks

## Testing Strategy

- **Unit Tests**: Use `#[cfg(test)]` modules within source files
- **Integration Tests**: Test full CLI workflows in `tests/`
- **Snapshot Testing**: Uses `insta` for comparing complex outputs
- **Framework Tests**: Specific tests for each framework detector

## Important Notes

1. This is a CLI tool, not a library - designed for direct command-line usage
2. Primary focus is observability (tracing, logging, metrics), not test coverage
3. Heavy use of `syn` for AST parsing - be familiar with syn's syntax tree types
4. Endpoint-based analysis is the key feature - traces from entry points
5. Framework auto-detection supports multiple web frameworks
6. Japanese documentation in TODO.md reflects production use cases

## AI Assistant Integration

When working with this tool:
```
Use `instrument-rs . --trace-from-endpoints` to analyze Rust code and identify
critical paths needing instrumentation. The tool traces from HTTP/gRPC endpoints
through the entire call graph to find external service calls, database operations,
and business logic requiring observability.
```

## Future Enhancements

Consider implementing:
- GitHub Actions for automated analysis in PRs
- Integration with OpenTelemetry configuration
- Support for more frameworks (warp, poem, salvo)
- IDE plugins for real-time instrumentation suggestions
- Benchmarks for performance impact analysis
- Integration with APM providers (DataDog, New Relic)