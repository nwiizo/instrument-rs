//! Human-readable tree view formatter for displaying hierarchical coverage and mutation data

use crate::reporting::{CombinedReport, CoverageSummary, FileCoverage};
use crate::mutation::{MutationSummary, MutationResult};
use crate::Result;
use super::traits::{OutputFormatter, FormatterOptions, OutputFormat, FormattedNode, NodeType};
use super::utils::{TreeBuilder, colorize, bold, dim, format_coverage, format_coverage_bar, colors};
use std::path::Path;

/// Tree view formatter for human-readable output
pub struct TreeFormatter {
    options: FormatterOptions,
}

impl TreeFormatter {
    /// Create a new tree formatter with the given options
    pub fn new(options: FormatterOptions) -> Self {
        Self { options }
    }
    
    /// Build a formatted node tree from coverage summary
    fn build_coverage_tree(&self, summary: &CoverageSummary) -> FormattedNode {
        let mut root = FormattedNode::new(
            "Project Coverage Summary".to_string(),
            NodeType::Root,
        );
        
        // Add overall statistics
        let stats_node = FormattedNode::new("Statistics".to_string(), NodeType::Summary)
            .with_value(format!(
                "Line: {:.1}% | Branch: {:.1}% | Function: {:.1}%",
                summary.line_coverage_percent,
                summary.branch_coverage_percent,
                summary.function_coverage_percent
            ));
        root.add_child(stats_node);
        
        // Add file coverage if requested
        if self.options.include_files {
            let mut files_node = FormattedNode::new(
                format!("Files ({})", summary.file_coverage.len()),
                NodeType::Directory,
            );
            
            for file in &summary.file_coverage {
                let file_node = self.build_file_node(file);
                files_node.add_child(file_node);
            }
            
            if self.options.sort_by_coverage {
                files_node.sort_by_coverage();
            }
            
            if let Some(threshold) = self.options.coverage_threshold {
                files_node.filter_by_coverage(threshold);
            }
            
            root.add_child(files_node);
        }
        
        root
    }
    
    /// Build a formatted node for a file
    fn build_file_node(&self, file: &FileCoverage) -> FormattedNode {
        let coverage = file.line_coverage_percent();
        let file_name = file.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>");
        
        let mut node = FormattedNode::new(file_name.to_string(), NodeType::File)
            .with_coverage(coverage)
            .with_value(format_coverage(coverage, self.options.use_color));
        
        node.metadata.path = Some(file.file_path.display().to_string());
        
        // Add uncovered lines if requested
        if self.options.include_lines && !self.options.compact {
            let uncovered = file.uncovered_lines();
            if !uncovered.is_empty() {
                let lines_node = FormattedNode::new(
                    format!("Uncovered lines: {:?}", uncovered),
                    NodeType::Line,
                );
                node.add_child(lines_node);
            }
        }
        
        node
    }
    
    /// Format a node tree into a string
    fn format_tree(&self, root: &FormattedNode) -> String {
        let mut builder = TreeBuilder::new(self.options.use_color);
        self.format_node(&mut builder, root, "", true, 0);
        builder.build()
    }
    
    /// Recursively format a node and its children
    fn format_node(
        &self,
        builder: &mut TreeBuilder,
        node: &FormattedNode,
        prefix: &str,
        is_last: bool,
        depth: usize,
    ) {
        // Check max depth
        if let Some(max) = self.options.max_depth {
            if depth > max {
                return;
            }
        }
        
        // Determine color based on node type and coverage
        let color = match node.node_type {
            NodeType::Root => Some(colors::BOLD),
            NodeType::Directory => Some(colors::BLUE),
            NodeType::File => {
                if let Some(cov) = node.metadata.coverage {
                    Some(super::utils::coverage_color(cov))
                } else {
                    Some(colors::WHITE)
                }
            }
            NodeType::Function => Some(colors::CYAN),
            NodeType::Summary => Some(colors::MAGENTA),
            NodeType::Error => Some(colors::RED),
            _ => None,
        };
        
        // Add the node
        if depth > 0 {
            builder.add_node(prefix, is_last, &node.label, node.value.as_deref(), color);
        } else {
            // Root node - no tree characters
            let label = if let Some(c) = color {
                colorize(&node.label, c, self.options.use_color)
            } else {
                node.label.clone()
            };
            
            let mut line = bold(&label, self.options.use_color);
            if let Some(ref v) = node.value {
                line.push(' ');
                line.push_str(v);
            }
            builder.lines.push(line);
        }
        
        // Format children
        let child_prefix = if depth > 0 {
            TreeBuilder::child_prefix(prefix, is_last)
        } else {
            String::new()
        };
        
        for (i, child) in node.children.iter().enumerate() {
            let child_is_last = i == node.children.len() - 1;
            self.format_node(builder, child, &child_prefix, child_is_last, depth + 1);
        }
    }
    
    /// Build a formatted node tree from mutation summary
    fn build_mutation_tree(&self, summary: &MutationSummary) -> FormattedNode {
        let mut root = FormattedNode::new(
            "Mutation Testing Summary".to_string(),
            NodeType::Root,
        );
        
        // Add overall statistics
        let stats_node = FormattedNode::new("Statistics".to_string(), NodeType::Summary)
            .with_value(format!(
                "Score: {:.1}% | Total: {} | Killed: {} | Survived: {} | Timeout: {}",
                summary.mutation_score,
                summary.total_mutations,
                summary.mutations_killed,
                summary.mutations_survived,
                summary.mutations_timeout
            ));
        root.add_child(stats_node);
        
        // Group mutations by file
        let mut file_mutations: std::collections::HashMap<&Path, Vec<&MutationResult>> = 
            std::collections::HashMap::new();
        
        for result in &summary.mutation_results {
            file_mutations
                .entry(&result.file_path)
                .or_default()
                .push(result);
        }
        
        // Add file nodes
        if self.options.include_files {
            let mut files_node = FormattedNode::new(
                format!("Files ({})", file_mutations.len()),
                NodeType::Directory,
            );
            
            for (path, mutations) in file_mutations {
                let survived = mutations.iter().filter(|m| m.survived).count();
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<unknown>");
                
                let score = if mutations.is_empty() {
                    100.0
                } else {
                    ((mutations.len() - survived) as f64 / mutations.len() as f64) * 100.0
                };
                
                let mut file_node = FormattedNode::new(file_name.to_string(), NodeType::File)
                    .with_coverage(score)
                    .with_value(format!(
                        "{} ({}/{} killed)",
                        format_coverage(score, self.options.use_color),
                        mutations.len() - survived,
                        mutations.len()
                    ));
                
                file_node.metadata.mutations = Some(mutations.len());
                file_node.metadata.survived_mutations = Some(survived);
                
                // Add survived mutations as children if not in compact mode
                if !self.options.compact && survived > 0 {
                    for mutation in mutations.iter().filter(|m| m.survived) {
                        let mut_node = FormattedNode::new(
                            format!(
                                "Line {}: {} → {}",
                                mutation.line_number,
                                mutation.original_code,
                                mutation.mutated_code
                            ),
                            NodeType::Line,
                        );
                        file_node.add_child(mut_node);
                    }
                }
                
                files_node.add_child(file_node);
            }
            
            if self.options.sort_by_coverage {
                files_node.sort_by_coverage();
            }
            
            root.add_child(files_node);
        }
        
        root
    }
}

impl OutputFormatter for TreeFormatter {
    fn format_coverage(&self, summary: &CoverageSummary) -> Result<String> {
        let tree = self.build_coverage_tree(summary);
        let output = self.format_tree(&tree);
        
        // Add coverage bar visualization
        let mut result = output;
        if !self.options.compact {
            result.push_str("\n\n");
            result.push_str(&bold("Coverage Overview:", self.options.use_color));
            result.push_str("\n");
            result.push_str(&format!(
                "  Line:     {} {:.1}%\n",
                format_coverage_bar(summary.line_coverage_percent, 20, self.options.use_color),
                summary.line_coverage_percent
            ));
            result.push_str(&format!(
                "  Branch:   {} {:.1}%\n",
                format_coverage_bar(summary.branch_coverage_percent, 20, self.options.use_color),
                summary.branch_coverage_percent
            ));
            result.push_str(&format!(
                "  Function: {} {:.1}%\n",
                format_coverage_bar(summary.function_coverage_percent, 20, self.options.use_color),
                summary.function_coverage_percent
            ));
        }
        
        Ok(result)
    }
    
    fn format_mutations(&self, summary: &MutationSummary) -> Result<String> {
        let tree = self.build_mutation_tree(summary);
        let output = self.format_tree(&tree);
        
        // Add mutation score bar
        let mut result = output;
        if !self.options.compact {
            result.push_str("\n\n");
            result.push_str(&bold("Mutation Score:", self.options.use_color));
            result.push_str("\n  ");
            result.push_str(&format_coverage_bar(summary.mutation_score, 30, self.options.use_color));
            result.push_str(&format!(" {:.1}%\n", summary.mutation_score));
        }
        
        Ok(result)
    }
    
    fn format_combined(&self, report: &CombinedReport) -> Result<String> {
        let mut output = String::new();
        
        // Header
        output.push_str(&bold(&format!("📊 {} - Quality Report", report.project_name), self.options.use_color));
        output.push_str("\n");
        output.push_str(&dim(&format!("Generated at: {}", 
            chrono::DateTime::<chrono::Utc>::from_timestamp(report.timestamp as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "Unknown".to_string())
        ), self.options.use_color));
        output.push_str("\n\n");
        
        // Overall quality score
        output.push_str(&bold("Overall Quality Score:", self.options.use_color));
        output.push_str("\n  ");
        output.push_str(&format_coverage_bar(report.quality_score, 30, self.options.use_color));
        output.push_str(&format!(" {:.1}%\n\n", report.quality_score));
        
        // Coverage section
        if let Some(ref coverage) = report.coverage {
            output.push_str(&self.format_coverage(coverage)?);
            output.push_str("\n\n");
        }
        
        // Mutation section
        if let Some(ref mutations) = report.mutations {
            output.push_str(&self.format_mutations(mutations)?);
        }
        
        Ok(output)
    }
    
    fn format_file_coverage(&self, file: &FileCoverage) -> Result<String> {
        let node = self.build_file_node(file);
        Ok(self.format_tree(&node))
    }
    
    fn format_type(&self) -> OutputFormat {
        OutputFormat::Tree
    }
    
    fn options(&self) -> &FormatterOptions {
        &self.options
    }
}