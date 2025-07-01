//! Console report generation

use crate::mutation::MutationSummary;
use crate::reporting::{CombinedReport, CoverageSummary, ReportFormat, ReportGenerator};
use crate::Result;
use std::path::Path;

/// Console report generator
pub struct ConsoleReportGenerator;

impl ConsoleReportGenerator {
    /// Create a new console generator
    pub fn new() -> Self {
        Self
    }
}

impl ReportGenerator for ConsoleReportGenerator {
    fn format(&self) -> ReportFormat {
        ReportFormat::Console
    }

    fn generate_coverage_report(
        &self,
        summary: &CoverageSummary,
        output_path: &Path,
    ) -> Result<()> {
        println!("Coverage Summary:");
        println!("  Total Files: {}", summary.total_files);
        println!("  Line Coverage: {:.1}%", summary.line_coverage_percent);
        println!("  Branch Coverage: {:.1}%", summary.branch_coverage_percent);
        println!(
            "  Function Coverage: {:.1}%",
            summary.function_coverage_percent
        );
        Ok(())
    }

    fn generate_mutation_report(
        &self,
        summary: &MutationSummary,
        output_path: &Path,
    ) -> Result<()> {
        println!("Mutation Testing Summary:");
        println!("  Total Mutations: {}", summary.total_mutations);
        println!("  Killed: {}", summary.killed);
        println!("  Survived: {}", summary.survived);
        println!("  Mutation Score: {:.1}%", summary.mutation_score);
        Ok(())
    }

    fn generate_combined_report(&self, report: &CombinedReport, output_path: &Path) -> Result<()> {
        println!("Combined Quality Report:");
        println!("  Project: {}", report.project_name);
        println!("  Overall Quality Score: {:.1}%", report.quality_score);

        if let Some(coverage) = &report.coverage {
            println!("\nCoverage:");
            println!("  Line: {:.1}%", coverage.line_coverage_percent);
            println!("  Branch: {:.1}%", coverage.branch_coverage_percent);
            println!("  Function: {:.1}%", coverage.function_coverage_percent);
        }

        if let Some(mutations) = &report.mutations {
            println!("\nMutation Testing:");
            println!("  Score: {:.1}%", mutations.mutation_score);
            println!(
                "  Killed: {} / {}",
                mutations.killed, mutations.total_mutations
            );
        }

        Ok(())
    }
}
