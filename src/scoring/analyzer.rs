//! AST-based analysis for instrumentation scoring
//!
//! This module provides functionality to analyze Rust AST nodes and calculate
//! instrumentation scores based on code structure and patterns.

use crate::scoring::instrumentation::{
    InstrumentationPriority, InstrumentationScore, InstrumentationScorer,
};
use std::collections::HashMap;
use syn::{
    visit::{self, Visit},
    Expr, ExprCall, ExprMethodCall, File, ItemFn, ReturnType, Stmt, Type,
};

/// Metadata collected about a function for scoring
#[derive(Debug, Default, Clone)]
pub struct FunctionMetadata {
    /// Function name
    pub name: String,
    /// Whether the function is public
    pub is_public: bool,
    /// Whether the function is async
    pub is_async: bool,
    /// Cyclomatic complexity
    pub complexity: u32,
    /// Whether the function has error handling
    pub has_error_handling: bool,
    /// Number of external calls detected
    pub external_call_count: usize,
    /// Whether the function returns a Result
    pub returns_result: bool,
    /// Number of match expressions (for pattern matching)
    pub match_count: usize,
    /// Number of if expressions
    pub if_count: usize,
    /// Line count
    pub line_count: usize,
    /// List of external calls found
    pub external_calls: Vec<String>,
}

/// AST visitor that collects function metadata for scoring
pub struct FunctionAnalyzer {
    /// Current function being analyzed
    current_function: Option<String>,
    /// Metadata for all functions
    pub functions: HashMap<String, FunctionMetadata>,
    /// Patterns that indicate external calls
    external_patterns: Vec<String>,
}

impl FunctionAnalyzer {
    /// Create a new function analyzer
    pub fn new() -> Self {
        Self {
            current_function: None,
            functions: HashMap::new(),
            external_patterns: vec![
                "client".to_string(),
                "request".to_string(),
                "fetch".to_string(),
                "query".to_string(),
                "execute".to_string(),
                "send".to_string(),
                "post".to_string(),
                "get".to_string(),
                "put".to_string(),
                "delete".to_string(),
                "connect".to_string(),
                "http".to_string(),
                "grpc".to_string(),
                "rpc".to_string(),
            ],
        }
    }

    /// Analyze a file and return function metadata
    pub fn analyze_file(&mut self, file: &File) {
        self.visit_file(file);
    }

    /// Check if a function call is likely external
    fn is_external_call(&self, call_name: &str) -> bool {
        let call_lower = call_name.to_lowercase();
        self.external_patterns
            .iter()
            .any(|pattern| call_lower.contains(pattern))
    }

    /// Get the current function's metadata
    fn current_metadata(&mut self) -> Option<&mut FunctionMetadata> {
        self.current_function
            .as_ref()
            .and_then(|name| self.functions.get_mut(name))
    }

    /// Calculate cyclomatic complexity for the current function
    fn increment_complexity(&mut self) {
        if let Some(metadata) = self.current_metadata() {
            metadata.complexity += 1;
        }
    }
}

impl<'ast> Visit<'ast> for FunctionAnalyzer {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        let func_name = func.sig.ident.to_string();
        
        // Initialize metadata for this function
        let metadata = FunctionMetadata {
            name: func_name.clone(),
            is_public: matches!(func.vis, syn::Visibility::Public(_)),
            is_async: func.sig.asyncness.is_some(),
            returns_result: matches!(&func.sig.output, ReturnType::Type(_, ty) if is_result_type(ty)),
            line_count: estimate_line_count(&func.block),
            ..Default::default()
        };

        self.functions.insert(func_name.clone(), metadata);
        self.current_function = Some(func_name);

        // Visit the function body
        visit::visit_item_fn(self, func);

        // Clear current function
        self.current_function = None;
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            // Count if expressions for complexity
            Expr::If(_) => {
                self.increment_complexity();
                if let Some(metadata) = self.current_metadata() {
                    metadata.if_count += 1;
                }
            }
            // Count match expressions
            Expr::Match(_) => {
                self.increment_complexity();
                if let Some(metadata) = self.current_metadata() {
                    metadata.match_count += 1;
                }
            }
            // Count loops
            Expr::Loop(_) | Expr::While(_) | Expr::ForLoop(_) => {
                self.increment_complexity();
            }
            // Detect try expressions (? operator)
            Expr::Try(_) => {
                if let Some(metadata) = self.current_metadata() {
                    metadata.has_error_handling = true;
                }
            }
            // Detect function calls
            Expr::Call(ExprCall { func, .. }) => {
                if let Some(call_name) = extract_call_name(func) {
                    if self.is_external_call(&call_name) {
                        if let Some(metadata) = self.current_metadata() {
                            metadata.external_call_count += 1;
                            metadata.external_calls.push(call_name);
                        }
                    }
                }
            }
            // Detect method calls
            Expr::MethodCall(ExprMethodCall { method, .. }) => {
                let method_name = method.to_string();
                if self.is_external_call(&method_name) {
                    if let Some(metadata) = self.current_metadata() {
                        metadata.external_call_count += 1;
                        metadata.external_calls.push(method_name);
                    }
                }
            }
            _ => {}
        }

        // Continue visiting
        visit::visit_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        // Look for error handling patterns in statements
        if let Stmt::Expr(Expr::Match(match_expr), _) = stmt {
            // Check if this is a Result match
            if let Expr::Try(_) = &*match_expr.expr {
                if let Some(metadata) = self.current_metadata() {
                    metadata.has_error_handling = true;
                }
            }
        }

        visit::visit_stmt(self, stmt);
    }
}

/// Check if a type is a Result type
fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result";
        }
    }
    false
}

/// Extract function name from an expression
fn extract_call_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(expr_path) => {
            expr_path.path.segments.last().map(|s| s.ident.to_string())
        }
        Expr::Field(field_expr) => Some(format!("{:?}", field_expr.member)),
        _ => None,
    }
}

/// Estimate line count for a block (rough approximation)
fn estimate_line_count(block: &syn::Block) -> usize {
    // This is a rough estimate - in real implementation, 
    // we'd use span information
    block.stmts.len().max(1) * 3
}

/// Analyze functions in a file and score them for instrumentation
pub struct InstrumentationAnalyzer {
    scorer: InstrumentationScorer,
    analyzer: FunctionAnalyzer,
}

impl InstrumentationAnalyzer {
    /// Create a new instrumentation analyzer
    pub fn new() -> Self {
        Self {
            scorer: InstrumentationScorer::new(),
            analyzer: FunctionAnalyzer::new(),
        }
    }

    /// Analyze a file and return scored functions
    pub fn analyze_and_score(&mut self, file: &File) -> Vec<(String, InstrumentationScore)> {
        // First, analyze the file to collect metadata
        self.analyzer.analyze_file(file);

        // Then score each function
        let mut scores = Vec::new();
        for (name, metadata) in &self.analyzer.functions {
            let score = self.scorer.score_function(
                &metadata.name,
                metadata.complexity,
                metadata.has_error_handling || metadata.returns_result,
                metadata.external_call_count,
                metadata.is_public,
            );
            scores.push((name.clone(), score));
        }

        // Sort by priority (highest first)
        scores.sort_by(|a, b| b.1.overall_score.partial_cmp(&a.1.overall_score).unwrap());

        scores
    }

    /// Get functions that meet a minimum priority threshold
    pub fn get_high_priority_functions(
        &mut self,
        file: &File,
        min_priority: InstrumentationPriority,
    ) -> Vec<String> {
        let scores = self.analyze_and_score(file);
        scores
            .into_iter()
            .filter(|(_, score)| score.priority >= min_priority)
            .map(|(name, _)| name)
            .collect()
    }

    /// Generate a detailed report of the analysis
    pub fn generate_report(&mut self, file: &File) -> InstrumentationReport {
        let scores = self.analyze_and_score(file);
        let metadata = self.analyzer.functions.clone();
        let summary = self.generate_summary(&scores);

        InstrumentationReport {
            total_functions: scores.len(),
            scored_functions: scores,
            metadata,
            summary,
        }
    }

    /// Generate a summary of the instrumentation analysis
    fn generate_summary(&self, scores: &[(String, InstrumentationScore)]) -> InstrumentationSummary {
        let mut summary = InstrumentationSummary::default();

        for (_, score) in scores {
            match score.priority {
                InstrumentationPriority::Critical => summary.critical_count += 1,
                InstrumentationPriority::High => summary.high_count += 1,
                InstrumentationPriority::Medium => summary.medium_count += 1,
                InstrumentationPriority::Low => summary.low_count += 1,
                InstrumentationPriority::Minimal => summary.minimal_count += 1,
            }
        }

        summary.average_score = if !scores.is_empty() {
            scores.iter().map(|(_, s)| s.overall_score).sum::<f64>() / scores.len() as f64
        } else {
            0.0
        };

        summary
    }
}

/// Report generated by instrumentation analysis
#[derive(Debug)]
pub struct InstrumentationReport {
    /// Total number of functions analyzed
    pub total_functions: usize,
    /// Functions with their scores
    pub scored_functions: Vec<(String, InstrumentationScore)>,
    /// Raw metadata for each function
    pub metadata: HashMap<String, FunctionMetadata>,
    /// Summary statistics
    pub summary: InstrumentationSummary,
}

/// Summary of instrumentation analysis
#[derive(Debug, Default)]
pub struct InstrumentationSummary {
    /// Number of critical priority functions
    pub critical_count: usize,
    /// Number of high priority functions
    pub high_count: usize,
    /// Number of medium priority functions
    pub medium_count: usize,
    /// Number of low priority functions
    pub low_count: usize,
    /// Number of minimal priority functions
    pub minimal_count: usize,
    /// Average instrumentation score
    pub average_score: f64,
}

impl Default for FunctionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InstrumentationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_analyze_simple_function() {
        let code = r#"
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let file = parse_str::<File>(code).unwrap();
        let mut analyzer = FunctionAnalyzer::new();
        analyzer.analyze_file(&file);

        assert_eq!(analyzer.functions.len(), 1);
        let metadata = analyzer.functions.get("add").unwrap();
        assert!(metadata.is_public);
        assert_eq!(metadata.complexity, 0); // No branches
        assert!(!metadata.has_error_handling);
    }

    #[test]
    fn test_analyze_complex_function() {
        let code = r#"
            async fn process_order(order: Order) -> Result<Receipt, Error> {
                if order.items.is_empty() {
                    return Err(Error::EmptyOrder);
                }

                let payment = match order.payment_method {
                    PaymentMethod::Card => process_card_payment(&order)?,
                    PaymentMethod::Cash => process_cash_payment(&order)?,
                };

                let receipt = database.save_order(&order).await?;
                Ok(receipt)
            }
        "#;

        let file = parse_str::<File>(code).unwrap();
        let mut analyzer = FunctionAnalyzer::new();
        analyzer.analyze_file(&file);

        let metadata = analyzer.functions.get("process_order").unwrap();
        assert!(metadata.is_async);
        assert!(metadata.returns_result);
        assert!(metadata.has_error_handling);
        assert!(metadata.complexity > 0);
        assert!(metadata.external_call_count > 0); // database.save_order
    }

    #[test]
    fn test_scoring_integration() {
        let code = r#"
            pub async fn authenticate_user(credentials: Credentials) -> Result<User, AuthError> {
                let user = database.find_user(&credentials.username).await?;
                
                if !verify_password(&credentials.password, &user.password_hash)? {
                    return Err(AuthError::InvalidCredentials);
                }

                Ok(user)
            }
        "#;

        let file = parse_str::<File>(code).unwrap();
        let mut analyzer = InstrumentationAnalyzer::new();
        let scores = analyzer.analyze_and_score(&file);

        assert_eq!(scores.len(), 1);
        let (name, score) = &scores[0];
        assert_eq!(name, "authenticate_user");
        assert!(score.overall_score > 60.0); // Should be high priority due to auth
        assert_eq!(score.priority, InstrumentationPriority::Critical);
    }
}