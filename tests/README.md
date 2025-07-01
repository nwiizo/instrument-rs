# Integration Tests for instrument-rs

This directory contains comprehensive integration tests for the instrument-rs library.

## Test Structure

### Test Modules

1. **common/mod.rs** - Test utilities and helpers
   - `TestProject` - Creates temporary project structures
   - `sample_projects` - Generators for different project types:
     - `simple_library()` - Basic Rust library with functions
     - `axum_web_app()` - Axum web application with routes
     - `complex_patterns()` - Project with traits, async, generics
     - `large_codebase()` - 50+ modules for performance testing
   - `create_sample_config_file()` - Creates test configuration files
   - `assertions` - Helper assertions for test validation

2. **e2e_analysis.rs** - End-to-end analysis tests
   - Tests complete project analysis workflow
   - Verifies AST analysis accuracy
   - Tests call graph generation
   - Validates instrumentation scoring
   - Checks mutation target identification

3. **framework_detection_integration.rs** - Framework detection tests
   - Tests detection of web frameworks (Axum, Actix-web, Rocket, Warp)
   - Tests detection of test frameworks
   - Validates framework-specific pattern recognition
   - Tests framework adapter functionality

4. **output_format_tests.rs** - Output format generation tests
   - Tests JSON output formatting
   - Tests tree-view output formatting
   - Tests Mermaid diagram generation
   - Tests HTML report generation
   - Validates output consistency across formats

5. **config_parsing_tests.rs** - Configuration parsing tests
   - Tests default configuration values
   - Tests TOML config file parsing
   - Tests configuration validation
   - Tests save/load roundtrip
   - Tests all configuration options

6. **performance_tests.rs** - Performance benchmarks
   - Tests analysis performance on large codebases
   - Tests parallel processing performance
   - Tests incremental analysis efficiency
   - Tests memory usage patterns
   - Validates performance thresholds

## Running Tests

### Run all integration tests:
```bash
cargo test --test integration_tests
```

### Run specific test module:
```bash
cargo test --test integration_tests e2e_analysis
```

### Run with output:
```bash
cargo test --test integration_tests -- --nocapture
```

### Run performance tests (may take longer):
```bash
cargo test --test integration_tests performance_tests -- --nocapture
```

## Test Data

The tests create temporary directories and sample projects on the fly. No external test data is required.

## Performance Thresholds

The performance tests enforce the following thresholds:
- File parsing: < 100ms per file
- Analysis: < 5ms per function
- Call graph building: < 50ms per file
- Scoring: < 2ms per function

## Adding New Tests

When adding new integration tests:

1. Add test functions to the appropriate module
2. Use the `TestProject` helper from `common` module
3. Follow the naming convention `test_<feature>_<scenario>`
4. Include assertions for both success and failure cases
5. Add performance checks for operations that might scale

## Known Issues

Note: Some tests may fail if the main library has compilation errors. The tests are designed to work with a fully functional instrument-rs library.