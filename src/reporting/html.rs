//! HTML report generation

use crate::mutation::MutationSummary;
use crate::reporting::{CombinedReport, CoverageSummary, ReportFormat, ReportGenerator};
use crate::Result;
use std::path::Path;

/// HTML report generator
pub struct HtmlReportGenerator;

impl HtmlReportGenerator {
    /// Create a new HTML generator
    pub fn new() -> Self {
        Self
    }
}

impl ReportGenerator for HtmlReportGenerator {
    fn format(&self) -> ReportFormat {
        ReportFormat::Html
    }

    fn generate_coverage_report(
        &self,
        summary: &CoverageSummary,
        output_path: &Path,
    ) -> Result<()> {
        // TODO: Implement HTML coverage report
        Ok(())
    }

    fn generate_mutation_report(
        &self,
        summary: &MutationSummary,
        output_path: &Path,
    ) -> Result<()> {
        // TODO: Implement HTML mutation report
        Ok(())
    }

    fn generate_combined_report(&self, report: &CombinedReport, output_path: &Path) -> Result<()> {
        // TODO: Implement combined HTML report
        Ok(())
    }
}
