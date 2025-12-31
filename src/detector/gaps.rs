//! Gap analysis for instrumentation coverage
//!
//! Identifies areas of code lacking proper instrumentation.

use super::{
    ExistingInstrumentation, GapSeverity, InstrumentationGap, InstrumentationKind,
    InstrumentationPoint,
};
use crate::call_graph::CallGraph;

/// Analyze gaps in instrumentation coverage
pub fn analyze_gaps(
    points: &[InstrumentationPoint],
    existing: &[ExistingInstrumentation],
    graph: &CallGraph,
) -> Vec<InstrumentationGap> {
    let mut gaps = Vec::new();

    // Find instrumentation points without coverage
    for point in points {
        if !is_covered(point, existing) {
            let severity = determine_gap_severity(point, graph);
            let suggested_fix = generate_suggested_fix(point);

            gaps.push(InstrumentationGap {
                location: point.location.clone(),
                description: format!(
                    "{} '{}' has no instrumentation",
                    point.kind.name(),
                    point.location.function_name
                ),
                suggested_fix,
                severity,
            });
        }
    }

    // Sort by severity (critical first)
    gaps.sort_by_key(|g| match g.severity {
        GapSeverity::Critical => 0,
        GapSeverity::Major => 1,
        GapSeverity::Minor => 2,
    });

    gaps
}

fn is_covered(point: &InstrumentationPoint, existing: &[ExistingInstrumentation]) -> bool {
    existing.iter().any(|e| {
        e.location.file == point.location.file
            && (e.location.line == point.location.line
                || e.location.function_name == point.location.function_name)
    })
}

fn determine_gap_severity(point: &InstrumentationPoint, graph: &CallGraph) -> GapSeverity {
    // Critical if it's an endpoint or external service call
    if matches!(
        point.kind,
        InstrumentationKind::Endpoint | InstrumentationKind::ExternalApiCall
    ) {
        return GapSeverity::Critical;
    }

    // Major if it's on a critical path (high connectivity in call graph)
    if let Some(node) = graph.get_node(&point.location.function_name) {
        let caller_count = node.called_by().len();
        let callee_count = node.calls().len();

        if caller_count > 3 || callee_count > 5 {
            return GapSeverity::Major;
        }
    }

    // Database and business logic are major
    if matches!(
        point.kind,
        InstrumentationKind::DatabaseCall | InstrumentationKind::BusinessLogic
    ) {
        return GapSeverity::Major;
    }

    GapSeverity::Minor
}

fn generate_suggested_fix(point: &InstrumentationPoint) -> String {
    let span_name = &point.suggested_span_name;
    let fields: Vec<_> = point
        .suggested_fields
        .iter()
        .filter(|f| !f.is_sensitive)
        .map(|f| format!("{} = %{}", f.name, f.expression))
        .collect();

    let fields_str = if fields.is_empty() {
        String::new()
    } else {
        format!(", {}", fields.join(", "))
    };

    match point.kind {
        InstrumentationKind::Endpoint => {
            format!(
                r#"#[instrument(name = "{span_name}"{fields_str}, skip_all, err)]
async fn {}(...) {{ ... }}"#,
                point.location.function_name
            )
        }
        InstrumentationKind::DatabaseCall => {
            format!(
                r#"#[instrument(name = "{span_name}"{fields_str}, skip(self), err)]
async fn {}(...) {{ ... }}"#,
                point.location.function_name
            )
        }
        InstrumentationKind::ExternalApiCall => {
            format!(
                r#"#[instrument(name = "{span_name}"{fields_str}, skip(client), err)]
async fn {}(...) {{ ... }}"#,
                point.location.function_name
            )
        }
        _ => {
            format!(
                r#"#[instrument(name = "{span_name}"{fields_str}, err)]
fn {}(...) {{ ... }}"#,
                point.location.function_name
            )
        }
    }
}

/// Calculate coverage percentage
pub fn calculate_coverage(
    points: &[InstrumentationPoint],
    existing: &[ExistingInstrumentation],
) -> f64 {
    if points.is_empty() {
        return 100.0;
    }

    let covered = points.iter().filter(|p| is_covered(p, existing)).count();
    (covered as f64 / points.len() as f64) * 100.0
}
