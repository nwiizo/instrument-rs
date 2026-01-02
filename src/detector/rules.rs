//! Naming convention and rule checking for instrumentation
//!
//! This module validates existing instrumentation against configurable rules
//! for span naming, required attributes, and forbidden patterns.

use crate::config::NamingRules;
use crate::detector::{
    ExistingInstrumentation, ExistingKind, InstrumentationKind, InstrumentationPoint, Location,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Rule violation found during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    /// Location of the violation
    pub location: Location,
    /// Type of violation
    pub kind: ViolationKind,
    /// Description of what's wrong
    pub message: String,
    /// Suggested fix
    pub suggestion: String,
    /// Severity level
    pub severity: ViolationSeverity,
}

/// Type of rule violation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationKind {
    /// Span name doesn't follow naming convention
    NamingConvention,
    /// Missing required attribute
    MissingAttribute,
    /// Contains forbidden pattern (e.g., sensitive data)
    ForbiddenPattern,
}

/// Severity of rule violation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationSeverity {
    /// Error - must be fixed
    Error,
    /// Warning - should be fixed
    Warning,
    /// Info - consider fixing
    Info,
}

/// Rule checker that validates instrumentation against configured rules
pub struct RuleChecker<'a> {
    rules: &'a NamingRules,
    forbidden_patterns: Vec<Regex>,
}

impl<'a> RuleChecker<'a> {
    /// Create a new rule checker with the given rules
    pub fn new(rules: &'a NamingRules) -> Self {
        let forbidden_patterns = rules
            .forbidden_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self {
            rules,
            forbidden_patterns,
        }
    }

    /// Check all existing instrumentation against rules
    pub fn check_existing(&self, existing: &[ExistingInstrumentation]) -> Vec<RuleViolation> {
        let mut violations = Vec::new();

        for inst in existing {
            violations.extend(self.check_single_existing(inst));
        }

        violations
    }

    /// Check instrumentation points for rule compliance
    pub fn check_points(&self, points: &[InstrumentationPoint]) -> Vec<RuleViolation> {
        let mut violations = Vec::new();

        for point in points {
            violations.extend(self.check_suggested_span(point));
        }

        violations
    }

    fn check_single_existing(&self, inst: &ExistingInstrumentation) -> Vec<RuleViolation> {
        let mut violations = Vec::new();

        // Check span name against naming conventions
        if let Some(ref span_name) = inst.span_name {
            // Check forbidden patterns
            for pattern in &self.forbidden_patterns {
                if pattern.is_match(span_name) {
                    violations.push(RuleViolation {
                        location: inst.location.clone(),
                        kind: ViolationKind::ForbiddenPattern,
                        message: format!(
                            "Span name '{}' matches forbidden pattern '{}'",
                            span_name,
                            pattern.as_str()
                        ),
                        suggestion: "Remove or obfuscate sensitive information from span name"
                            .to_string(),
                        severity: ViolationSeverity::Error,
                    });
                }
            }

            // Check naming convention based on instrumentation type
            if let Some(violation) = self.check_naming_convention(span_name, inst) {
                violations.push(violation);
            }
        }

        violations
    }

    fn check_naming_convention(
        &self,
        span_name: &str,
        inst: &ExistingInstrumentation,
    ) -> Option<RuleViolation> {
        // Determine expected prefix based on context
        // For now, we use the kind from existing instrumentation
        // In future, we could try to infer the kind from the function

        let expected_prefix = match inst.kind {
            ExistingKind::TracingInstrument | ExistingKind::ManualSpan => {
                // Try to detect from span name patterns
                if span_name.contains("db")
                    || span_name.contains("sql")
                    || span_name.contains("query")
                {
                    self.rules.database_prefix.as_deref()
                } else if span_name.contains("cache") || span_name.contains("redis") {
                    self.rules.cache_prefix.as_deref()
                } else if span_name.contains("api")
                    || span_name.contains("http")
                    || span_name.contains("grpc")
                {
                    self.rules.endpoint_prefix.as_deref()
                } else if span_name.contains("external") || span_name.contains("client") {
                    self.rules.external_prefix.as_deref()
                } else {
                    None
                }
            }
            ExistingKind::LogMacro | ExistingKind::Metrics => None,
        };

        if let Some(prefix) = expected_prefix {
            if !span_name.starts_with(prefix) {
                return Some(RuleViolation {
                    location: inst.location.clone(),
                    kind: ViolationKind::NamingConvention,
                    message: format!(
                        "Span name '{}' should start with prefix '{}'",
                        span_name, prefix
                    ),
                    suggestion: format!("Rename span to '{}{}'", prefix, span_name),
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        None
    }

    fn check_suggested_span(&self, point: &InstrumentationPoint) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let span_name = &point.suggested_span_name;

        // Check forbidden patterns in suggested span names
        for pattern in &self.forbidden_patterns {
            if pattern.is_match(span_name) {
                violations.push(RuleViolation {
                    location: point.location.clone(),
                    kind: ViolationKind::ForbiddenPattern,
                    message: format!(
                        "Suggested span name '{}' matches forbidden pattern '{}'",
                        span_name,
                        pattern.as_str()
                    ),
                    suggestion: "Adjust pattern detection to avoid sensitive names".to_string(),
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        // Check naming convention based on point kind
        let expected_prefix = match point.kind {
            InstrumentationKind::Endpoint => self.rules.endpoint_prefix.as_deref(),
            InstrumentationKind::DatabaseCall => self.rules.database_prefix.as_deref(),
            InstrumentationKind::ExternalApiCall => self.rules.external_prefix.as_deref(),
            InstrumentationKind::CacheOperation => self.rules.cache_prefix.as_deref(),
            _ => None,
        };

        if let Some(prefix) = expected_prefix {
            if !span_name.starts_with(prefix) {
                violations.push(RuleViolation {
                    location: point.location.clone(),
                    kind: ViolationKind::NamingConvention,
                    message: format!(
                        "Span name '{}' for {:?} should start with prefix '{}'",
                        span_name, point.kind, prefix
                    ),
                    suggestion: format!("Use '{}{}'", prefix, span_name),
                    severity: ViolationSeverity::Info,
                });
            }
        }

        violations
    }
}

impl ViolationKind {
    /// Get a human-readable name for this violation kind
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::NamingConvention => "Naming Convention",
            Self::MissingAttribute => "Missing Attribute",
            Self::ForbiddenPattern => "Forbidden Pattern",
        }
    }
}

impl ViolationSeverity {
    /// Get a human-readable name for this severity
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_rules() -> NamingRules {
        NamingRules {
            endpoint_prefix: Some("api.".to_string()),
            database_prefix: Some("db.".to_string()),
            external_prefix: Some("ext.".to_string()),
            cache_prefix: Some("cache.".to_string()),
            required_endpoint_attrs: vec!["err".to_string()],
            required_database_attrs: vec!["skip_all".to_string()],
            forbidden_patterns: vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
            ],
        }
    }

    fn create_test_location() -> Location {
        Location {
            file: PathBuf::from("src/test.rs"),
            line: 10,
            column: 1,
            function_name: "test_fn".to_string(),
        }
    }

    #[test]
    fn test_forbidden_pattern_detection() {
        let rules = create_test_rules();
        let checker = RuleChecker::new(&rules);

        let inst = ExistingInstrumentation {
            location: create_test_location(),
            kind: ExistingKind::TracingInstrument,
            span_name: Some("get_user_password".to_string()),
            quality: crate::detector::InstrumentationQuality::default(),
        };

        let violations = checker.check_existing(&[inst]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].kind, ViolationKind::ForbiddenPattern);
    }

    #[test]
    fn test_naming_convention_database() {
        let rules = create_test_rules();
        let checker = RuleChecker::new(&rules);

        let inst = ExistingInstrumentation {
            location: create_test_location(),
            kind: ExistingKind::TracingInstrument,
            span_name: Some("query_users".to_string()), // Contains "query" but doesn't start with "db."
            quality: crate::detector::InstrumentationQuality::default(),
        };

        let violations = checker.check_existing(&[inst]);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].kind, ViolationKind::NamingConvention);
    }

    #[test]
    fn test_valid_naming_convention() {
        let rules = create_test_rules();
        let checker = RuleChecker::new(&rules);

        let inst = ExistingInstrumentation {
            location: create_test_location(),
            kind: ExistingKind::TracingInstrument,
            span_name: Some("db.query_users".to_string()),
            quality: crate::detector::InstrumentationQuality::default(),
        };

        let violations = checker.check_existing(&[inst]);
        assert!(violations.is_empty());
    }
}
