# instrument-rs CLI Usage Guide

The `instrument-rs` command-line tool provides a comprehensive interface for analyzing and instrumenting Rust projects for coverage and mutation testing.

## Installation

```bash
cargo install instrument-rs
```

## Quick Start

```bash
# Initialize a configuration file
instrument-rs init

# Analyze your project
instrument-rs analyze

# Generate reports
instrument-rs report --formats html json
```

## Commands

### `init` - Initialize Configuration

Create a new `instrument-rs.toml` configuration file in the current directory.

```bash
# Create default configuration
instrument-rs init

# Create minimal configuration
instrument-rs init --minimal

# Force overwrite existing configuration
instrument-rs init --force
```

### `analyze` - Analyze Project

Analyze a Rust project and apply instrumentation for coverage and/or mutation testing.

```bash
# Basic analysis with default settings
instrument-rs analyze

# Analyze a specific directory
instrument-rs analyze /path/to/project

# Specify instrumentation mode
instrument-rs analyze --mode coverage    # Coverage only
instrument-rs analyze --mode mutation    # Mutation only
instrument-rs analyze --mode combined    # Both coverage and mutation

# Different output formats
instrument-rs analyze --format human     # Default: colored terminal output
instrument-rs analyze --format json      # JSON output
instrument-rs analyze --format mermaid   # Mermaid diagram for call graphs

# Save output to file
instrument-rs analyze --format json --output analysis.json

# Dry run - analyze without writing files
instrument-rs analyze --dry-run

# Generate report after analysis
instrument-rs analyze --report

# Include source code in output
instrument-rs analyze --include-source

# Specify number of threads
instrument-rs analyze -j 8
```

### `mutate` - Run Mutation Testing

Run mutation tests on an already instrumented project.

```bash
# Run with default operators
instrument-rs mutate

# Specify mutation operators
instrument-rs mutate --operators arithmetic comparison logical

# Set maximum mutations
instrument-rs mutate --max-mutations 50

# Use specific random seed for reproducibility
instrument-rs mutate --seed 12345

# Set timeout per mutation test
instrument-rs mutate --timeout 60
```

### `report` - Generate Reports

Generate reports from existing instrumentation data.

```bash
# Generate HTML and JSON reports
instrument-rs report --formats html json

# Specify output directory
instrument-rs report --formats html --output-dir ./reports

# Open HTML report in browser after generation
instrument-rs report --formats html --open
```

### `clean` - Clean Artifacts

Remove instrumentation artifacts and generated files.

```bash
# Clean instrumentation and report directories
instrument-rs clean

# Also remove configuration file
instrument-rs clean --all

# Force removal without confirmation
instrument-rs clean --force
```

## Global Options

These options can be used with any command:

- `-c, --config <FILE>` - Path to configuration file (default: `instrument-rs.toml`)
- `-v, --verbose` - Increase verbosity (-v, -vv, -vvv for more detail)
- `-q, --quiet` - Suppress all output except errors
- `-j, --threads <N>` - Number of threads to use

## Configuration File

The tool uses a TOML configuration file (`instrument-rs.toml`) to specify project settings. See `instrument-rs.toml.example` for a complete example.

Key configuration sections:
- `[project]` - Project paths and exclusions
- `[instrumentation]` - Instrumentation settings
- `[mutation]` - Mutation testing configuration
- `[reporting]` - Report generation settings

## Output Formats

### Human Format
Default colored terminal output with progress indicators and summary statistics.

### JSON Format
Structured JSON output containing:
- Analysis metadata
- File and function statistics
- Coverage points
- Mutation information
- Call graph data

### Mermaid Format
Generates Mermaid diagram syntax for visualizing:
- Function call graphs
- Module dependencies
- Test coverage paths

## Examples

### Basic Coverage Analysis
```bash
# Initialize configuration
instrument-rs init

# Run coverage analysis
instrument-rs analyze --mode coverage

# Generate HTML report
instrument-rs report --formats html --open
```

### Mutation Testing Workflow
```bash
# Analyze with mutation instrumentation
instrument-rs analyze --mode mutation

# Run mutation tests with specific operators
instrument-rs mutate --operators arithmetic comparison

# Generate comprehensive report
instrument-rs report --formats html json
```

### CI/CD Integration
```bash
# Run in CI with machine-readable output
instrument-rs analyze --format json --output results.json

# Check coverage thresholds (configured in instrument-rs.toml)
instrument-rs analyze --mode coverage || exit 1
```

### Advanced Analysis
```bash
# Full analysis with verbose output
instrument-rs -vv analyze --mode combined --report

# Analyze specific directories with custom config
instrument-rs -c custom.toml analyze ./src --include-source
```

## Environment Variables

- `RUST_LOG` - Control logging level (trace, debug, info, warn, error)
- `NO_COLOR` - Disable colored output
- `INSTRUMENT_RS_THREADS` - Default number of threads

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Analysis error
- `4` - Threshold not met