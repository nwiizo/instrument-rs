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
# IMPORTANT: Always run these checks before pushing to avoid CI failures
cargo fmt && cargo clippy -- -D warnings && cargo test

# Check for unused dependencies
cargo machete

# Update dependencies
cargo update
```

### Pre-Push Checklist (CRITICAL)

**⚠️ 必ずコミット前にローカルで以下を実行すること:**

```bash
# 全チェックを一括実行（コミット前に必須）
cargo fmt && cargo clippy -- -D warnings && cargo test && cargo doc --no-deps
```

| チェック | コマンド | 備考 |
|---------|---------|------|
| フォーマット | `cargo fmt` | 自動修正される |
| Lint | `cargo clippy -- -D warnings` | 警告をエラーとして扱う |
| テスト | `cargo test` | 全テスト通過必須 |
| ドキュメント | `cargo doc --no-deps` | docコメントのリンク切れ検出 |

### よくあるCI失敗パターン

1. **rustdoc broken links**: docコメント内の `#[instrument]` などが壊れたリンクとして検出される
   - 解決: バッククォートで囲む（例: `` `#[instrument]` ``）

2. **clippy warnings**: CIでは `-D warnings` で警告がエラーになる
   - 解決: ローカルで `cargo clippy -- -D warnings` を実行

3. **test failures**: ローカルで通ってもCIで落ちることがある
   - 解決: `cargo test` を必ず実行

CIはこれらすべてをチェックするため、ローカルで確認せずにpushするとCIが失敗する。

## Architecture Overview

### Core Components

1. **CLI Entry Point (`src/main.rs`)**
   - Command-line argument parsing with clap
   - Subcommands: `init`, `check`
   - Output formats: human, json, mermaid

2. **Analyzer (`src/lib.rs`)**
   - Main `Analyzer` struct orchestrates analysis
   - Returns `AnalysisResult` with endpoints, instrumentation points, stats

3. **Dependency Analysis (`src/dependencies.rs`)**
   - Parses `Cargo.toml` using `cargo_metadata`
   - Detects: databases (sqlx, diesel), HTTP clients (reqwest), caches (redis), frameworks (axum)
   - `DetectionContext` for context-aware pattern matching

4. **AST Analysis (`src/ast/`)**
   - `analyzer.rs`: Core AST analysis using `syn`
   - `visitor.rs`: AST traversal with accurate span locations
   - `helpers.rs`: Utility functions for AST manipulation

5. **Call Graph (`src/call_graph/`)**
   - `builder.rs`: Builds function call graphs
   - `graph.rs`: Graph data structure with cycle detection
   - `resolver.rs`: Symbol resolution

6. **Detector (`src/detector/`)**
   - `existing.rs`: Finds existing `#[instrument]` macros
   - `priority.rs`: Context-aware prioritization
   - `gaps.rs`: Instrumentation gap analysis

7. **Framework Detection (`src/framework/web/`)**
   - `axum.rs`, `actix.rs`, `rocket.rs`, `tonic.rs`
   - Extracts routes from Router definitions

8. **Output Formatting (`src/output/`)**
   - `tree.rs`: Human-readable output with colors
   - `json.rs`: JSON for CI/CD
   - `mermaid.rs`: Mermaid diagrams

### Module Structure (Actual)

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Analyzer, AnalysisResult
├── config.rs            # Config struct
├── error.rs             # Error types
├── dependencies.rs      # Cargo.toml parsing (Phase 2)
├── ast/                 # AST analysis
│   ├── analyzer.rs
│   ├── visitor.rs
│   └── helpers.rs
├── call_graph/          # Call graph construction
│   ├── builder.rs
│   ├── graph.rs
│   ├── node.rs
│   ├── edge.rs
│   └── resolver.rs
├── detector/            # Instrumentation detection
│   ├── existing.rs
│   ├── priority.rs
│   └── gaps.rs
├── framework/web/       # Framework adapters
│   ├── axum.rs
│   ├── actix.rs
│   ├── rocket.rs
│   └── tonic.rs
├── patterns/            # Pattern matching
│   ├── matcher.rs
│   └── pattern_set.rs
└── output/              # Output formatting
    ├── tree.rs
    ├── json.rs
    └── mermaid.rs
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

### Test Files

```
tests/
├── e2e_tests.rs           # 16 comprehensive E2E tests
├── framework_detection.rs # Framework-specific tests
├── pattern_matching.rs    # Pattern matching tests
├── call_graph_test.rs     # Call graph construction
├── ast_test.rs            # AST analysis tests
└── common/mod.rs          # Test utilities & sample project generators
```

### Test Categories

- **Unit Tests**: `#[cfg(test)]` modules within source files (55+ tests)
- **E2E Tests**: Full pipeline tests in `tests/e2e_tests.rs`
  - Uses `tempfile::TempDir` for isolated test projects
  - Generates realistic Cargo.toml and source files
  - Tests dependency detection, endpoint detection, analysis stats
- **Integration Tests**: CLI workflow tests in `tests/`

### Running Tests

```bash
# Run all tests
cargo test

# Run only E2E tests
cargo test --test e2e_tests

# Run with output visible
cargo test -- --nocapture

# Test count: ~90 tests (55 unit + 16 E2E + others)
```

### Creating E2E Test Projects

Use helpers from `tests/common/mod.rs`:

```rust
use common::{TestProject, sample_projects};

let project = TestProject::new();
project.add_cargo_toml(r#"[package]..."#);
project.add_source_file("main.rs", "fn main() {}");

// Or use pre-built sample projects
let project = sample_projects::axum_web_app();
```

## Learnings & Common Issues

### CLI Output Parsing

When testing JSON output, filter out cargo build messages:
```bash
# Wrong - cargo messages break jq
cargo run --bin instrument-rs -- . --format json | jq '.'

# Correct - filter or redirect stderr
cargo run --bin instrument-rs -- . --format json 2>/dev/null | jq '.'
```

### Check Command

The `check` command operates on current directory, not a path argument:
```bash
# Wrong
instrument-rs check --threshold 80 /path/to/project

# Correct - run from project directory
cd /path/to/project && instrument-rs check --threshold 80
```

### Dependency Detection

The `DetectionContext` uses specific patterns to reduce false positives:
- `is_likely_http_call()` matches: `send_request`, `http_get`, `call_api`, `fetch_from`
- It does NOT match generic names like `get_user` (prevents false positives)

### Line Number Accuracy

Line numbers point to the router definition line, not the handler function definition.
Example: All endpoints show same line if defined in one `.route()` chain.

### E2E Test Project Location

Tests that require `/tmp/e2e-test-project` are marked with:
```rust
#[ignore = "Requires /tmp/e2e-test-project to exist locally"]
```

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

## Rust 2024 Edition Best Practices

This project uses **Rust 2024 edition** (requires Rust 1.85+).

### Key Changes from 2021

| Feature | 2021 | 2024 |
|---------|------|------|
| Pattern matching | `ref` allowed in implicit borrows | `ref` not allowed with implicit borrows |
| RPIT lifetime | Explicit `+ 'a` often needed | Automatic lifetime capture |
| unsafe fn | Body is implicitly unsafe | Requires explicit `unsafe {}` blocks |
| Prelude | Standard prelude | Adds `Future`, `IntoFuture` |
| async closures | `\|\| async {}` workaround | Native `async \|\| {}` syntax |

### Code Guidelines

```rust
// Pattern matching - remove unnecessary ref
// Before (2021)
match &value {
    Some(ref x) => use(x),
    None => {},
}

// After (2024)
match &value {
    Some(x) => use(x),  // ref is implicit
    None => {},
}

// RPIT lifetime capture - simplified
// Before (2021)
fn get_data<'a>(&'a self) -> impl Iterator<Item = &'a str> + 'a {
    self.items.iter().map(|s| s.as_str())
}

// After (2024)
fn get_data(&self) -> impl Iterator<Item = &str> {
    self.items.iter().map(|s| s.as_str())
}

// unsafe fn - explicit blocks required
unsafe fn dangerous(ptr: *const i32) -> i32 {
    // 2024: must wrap unsafe operations
    unsafe { *ptr }
}
```

### Configuration Files

- `Cargo.toml`: `edition = "2024"`, `rust-version = "1.85"`
- `rustfmt.toml`: `edition = "2024"`, `style_edition = "2024"`
- `clippy.toml`: Cognitive complexity and argument thresholds

### Lints (Cargo.toml)

```toml
[lints.rust]
unsafe_op_in_unsafe_fn = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

### Migration Commands

```bash
# Automatic migration (conservative)
cargo fix --edition

# Check compatibility
cargo clippy -- -W rust-2024-compatibility

# Format with 2024 style
cargo fmt
```

### Reserved Keywords

- `gen` is reserved for future generator syntax
- Update `rand` to 0.9+ if using `gen` as identifier

## Future Enhancements

Consider implementing:
- GitHub Actions for automated analysis in PRs
- Integration with OpenTelemetry configuration
- Support for more frameworks (warp, poem, salvo)
- IDE plugins for real-time instrumentation suggestions
- Benchmarks for performance impact analysis
- Integration with APM providers (DataDog, New Relic)