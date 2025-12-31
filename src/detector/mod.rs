//! Instrumentation point detection module
//!
//! This module provides functionality to detect optimal instrumentation points
//! for observability (tracing, logging, metrics) in Rust code.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod endpoint;
pub mod existing;
pub mod gaps;
pub mod priority;

/// Location in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// File path
    pub file: PathBuf,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Function or method name
    pub function_name: String,
}

/// HTTP/gRPC endpoint detected in the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// HTTP method (GET, POST, etc.) or gRPC
    pub method: String,
    /// Route path (e.g., "/api/users/:id")
    pub path: String,
    /// Handler function name
    pub handler: String,
    /// Location in source
    pub location: Location,
    /// Framework that defines this endpoint
    pub framework: String,
}

/// Suggested instrumentation point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationPoint {
    /// Location in source code
    pub location: Location,
    /// Kind of instrumentation needed
    pub kind: InstrumentationKind,
    /// Priority level
    pub priority: Priority,
    /// Reason for suggesting this point
    pub reason: String,
    /// Suggested span name for tracing
    pub suggested_span_name: String,
    /// Suggested fields to capture
    pub suggested_fields: Vec<Field>,
}

/// Field to capture in instrumentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Expression to capture (e.g., "user_id", "request.path")
    pub expression: String,
    /// Whether this field contains sensitive data
    pub is_sensitive: bool,
}

/// Kind of code construct requiring instrumentation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstrumentationKind {
    /// HTTP/gRPC endpoint handler
    Endpoint,
    /// Database query or operation
    DatabaseCall,
    /// External API call (HTTP client, etc.)
    ExternalApiCall,
    /// Cache operation (get, set, invalidate)
    CacheOperation,
    /// Business logic (payment, order processing, etc.)
    BusinessLogic,
    /// Error handling boundary
    ErrorBoundary,
    /// Background job or async task
    BackgroundJob,
    /// Message queue operation
    MessageQueue,
}

/// Priority level for instrumentation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Critical - must have instrumentation
    Critical,
    /// High - strongly recommended
    High,
    /// Medium - recommended
    Medium,
    /// Low - nice to have
    Low,
}

/// Existing instrumentation found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingInstrumentation {
    /// Location in source
    pub location: Location,
    /// Type of existing instrumentation
    pub kind: ExistingKind,
    /// Span name if applicable
    pub span_name: Option<String>,
    /// Quality assessment
    pub quality: InstrumentationQuality,
}

/// Kind of existing instrumentation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExistingKind {
    /// tracing #[instrument] macro
    TracingInstrument,
    /// Manual tracing span
    ManualSpan,
    /// log! macro (info!, warn!, error!, etc.)
    LogMacro,
    /// Metrics recording
    Metrics,
}

/// Quality assessment of existing instrumentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationQuality {
    /// Overall quality score (0.0-1.0)
    pub score: f64,
    /// Issues found
    pub issues: Vec<QualityIssue>,
}

/// Quality issue with existing instrumentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// Issue type
    pub kind: QualityIssueKind,
    /// Description
    pub message: String,
}

/// Kind of quality issue
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityIssueKind {
    /// Missing important fields
    MissingFields,
    /// Generic or unclear span name
    PoorNaming,
    /// No error handling instrumentation
    NoErrorHandling,
    /// Sensitive data not redacted
    SensitiveData,
    /// Missing skip directive for large data
    MissingSkip,
}

/// Gap in instrumentation coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationGap {
    /// Location of the gap
    pub location: Location,
    /// What's missing
    pub description: String,
    /// Suggested fix
    pub suggested_fix: String,
    /// Severity
    pub severity: GapSeverity,
}

/// Severity of instrumentation gap
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GapSeverity {
    /// Critical gap - will impact debugging
    Critical,
    /// Major gap - may impact observability
    Major,
    /// Minor gap - nice to fix
    Minor,
}

impl InstrumentationKind {
    /// Get a human-readable name for this kind
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Endpoint => "HTTP/gRPC Endpoint",
            Self::DatabaseCall => "Database Call",
            Self::ExternalApiCall => "External API Call",
            Self::CacheOperation => "Cache Operation",
            Self::BusinessLogic => "Business Logic",
            Self::ErrorBoundary => "Error Boundary",
            Self::BackgroundJob => "Background Job",
            Self::MessageQueue => "Message Queue",
        }
    }
}

impl Priority {
    /// Get a human-readable name for this priority
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }

    /// Get the numeric score (higher = more important)
    #[must_use]
    pub fn score(&self) -> u8 {
        match self {
            Self::Critical => 4,
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }
}

impl Default for InstrumentationQuality {
    fn default() -> Self {
        Self {
            score: 1.0,
            issues: Vec::new(),
        }
    }
}
