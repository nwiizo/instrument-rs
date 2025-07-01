//! Mermaid diagram generator for visualization

use crate::reporting::{CombinedReport, CoverageSummary, FileCoverage};
use crate::mutation::MutationSummary;
use crate::Result;
use super::traits::{OutputFormatter, FormatterOptions, OutputFormat};
use std::fmt::Write;
use std::path::Path;
use std::collections::HashMap;

/// Helper macro to convert fmt::Error to our Error type
macro_rules! write_line {
    ($output:expr, $($arg:tt)*) => {
        writeln!($output, $($arg)*)
            .map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    };
}

/// Mermaid diagram formatter for visualization
pub struct MermaidFormatter {
    options: FormatterOptions,
}

impl MermaidFormatter {
    /// Create a new Mermaid formatter with the given options
    pub fn new(options: FormatterOptions) -> Self {
        Self { options }
    }
    
    /// Generate a coverage flowchart
    fn generate_coverage_flowchart(&self, summary: &CoverageSummary) -> Result<String> {
        let mut output = String::new();
        let mut coverage_groups: HashMap<String, Vec<&FileCoverage>> = HashMap::new();
        
        write_line!(&mut output, "```mermaid");
        write_line!(&mut output, "flowchart TD");
        write_line!(&mut output, "    Start[Project Coverage: {:.1}%]", summary.line_coverage_percent);
        write_line!(&mut output, "    Start --> Stats[Statistics]");
        write_line!(&mut output, "    Stats --> LineC[Line Coverage: {:.1}%]", summary.line_coverage_percent);
        write_line!(&mut output, "    Stats --> BranchC[Branch Coverage: {:.1}%]", summary.branch_coverage_percent);
        write_line!(&mut output, "    Stats --> FuncC[Function Coverage: {:.1}%]", summary.function_coverage_percent);
        
        if self.options.include_files && !summary.file_coverage.is_empty() {
            write_line!(&mut output, "    Start --> Files[Files: {}]", summary.file_coverage.len());
            
            // Group files by coverage range
            for file in &summary.file_coverage {
                let coverage = file.line_coverage_percent();
                let group = match coverage {
                    c if c >= 80.0 => "High Coverage (≥80%)",
                    c if c >= 60.0 => "Medium Coverage (60-79%)",
                    c if c >= 40.0 => "Low Coverage (40-59%)",
                    _ => "Very Low Coverage (<40%)",
                };
                coverage_groups.entry(group.to_string()).or_default().push(file);
            }
            
            for (group, files) in &coverage_groups {
                let group_id = group.replace(' ', "_").replace('(', "").replace(')', "").replace('%', "");
                write_line!(&mut output, "    Files --> {}[{}: {} files]", 
                    group_id, 
                    group, 
                    files.len()
                );
                
                if !self.options.compact {
                    for (i, file) in files.iter().enumerate().take(5) {
                        let file_name = file.file_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        write_line!(&mut output, "    {} --> File{}{}[{}: {:.1}%]",
                            group_id,
                            group_id,
                            i,
                            file_name,
                            file.line_coverage_percent()
                        );
                    }
                    if files.len() > 5 {
                        write_line!(&mut output, "    {} --> More{}[...and {} more]",
                            group_id,
                            group_id,
                            files.len() - 5
                        );
                    }
                }
            }
        }
        
        // Add styling
        write_line!(&mut output, "    ");
        write_line!(&mut output, "    classDef highCov fill:#4ade80,stroke:#16a34a,color:#fff");
        write_line!(&mut output, "    classDef medCov fill:#facc15,stroke:#ca8a04,color:#000");
        write_line!(&mut output, "    classDef lowCov fill:#fb923c,stroke:#ea580c,color:#fff");
        write_line!(&mut output, "    classDef veryLowCov fill:#f87171,stroke:#dc2626,color:#fff");
        write_line!(&mut output, "    classDef stats fill:#a78bfa,stroke:#7c3aed,color:#fff");
        write_line!(&mut output, "    ");
        write_line!(&mut output, "    class Stats,LineC,BranchC,FuncC stats");
        if coverage_groups.contains_key("High Coverage (≥80%)") {
            write_line!(&mut output, "    class High_Coverage_80 highCov");
        }
        if coverage_groups.contains_key("Medium Coverage (60-79%)") {
            write_line!(&mut output, "    class Medium_Coverage_60_79 medCov");
        }
        if coverage_groups.contains_key("Low Coverage (40-59%)") {
            write_line!(&mut output, "    class Low_Coverage_40_59 lowCov");
        }
        if coverage_groups.contains_key("Very Low Coverage (<40%)") {
            write_line!(&mut output, "    class Very_Low_Coverage_40 veryLowCov");
        }
        
        write_line!(&mut output, "```");
        
        Ok(output)
    }
    
    /// Generate a mutation testing pie chart
    fn generate_mutation_pie_chart(&self, summary: &MutationSummary) -> Result<String> {
        let mut output = String::new();
        write_line!(&mut output, "```mermaid");
        write_line!(&mut output, "pie title Mutation Testing Results");
        write_line!(&mut output, "    \"Killed\" : {}", summary.mutations_killed);
        write_line!(&mut output, "    \"Survived\" : {}", summary.mutations_survived);
        write_line!(&mut output, "    \"Timeout\" : {}", summary.mutations_timeout);
        if summary.compile_errors > 0 {
            write_line!(&mut output, "    \"Compile Errors\" : {}", summary.compile_errors);
        }
        write_line!(&mut output, "```");
        
        Ok(output)
    }
    
    /// Generate a combined quality dashboard
    fn generate_quality_dashboard(&self, report: &CombinedReport) -> Result<String> {
        let mut output = String::new();
        
        // Main flowchart
        write_line!(&mut output, "```mermaid");
        write_line!(&mut output, "flowchart TB");
        write_line!(&mut output, "    subgraph Dashboard[\"📊 {} Quality Dashboard\"]", report.project_name);
        write_line!(&mut output, "        Score[Overall Score: {:.1}%]", report.quality_score);
        
        if let Some(ref coverage) = report.coverage {
            write_line!(&mut output, "        Score --> Coverage[Coverage Metrics]");
            write_line!(&mut output, "        Coverage --> CovLine[Line: {:.1}%]", coverage.line_coverage_percent);
            write_line!(&mut output, "        Coverage --> CovBranch[Branch: {:.1}%]", coverage.branch_coverage_percent);
            write_line!(&mut output, "        Coverage --> CovFunc[Function: {:.1}%]", coverage.function_coverage_percent);
        }
        
        if let Some(ref mutations) = report.mutations {
            write_line!(&mut output, "        Score --> Mutations[Mutation Testing]");
            write_line!(&mut output, "        Mutations --> MutScore[Score: {:.1}%]", mutations.mutation_score);
            write_line!(&mut output, "        Mutations --> MutStats[K:{} S:{} T:{}]",
                mutations.mutations_killed,
                mutations.mutations_survived,
                mutations.mutations_timeout
            );
        }
        
        write_line!(&mut output, "    end");
        
        // Add quality gates
        write_line!(&mut output, "    ");
        write_line!(&mut output, "    Dashboard --> QualityGate{{Quality Gate}}");
        
        let gate_status = if report.quality_score >= 80.0 {
            "Pass[✅ PASS]"
        } else if report.quality_score >= 60.0 {
            "Warning[⚠️ WARNING]"
        } else {
            "Fail[❌ FAIL]"
        };
        write_line!(&mut output, "    QualityGate --> {}", gate_status);
        
        // Add styling
        write_line!(&mut output, "    ");
        write_line!(&mut output, "    classDef good fill:#4ade80,stroke:#16a34a,color:#fff");
        write_line!(&mut output, "    classDef warning fill:#facc15,stroke:#ca8a04,color:#000");
        write_line!(&mut output, "    classDef bad fill:#f87171,stroke:#dc2626,color:#fff");
        write_line!(&mut output, "    classDef metric fill:#60a5fa,stroke:#2563eb,color:#fff");
        write_line!(&mut output, "    ");
        
        if report.quality_score >= 80.0 {
            write_line!(&mut output, "    class Pass good");
            write_line!(&mut output, "    class Score good");
        } else if report.quality_score >= 60.0 {
            write_line!(&mut output, "    class Warning warning");
            write_line!(&mut output, "    class Score warning");
        } else {
            write_line!(&mut output, "    class Fail bad");
            write_line!(&mut output, "    class Score bad");
        }
        
        write_line!(&mut output, "    class Coverage,Mutations,CovLine,CovBranch,CovFunc,MutScore,MutStats metric");
        write_line!(&mut output, "```");
        
        Ok(output)
    }
    
    /// Generate a file tree diagram
    fn generate_file_tree(&self, summary: &CoverageSummary) -> Result<String> {
        let mut output = String::new();
        write_line!(&mut output, "```mermaid");
        write_line!(&mut output, "graph TD");
        
        // Group files by directory
        let mut dir_map: HashMap<String, Vec<&FileCoverage>> = HashMap::new();
        for file in &summary.file_coverage {
            let dir = file.file_path.parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".");
            dir_map.entry(dir.to_string()).or_default().push(file);
        }
        
        write_line!(&mut output, "    Root[Project Root]");
        
        for (i, (dir, files)) in dir_map.iter().enumerate() {
            let dir_id = format!("Dir{}", i);
            let dir_name = Path::new(dir).file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(dir);
            
            write_line!(&mut output, "    Root --> {}[\"{}\"]", dir_id, dir_name);
            
            for (j, file) in files.iter().enumerate().take(10) {
                let file_id = format!("File{}{}", i, j);
                let file_name = file.file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let coverage = file.line_coverage_percent();
                
                write_line!(&mut output, "    {} --> {}[\"{}: {:.1}%\"]",
                    dir_id, file_id, file_name, coverage
                );
                
                // Style based on coverage
                let style_class = if coverage >= 80.0 {
                    "highCov"
                } else if coverage >= 60.0 {
                    "medCov"
                } else {
                    "lowCov"
                };
                write_line!(&mut output, "    class {} {}", file_id, style_class);
            }
            
            if files.len() > 10 {
                write_line!(&mut output, "    {} --> More{}[\"...{} more files\"]",
                    dir_id, i, files.len() - 10
                );
            }
        }
        
        // Add styling
        write_line!(&mut output, "    ");
        write_line!(&mut output, "    classDef highCov fill:#4ade80,stroke:#16a34a");
        write_line!(&mut output, "    classDef medCov fill:#facc15,stroke:#ca8a04");
        write_line!(&mut output, "    classDef lowCov fill:#f87171,stroke:#dc2626");
        write_line!(&mut output, "    classDef dir fill:#e0e7ff,stroke:#6366f1");
        write_line!(&mut output, "    ");
        
        for (i, _) in dir_map.iter().enumerate() {
            write_line!(&mut output, "    class Dir{} dir", i);
        }
        
        write_line!(&mut output, "```");
        
        Ok(output)
    }
}

impl OutputFormatter for MermaidFormatter {
    fn format_coverage(&self, summary: &CoverageSummary) -> Result<String> {
        let mut output = String::new();
        
        // Generate flowchart
        output.push_str(&self.generate_coverage_flowchart(summary)?);
        
        if !self.options.compact {
            output.push_str("\n\n");
            // Generate file tree
            output.push_str(&self.generate_file_tree(summary)?);
        }
        
        Ok(output)
    }
    
    fn format_mutations(&self, summary: &MutationSummary) -> Result<String> {
        let mut output = String::new();
        
        // Generate pie chart
        output.push_str(&self.generate_mutation_pie_chart(summary)?);
        
        if !self.options.compact && self.options.include_files {
            output.push_str("\n\n");
            
            // Generate mutation flow
            write_line!(&mut output, "```mermaid");
            write_line!(&mut output, "flowchart LR");
            write_line!(&mut output, "    Total[Total Mutations: {}]", summary.total_mutations);
            write_line!(&mut output, "    Total --> Killed[Killed: {}]", summary.mutations_killed);
            write_line!(&mut output, "    Total --> Survived[Survived: {}]", summary.mutations_survived);
            write_line!(&mut output, "    Total --> Timeout[Timeout: {}]", summary.mutations_timeout);
            write_line!(&mut output, "    Total --> Score[Score: {:.1}%]", summary.mutation_score);
            write_line!(&mut output, "    ");
            write_line!(&mut output, "    classDef good fill:#4ade80,stroke:#16a34a");
            write_line!(&mut output, "    classDef bad fill:#f87171,stroke:#dc2626");
            write_line!(&mut output, "    classDef neutral fill:#60a5fa,stroke:#2563eb");
            write_line!(&mut output, "    ");
            write_line!(&mut output, "    class Killed good");
            write_line!(&mut output, "    class Survived bad");
            write_line!(&mut output, "    class Timeout,Score neutral");
            write_line!(&mut output, "```");
        }
        
        Ok(output)
    }
    
    fn format_combined(&self, report: &CombinedReport) -> Result<String> {
        self.generate_quality_dashboard(report)
    }
    
    fn format_file_coverage(&self, file: &FileCoverage) -> Result<String> {
        let mut output = String::new();
        
        write_line!(&mut output, "```mermaid");
        write_line!(&mut output, "flowchart TD");
        
        let file_name = file.file_path.display();
        let coverage = file.line_coverage_percent();
        
        write_line!(&mut output, "    File[\"📄 {}\"]", file_name);
        write_line!(&mut output, "    File --> Coverage[Coverage: {:.1}%]", coverage);
        write_line!(&mut output, "    File --> Lines[Lines: {}/{}]", 
            file.covered_lines, file.executable_lines);
        
        if !file.branch_coverage.is_empty() {
            let total_branches = file.branch_coverage.len() * 2;
            let covered_branches: usize = file.branch_coverage.iter()
                .map(|b| {
                    (if b.true_count > 0 { 1 } else { 0 }) +
                    (if b.false_count > 0 { 1 } else { 0 })
                })
                .sum();
            write_line!(&mut output, "    File --> Branches[Branches: {}/{}]", 
                covered_branches, total_branches);
        }
        
        if !file.function_coverage.is_empty() {
            let covered_functions = file.function_coverage.iter()
                .filter(|f| f.hit_count > 0)
                .count();
            write_line!(&mut output, "    File --> Functions[Functions: {}/{}]",
                covered_functions, file.function_coverage.len());
        }
        
        // Add uncovered lines if not too many
        let uncovered = file.uncovered_lines();
        if !uncovered.is_empty() && uncovered.len() <= 10 {
            write_line!(&mut output, "    Coverage --> Uncovered[Uncovered Lines]");
            for line in &uncovered {
                write_line!(&mut output, "    Uncovered --> L{}[Line {}]", line, line);
            }
        }
        
        // Styling
        write_line!(&mut output, "    ");
        let coverage_class = if coverage >= 80.0 { "good" } 
            else if coverage >= 60.0 { "warning" } 
            else { "bad" };
        write_line!(&mut output, "    classDef good fill:#4ade80,stroke:#16a34a");
        write_line!(&mut output, "    classDef warning fill:#facc15,stroke:#ca8a04");
        write_line!(&mut output, "    classDef bad fill:#f87171,stroke:#dc2626");
        write_line!(&mut output, "    classDef info fill:#60a5fa,stroke:#2563eb");
        write_line!(&mut output, "    ");
        write_line!(&mut output, "    class Coverage {}", coverage_class);
        write_line!(&mut output, "    class Lines,Branches,Functions info");
        
        write_line!(&mut output, "```");
        
        Ok(output)
    }
    
    fn format_type(&self) -> OutputFormat {
        OutputFormat::Mermaid
    }
    
    fn options(&self) -> &FormatterOptions {
        &self.options
    }
}