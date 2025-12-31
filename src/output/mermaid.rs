//! Mermaid diagram output formatter

use super::traits::{FormatterOptions, OutputFormat, OutputFormatter};
use crate::AnalysisResult;
use crate::Result;

/// Mermaid diagram formatter for analysis results
pub struct MermaidFormatter {
    options: FormatterOptions,
}

impl MermaidFormatter {
    /// Create a new Mermaid formatter
    pub fn new(options: FormatterOptions) -> Self {
        Self { options }
    }
}

impl OutputFormatter for MermaidFormatter {
    fn format(&self, result: &AnalysisResult) -> Result<String> {
        let mut output = String::new();
        output.push_str("graph TD\n");

        // Add endpoints as entry points
        for (i, endpoint) in result.endpoints.iter().enumerate() {
            let node_id = format!("EP{i}");
            output.push_str(&format!(
                "    {node_id}[\"{} {}\"]\n",
                endpoint.method, endpoint.path
            ));

            // Connect to handler
            let handler_id = sanitize_id(&endpoint.handler);
            output.push_str(&format!("    {node_id} --> {handler_id}\n"));
        }

        // Add call graph edges
        for edge in result.call_graph.edges() {
            let from_id = sanitize_id(&edge.from);
            let to_id = sanitize_id(&edge.to);
            output.push_str(&format!("    {from_id} --> {to_id}\n"));
        }

        // Add instrumentation points with styles
        for point in &result.points {
            let node_id = sanitize_id(&point.location.function_name);
            let style_class = match point.priority {
                crate::detector::Priority::Critical => "critical",
                crate::detector::Priority::High => "high",
                crate::detector::Priority::Medium => "medium",
                crate::detector::Priority::Low => "low",
            };
            output.push_str(&format!("    class {node_id} {style_class}\n"));
        }

        // Add style definitions
        output.push_str("\n    classDef critical fill:#ff0000,stroke:#333,stroke-width:2px\n");
        output.push_str("    classDef high fill:#ff6600,stroke:#333,stroke-width:2px\n");
        output.push_str("    classDef medium fill:#ffcc00,stroke:#333,stroke-width:1px\n");
        output.push_str("    classDef low fill:#99cc99,stroke:#333,stroke-width:1px\n");

        Ok(output)
    }

    fn format_type(&self) -> OutputFormat {
        OutputFormat::Mermaid
    }
}

fn sanitize_id(id: &str) -> String {
    id.replace("::", "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace(' ', "_")
}
