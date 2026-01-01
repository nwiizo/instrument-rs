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
        /// Paths to analyze (default: current directory)
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,

        /// Minimum coverage threshold (0-100)
        #[arg(long, default_value = "80")]
        threshold: f64,

        /// Only consider critical gaps (ignore minor/major)
        #[arg(long)]
        critical_only: bool,

        /// Output format (human or json)
        #[arg(short, long, value_enum, default_value = "human")]
        format: OutputFormat,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { output }) => {
            init_config(output)?;
        }
        Some(Commands::Check {
            ref paths,
            threshold,
            critical_only,
            format,
        }) => {
            check_coverage(&cli, paths, threshold, critical_only, format)?;
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

fn check_coverage(
    cli: &Cli,
    paths: &[PathBuf],
    threshold: f64,
    critical_only: bool,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let config = build_config(cli);
    let analyzer = Analyzer::new(config);

    let paths: Vec<&str> = paths.iter().map(|p| p.to_str().unwrap_or(".")).collect();
    let result = analyzer.analyze(&paths)?;

    // Calculate gap-based coverage (more accurate)
    let total_points = result.stats.instrumentation_points;
    let gaps = if critical_only {
        result
            .gaps
            .iter()
            .filter(|g| matches!(g.severity, instrument_rs::detector::GapSeverity::Critical))
            .count()
    } else {
        result.stats.gaps_count
    };

    let covered = total_points.saturating_sub(gaps);
    let coverage = if total_points > 0 {
        (covered as f64 / total_points as f64) * 100.0
    } else {
        100.0
    };

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "coverage": coverage,
                "threshold": threshold,
                "passed": coverage >= threshold,
                "total_points": total_points,
                "covered": covered,
                "gaps": gaps,
                "critical_only": critical_only,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("Instrumentation Coverage Check");
            println!("══════════════════════════════");
            println!();
            println!("📊 Results:");
            println!("   Total instrumentation points: {}", total_points);
            println!(
                "   Existing instrumentation:     {}",
                result.stats.existing_count
            );
            println!(
                "   Gaps:                         {}{}",
                gaps,
                if critical_only {
                    " (critical only)"
                } else {
                    ""
                }
            );
            println!("   Coverage:                     {:.1}%", coverage);
            println!("   Threshold:                    {:.1}%", threshold);
            println!();

            if coverage >= threshold {
                println!("✅ PASSED: Coverage meets threshold");
            } else {
                println!(
                    "❌ FAILED: Coverage {:.1}% is below threshold {:.1}%",
                    coverage, threshold
                );

                // Show critical gaps
                if !result.gaps.is_empty() {
                    println!();
                    println!("🚨 Critical gaps to fix:");
                    for gap in result.gaps.iter().take(5) {
                        println!(
                            "   - {} ({}:{})",
                            gap.location.function_name,
                            gap.location.file.display(),
                            gap.location.line
                        );
                    }
                    if result.gaps.len() > 5 {
                        println!("   ... and {} more", result.gaps.len() - 5);
                    }
                }
            }
        }
    }

    if coverage < threshold {
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
