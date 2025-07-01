# Changelog

All notable changes to instrument-rs will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of instrument-rs
- AST-based code analysis using `syn` crate
- Call graph construction with cross-module resolution
- Pattern matching system for identifying code constructs
- Web framework detection (Axum, Actix-web, Rocket, Tonic)
- Mutation testing with multiple operators
- Coverage tracking instrumentation
- Multiple output formats (JSON, HTML, Mermaid, DOT)
- Configuration file support (TOML)
- CLI interface with subcommands
- Parallel processing for performance
- Comprehensive documentation

### Framework Support
- Axum: Handler detection, middleware analysis, route extraction
- Actix-web: Service detection, actor analysis, handler identification
- Rocket: Route handler detection, guard analysis, fairing support
- Tonic: gRPC service detection, method analysis, interceptor support

### Mutation Operators
- Arithmetic operator replacement (+, -, *, /, %)
- Comparison operator replacement (<, >, <=, >=, ==, !=)
- Logical operator replacement (&&, ||, !)
- Assignment operator replacement (+=, -=, *=, /=, %=)
- Statement deletion
- Constant replacement
- Return value replacement
- Function call replacement
- Loop condition modification

### Output Formats
- Human-readable console output with colors
- JSON for CI/CD integration
- HTML reports with syntax highlighting
- Mermaid diagrams for documentation
- DOT format for Graphviz
- LCOV format for coverage tools
- JUnit XML for test reports

## [0.1.0] - TBD

Initial public release. See Unreleased section for features.