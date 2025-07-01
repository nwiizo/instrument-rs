//! JSON report generation

use crate::mutation::MutationSummary;
use crate::reporting::{CombinedReport, CoverageSummary, ReportFormat, ReportGenerator};
use crate::Result;
use std::path::Path;

/// JSON report generator
pub struct JsonReportGenerator;

impl JsonReportGenerator {
    /// Create a new JSON generator
    pub fn new() -> Self {
        Self
    }
}

impl ReportGenerator for JsonReportGenerator {
    fn format(&self) -> ReportFormat {
        ReportFormat::Json
    }

    fn generate_coverage_report(
        &self,
        summary: &CoverageSummary,
        output_path: &Path,
    ) -> Result<()> {
        let json = serde_json::to_string_pretty(summary)?;
        std::fs::write(output_path, json)?;
        Ok(())
    }

    fn generate_mutation_report(
        &self,
        summary: &MutationSummary,
        output_path: &Path,
    ) -> Result<()> {
        let json = serde_json::to_string_pretty(summary)?;
        std::fs::write(output_path, json)?;
        Ok(())
    }

    fn generate_combined_report(&self, report: &CombinedReport, output_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        std::fs::write(output_path, json)?;
        Ok(())
    }
}
