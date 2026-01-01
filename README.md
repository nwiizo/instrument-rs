# instrument-rs

A Rust CLI tool for analyzing code and detecting optimal instrumentation points for observability (tracing, logging, metrics). It traces execution flows from HTTP/gRPC endpoints to identify critical paths that need monitoring, helping teams implement comprehensive observability strategies.

## 🎯 Overview

`instrument-rs` is a code analysis tool focused on observability that helps you:

**Smart Dependency-Aware Analysis:**
- Analyze `Cargo.toml` to understand project dependencies
- Context-aware detection based on used crates (sqlx, reqwest, redis, etc.)
- Reduce false positives by understanding what your project actually uses
- Auto-detect web frameworks (Axum, Actix-web, Rocket, Tonic)

**Call Graph & Path Tracing:**
- Build comprehensive call graphs to understand code structure
- Trace execution paths from HTTP/gRPC endpoints
- Identify critical business logic and external service calls
- Detect patterns in code (database operations, API calls, error handling)

**Instrumentation Detection:**
- Find existing `#[instrument]` macros and manual span creation
- Identify gaps in current observability coverage
- Suggest optimal points for tracing, logging, and metrics
- Score existing instrumentation quality

## ✨ Features

### Smart Analysis (Phase 2)
- **Dependency-Aware Detection**: Analyzes `Cargo.toml` to understand what crates your project uses
- **Context-Based Matching**: Prioritizes patterns based on detected dependencies
- **False Positive Reduction**: `get_user` won't be flagged as HTTP client when you use sqlx
- **Accurate Line Numbers**: Precise source locations using proc-macro2 span-locations

### Core Capabilities
- **AST-based Analysis**: Deep code analysis using Rust's syntax tree
- **Call Graph Construction**: Build comprehensive function call graphs
- **Pattern Recognition**: Configurable pattern matching for code constructs
- **Framework Detection**: Auto-detect web frameworks (Axum, Actix-web, Rocket, Tonic)
- **Existing Instrumentation Detection**: Find `#[instrument]` macros and manual spans

### Reporting & Visualization
- **Multiple Output Formats**: JSON, Mermaid, DOT, Console
- **Visual Call Graphs**: Generate interactive diagrams
- **Quality Scoring**: Evaluate and score existing instrumentation
- **Critical Path Identification**: Highlight paths needing observability

## 📦 Installation

```bash
# Install from crates.io (coming soon)
cargo install instrument-rs

# Build from source
git clone https://github.com/nwiizo/instrument-rs
cd instrument-rs
cargo build --release
```

## 🚀 Quick Start

```bash
# Basic usage - analyze current directory
instrument-rs .

# Trace from endpoints with visual output
instrument-rs . --trace-from-endpoints --format mermaid

# Analyze specific framework
instrument-rs . --framework axum --trace-from-endpoints

# Generate JSON report for CI/CD integration
instrument-rs . --format json > instrumentation-report.json
```

## 📊 Example Output

### Endpoint-based Analysis

```
Tracing from HTTP endpoints:
────────────────────────────────────────────────────────────
POST /api/v1/payments -> process_payment_handler
├── validate_auth_token (auth.rs:45)
│   └── verify_jwt (jwt.rs:23)
├── parse_payment_request (models.rs:89)
├── process_payment (payment.rs:123) ⚠️ Critical Path
│   ├── validate_card (validation.rs:45)
│   ├── check_fraud (fraud.rs:78) ⚠️ External Service
│   ├── charge_card (payment_gateway.rs:90) ⚠️ External Service
│   └── save_transaction (db.rs:234) ⚠️ Database Operation
└── send_confirmation (notification.rs:56) ⚠️ External Service

Critical paths identified: 8
External service calls: 4
Database operations: 2
```

### Visual Call Graph (Mermaid)

```mermaid
graph TD
    A[POST /payments] --> B[validate_auth]
    A --> C[process_payment]
    C --> D[validate_card]
    C --> E[charge_card]
    C --> F[save_transaction]
    E --> G[payment_gateway::charge]
    F --> H[db::insert]
    
    style C fill:#f9f,stroke:#333,stroke-width:4px
    style E fill:#ff9,stroke:#333,stroke-width:2px
    style G fill:#9ff,stroke:#333,stroke-width:2px
```

## 🔧 Command Line Options

```bash
instrument-rs [OPTIONS] [PATH] [COMMAND]

ARGUMENTS:
    [PATH]                           Path to analyze [default: .]

OPTIONS:
    -t, --threshold <FLOAT>          Detection threshold (0.0-1.0) [default: 0.8]
    -m, --min-lines <NUM>            Minimum function lines [default: 3]
    --trace-from-endpoints           Trace execution paths from entry points
    --framework <NAME>               Framework [axum|actix|rocket|tonic|auto]
    --max-depth <NUM>                Maximum call depth to analyze [default: 10]
    --include-tests                  Include test endpoints in analysis
    --show-call-graph                Display visual call graph
    --patterns <FILE>                Custom patterns file
    -f, --format <FORMAT>            Output format [human|json|dot|mermaid]
    --filter-path <REGEX>            Only show paths matching regex
    -c, --config <FILE>              Configuration file path
    -v, --verbose                    Increase verbosity (use multiple times)
    -h, --help                       Print help information
    -V, --version                    Print version information

COMMANDS:
    analyze                          Analyze code and suggest instrumentation points
    init                             Initialize a new configuration file
    generate                         Generate instrumentation code
    help                             Print this message or the help of subcommands
```

## 🤖 AI Integration

This tool is designed to work seamlessly with AI assistants:

```
`instrument-rs . --trace-from-endpoints` analyzes Rust code endpoints to trace 
critical execution paths and detect instrumentation points. Use this to create 
comprehensive observability implementation plans. Run `instrument-rs -h` for options.
```

## 📚 Documentation

- [Architecture Guide](docs/architecture.md) - Detailed design and architecture
- [Configuration Reference](instrument-rs.toml.example) - All configuration options
- [API Documentation](https://docs.rs/instrument-rs) - Full API reference (when published)
- [Changelog](CHANGELOG.md) - Version history and changes

## 📂 Project Structure

```
instrument-rs/
├── src/
│   ├── main.rs              # CLI entry point (streamlined)
│   ├── lib.rs               # Library interface
│   ├── dependencies.rs      # Cargo.toml dependency analysis (Phase 2)
│   ├── ast/                 # AST analysis and manipulation
│   │   ├── analyzer.rs      # Core analysis functionality
│   │   ├── visitor.rs       # AST traversal with accurate spans
│   │   └── helpers.rs       # AST manipulation helpers
│   ├── call_graph/          # Call graph construction
│   │   ├── builder.rs       # Graph builder
│   │   ├── graph.rs         # Graph data structure
│   │   └── resolver.rs      # Symbol resolution
│   ├── detector/            # Instrumentation detection
│   │   ├── existing.rs      # Existing instrumentation finder
│   │   ├── priority.rs      # Context-aware prioritization
│   │   └── patterns.rs      # Detection patterns
│   ├── framework/           # Framework detection
│   │   ├── detector.rs      # Auto-detection logic
│   │   └── web/             # Web framework adapters
│   │       ├── axum.rs      # Axum support
│   │       ├── actix.rs     # Actix-web support
│   │       ├── rocket.rs    # Rocket support
│   │       └── tonic.rs     # Tonic/gRPC support
│   ├── patterns/            # Pattern matching system
│   │   ├── matcher.rs       # Pattern matching engine
│   │   └── pattern_set.rs   # Pattern definitions
│   └── output/              # Output formatting
│       ├── json.rs          # JSON formatter
│       ├── mermaid.rs       # Mermaid diagrams
│       └── tree.rs          # Tree visualization
├── examples/                # Example usage
├── tests/                   # Integration tests
└── CLAUDE.md                # AI assistant instructions
```

## 🛠️ Development

### Building

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the CLI tool
cargo run -- . --trace-from-endpoints
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Generate documentation
cargo doc --no-deps --open
```

## 🚧 Roadmap

### Completed
- **Phase 1**: Core refactoring - streamlined architecture, removed unused modules
- **Phase 2**: Smart analysis - cargo_metadata integration, dependency-aware detection

### In Progress
- **Prometheus/OpenTelemetry Integration**: Verify compatibility with observability tools
- **Coverage Metrics**: Calculate observability coverage by module/criticality

### Planned
- **LSP Integration**: Type information for more accurate detection
- **Custom Pattern Files**: `.instrument-rs.toml` for project-specific patterns
- **Additional Frameworks**: Warp, Poem, Salvo support
- **Cost Optimization**: Telemetry cost estimation (DataDog, CloudWatch)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Run `cargo fmt` before committing
2. Ensure `cargo clippy` passes with no warnings
3. Add tests for new functionality
4. Update documentation as needed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Authors

- nwiizo

## Repository

[https://github.com/nwiizo/instrument-rs](https://github.com/nwiizo/instrument-rs)