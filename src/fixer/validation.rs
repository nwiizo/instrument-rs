//! Syntax validation for modified source files
//!
//! This module provides validation to ensure that modifications to source
//! files result in syntactically valid Rust code.

use std::fmt;

/// Validation error types
#[derive(Debug)]
pub enum ValidationError {
    /// Syntax error in the modified source
    SyntaxError {
        /// Error message from the parser
        message: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SyntaxError { message } => {
                write!(f, "Syntax error: {}", message)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate that the modified source still parses correctly
///
/// This function uses `syn` to parse the source code and returns
/// an error if the syntax is invalid.
pub fn validate_syntax(source: &str) -> Result<(), ValidationError> {
    syn::parse_file(source)
        .map(|_| ())
        .map_err(|e| ValidationError::SyntaxError {
            message: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_syntax() {
        let source = r#"
fn foo() {
    println!("Hello");
}
"#;
        assert!(validate_syntax(source).is_ok());
    }

    #[test]
    fn test_validate_with_instrument() {
        let source = r#"
use tracing::instrument;

#[instrument]
fn foo() {
    println!("Hello");
}
"#;
        assert!(validate_syntax(source).is_ok());
    }

    #[test]
    fn test_validate_invalid_syntax() {
        let source = r#"
fn foo( {
    // Missing closing paren
}
"#;
        assert!(validate_syntax(source).is_err());
    }

    #[test]
    fn test_validate_incomplete_attribute() {
        let source = r#"
#[instrument(
fn foo() {}
"#;
        assert!(validate_syntax(source).is_err());
    }
}
