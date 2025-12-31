//! instrument-rs CLI
//!
//! A Rust CLI tool for detecting optimal instrumentation points for observability.

use clap::{Parser, Subcommand};
use instrument_rs::config::{FrameworkType, OutputFormat};
use instrument_rs::output::{FormatterFactory, FormatterOptions, write_output};
use instrument_rs::{Analyzer, Config};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "instrument-rs",
    about = "Detect optimal instrumentation points for observability",
    version,
    author
)]
struct Cli {
    /// Paths to analyze (default: current directory)
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    /// Trace from HTTP/gRPC endpoints
    #[arg(long)]
    trace_from_endpoints: bool,

    /// Web framework to use for endpoint detection
    #[arg(long, value_enum, default_value = "auto")]
    framework: FrameworkType,

    /// Output format
    #[arg(short, long, value_enum, default_value = "human")]
    format: OutputFormat,

    /// Filter paths by pattern (regex)
    #[arg(long)]
    filter_path: Option<String>,

    /// Maximum call graph depth
    #[arg(long, default_value = "10")]
    max_depth: usize,

    /// Detection threshold (0.0-1.0)
    #[arg(long, default_value = "0.8")]
    threshold: f64,

    /// Include test functions in analysis
    #[arg(long)]
    include_tests: bool,

    /// Custom patterns file
    #[arg(long)]
    patterns: Option<PathBuf>,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize configuration file
    Init {
        /// Output path for config file
        #[arg(short, long, default_value = "instrument-rs.toml")]
        output: PathBuf,
    },
    /// Check instrumentation coverage (for CI)
    Check {
        /// Minimum coverage threshold (0-100)
        #[arg(long, default_value = "80")]
        threshold: f64,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { output }) => {
            init_config(output)?;
        }
        Some(Commands::Check { threshold }) => {
            check_coverage(&cli, threshold)?;
        }
        None => {
            analyze(&cli)?;
        }
    }

    Ok(())
}

fn init_config(output: PathBuf) -> anyhow::Result<()> {
    let config = Config::default();
    config.save(&output)?;
    println!("Created config file: {}", output.display());
    Ok(())
}

fn check_coverage(cli: &Cli, threshold: f64) -> anyhow::Result<()> {
    let config = build_config(cli);
    let analyzer = Analyzer::new(config);

    let paths: Vec<&str> = cli
        .paths
        .iter()
        .map(|p| p.to_str().unwrap_or("."))
        .collect();
    let result = analyzer.analyze(&paths)?;

    let coverage = if result.stats.total_functions > 0 {
        (result.stats.instrumentation_points as f64 / result.stats.total_functions as f64) * 100.0
    } else {
        100.0
    };

    println!("Instrumentation coverage: {coverage:.1}%");

    if coverage < threshold {
        eprintln!("Coverage {coverage:.1}% is below threshold {threshold:.1}%");
        std::process::exit(1);
    }

    Ok(())
}

fn analyze(cli: &Cli) -> anyhow::Result<()> {
    let config = build_config(cli);
    let analyzer = Analyzer::new(config);

    let paths: Vec<&str> = cli
        .paths
        .iter()
        .map(|p| p.to_str().unwrap_or("."))
        .collect();
    let result = analyzer.analyze(&paths)?;

    let output_format = match cli.format {
        OutputFormat::Human => instrument_rs::output::OutputFormat::Tree,
        OutputFormat::Json => instrument_rs::output::OutputFormat::Json,
        OutputFormat::Mermaid => instrument_rs::output::OutputFormat::Mermaid,
    };

    let options = FormatterOptions {
        use_colors: atty::is(atty::Stream::Stdout),
        max_depth: Some(cli.max_depth),
        include_source: false,
        min_priority: None,
    };

    let formatter = FormatterFactory::create(output_format, options);
    let output = formatter.format(&result)?;

    write_output(&output, cli.output.as_deref())?;

    Ok(())
}

fn build_config(cli: &Cli) -> Config {
    Config {
        threshold: cli.threshold,
        max_depth: cli.max_depth,
        include_tests: cli.include_tests,
        framework: cli.framework,
        patterns_file: cli.patterns.clone(),
        exclude_patterns: vec![
            "target".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
        ],
        source_dirs: vec![PathBuf::from("src")],
    }
}
