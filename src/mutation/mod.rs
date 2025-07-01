//! Mutation testing implementation

use crate::ast::{ElementKind, InstrumentableElement};
use crate::config::MutationOperator;
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{BinOp, Expr, UnOp};

pub mod analyzer;
pub mod generator;
pub mod runner;

/// Represents a single mutation that can be applied to code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mutation {
    /// Unique identifier for this mutation
    pub id: String,

    /// The operator that created this mutation
    pub operator: MutationOperator,

    /// The element being mutated
    #[serde(skip)]
    pub element: InstrumentableElement,

    /// Original code (as string)
    pub original_code: String,

    /// Mutated code (as string)
    pub mutated_code: String,

    /// Token stream for the mutation
    #[serde(skip)]
    pub mutation_tokens: TokenStream,

    /// Description of what was changed
    pub description: String,
}

/// Result of running a single mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    /// The mutation that was tested
    pub mutation: Mutation,

    /// Whether the mutation was killed (caught by tests)
    pub killed: bool,

    /// Which tests killed the mutation (if any)
    pub killing_tests: Vec<String>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Whether the mutation caused a timeout
    pub timed_out: bool,

    /// Whether the mutation caused a compilation error
    pub compile_error: bool,

    /// Error message if applicable
    pub error_message: Option<String>,
    
    // Additional fields for output formatting
    /// Whether the mutation survived (opposite of killed)
    pub survived: bool,
    
    /// File path where the mutation was applied
    pub file_path: std::path::PathBuf,
    
    /// Line number where the mutation was applied
    pub line_number: usize,
    
    /// Original code before mutation
    pub original_code: String,
    
    /// Mutated code
    pub mutated_code: String,
}

/// Summary of mutation testing results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationSummary {
    /// Total number of mutations generated
    pub total_mutations: usize,

    /// Number of mutations killed by tests
    pub killed: usize,

    /// Number of mutations that survived (not caught)
    pub survived: usize,

    /// Number of mutations that timed out
    pub timed_out: usize,

    /// Number of mutations that caused compilation errors
    pub compile_errors: usize,

    /// Mutation score (killed / (total - compile_errors)) * 100
    pub mutation_score: f64,

    /// Results for each mutation
    #[serde(skip)]
    pub results: Vec<MutationResult>,
    
    // Convenience aliases for compatibility
    /// Alias for killed
    pub mutations_killed: usize,
    
    /// Alias for survived
    pub mutations_survived: usize,
    
    /// Alias for timed_out
    pub mutations_timeout: usize,
    
    /// Detailed mutation results
    pub mutation_results: Vec<MutationResult>,
}

/// Generator for creating mutations
pub struct MutationGenerator {
    /// Enabled mutation operators
    operators: Vec<MutationOperator>,

    /// Random number generator for selection
    rng: rand::rngs::StdRng,
}

impl MutationGenerator {
    /// Create a new mutation generator
    pub fn new(operators: Vec<MutationOperator>, seed: Option<u64>) -> Self {
        use rand::SeedableRng;

        let rng = if let Some(seed) = seed {
            rand::rngs::StdRng::seed_from_u64(seed)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        Self { operators, rng }
    }

    /// Generate mutations for an element
    pub fn generate_mutations(
        &mut self,
        element: &InstrumentableElement,
        expr: &Expr,
    ) -> Vec<Mutation> {
        let mut mutations = Vec::new();

        for operator in &self.operators {
            if let Some(mutation) = self.apply_operator(operator, element, expr) {
                mutations.push(mutation);
            }
        }

        mutations
    }

    /// Apply a specific mutation operator
    fn apply_operator(
        &self,
        operator: &MutationOperator,
        element: &InstrumentableElement,
        expr: &Expr,
    ) -> Option<Mutation> {
        match operator {
            MutationOperator::ArithmeticOperatorReplacement => {
                self.mutate_arithmetic_op(element, expr)
            }
            MutationOperator::ComparisonOperatorReplacement => {
                self.mutate_comparison_op(element, expr)
            }
            MutationOperator::LogicalOperatorReplacement => self.mutate_logical_op(element, expr),
            MutationOperator::ConstantReplacement => self.mutate_constant(element, expr),
            MutationOperator::ReturnValueReplacement => self.mutate_return_value(element, expr),
            _ => None, // TODO: Implement other operators
        }
    }

    /// Mutate arithmetic operators
    fn mutate_arithmetic_op(
        &self,
        element: &InstrumentableElement,
        expr: &Expr,
    ) -> Option<Mutation> {
        if let Expr::Binary(bin_expr) = expr {
            let replacement = match &bin_expr.op {
                BinOp::Add(_) => Some((quote! { - }, "Changed + to -")),
                BinOp::Sub(_) => Some((quote! { + }, "Changed - to +")),
                BinOp::Mul(_) => Some((quote! { / }, "Changed * to /")),
                BinOp::Div(_) => Some((quote! { * }, "Changed / to *")),
                BinOp::Rem(_) => Some((quote! { * }, "Changed % to *")),
                _ => None,
            };

            if let Some((new_op, description)) = replacement {
                let left = &bin_expr.left;
                let right = &bin_expr.right;
                let mutation_tokens = quote! { #left #new_op #right };

                return Some(Mutation {
                    id: format!("mut_arith_{}", element.id),
                    operator: MutationOperator::ArithmeticOperatorReplacement,
                    element: element.clone(),
                    original_code: quote! { #expr }.to_string(),
                    mutated_code: mutation_tokens.to_string(),
                    mutation_tokens,
                    description: description.to_string(),
                });
            }
        }
        None
    }

    /// Mutate comparison operators
    fn mutate_comparison_op(
        &self,
        element: &InstrumentableElement,
        expr: &Expr,
    ) -> Option<Mutation> {
        if let Expr::Binary(bin_expr) = expr {
            let replacement = match &bin_expr.op {
                BinOp::Lt(_) => Some((quote! { > }, "Changed < to >")),
                BinOp::Gt(_) => Some((quote! { < }, "Changed > to <")),
                BinOp::Le(_) => Some((quote! { >= }, "Changed <= to >=")),
                BinOp::Ge(_) => Some((quote! { <= }, "Changed >= to <=")),
                BinOp::Eq(_) => Some((quote! { != }, "Changed == to !=")),
                BinOp::Ne(_) => Some((quote! { == }, "Changed != to ==")),
                _ => None,
            };

            if let Some((new_op, description)) = replacement {
                let left = &bin_expr.left;
                let right = &bin_expr.right;
                let mutation_tokens = quote! { #left #new_op #right };

                return Some(Mutation {
                    id: format!("mut_comp_{}", element.id),
                    operator: MutationOperator::ComparisonOperatorReplacement,
                    element: element.clone(),
                    original_code: quote! { #expr }.to_string(),
                    mutated_code: mutation_tokens.to_string(),
                    mutation_tokens,
                    description: description.to_string(),
                });
            }
        }
        None
    }

    /// Mutate logical operators
    fn mutate_logical_op(&self, element: &InstrumentableElement, expr: &Expr) -> Option<Mutation> {
        match expr {
            Expr::Binary(bin_expr) => {
                let replacement = match &bin_expr.op {
                    BinOp::And(_) => Some((quote! { || }, "Changed && to ||")),
                    BinOp::Or(_) => Some((quote! { && }, "Changed || to &&")),
                    _ => None,
                };

                if let Some((new_op, description)) = replacement {
                    let left = &bin_expr.left;
                    let right = &bin_expr.right;
                    let mutation_tokens = quote! { #left #new_op #right };

                    return Some(Mutation {
                        id: format!("mut_logic_{}", element.id),
                        operator: MutationOperator::LogicalOperatorReplacement,
                        element: element.clone(),
                        original_code: quote! { #expr }.to_string(),
                        mutated_code: mutation_tokens.to_string(),
                        mutation_tokens,
                        description: description.to_string(),
                    });
                }
            }
            Expr::Unary(un_expr) if matches!(un_expr.op, UnOp::Not(_)) => {
                let inner = &un_expr.expr;
                let mutation_tokens = quote! { #inner };

                return Some(Mutation {
                    id: format!("mut_not_{}", element.id),
                    operator: MutationOperator::LogicalOperatorReplacement,
                    element: element.clone(),
                    original_code: quote! { #expr }.to_string(),
                    mutated_code: mutation_tokens.to_string(),
                    mutation_tokens,
                    description: "Removed logical NOT operator".to_string(),
                });
            }
            _ => {}
        }
        None
    }

    /// Mutate constant values
    fn mutate_constant(&self, element: &InstrumentableElement, expr: &Expr) -> Option<Mutation> {
        match expr {
            Expr::Lit(lit_expr) => {
                use syn::Lit;
                let (mutation_tokens, description) = match &lit_expr.lit {
                    Lit::Int(int_lit) => {
                        let value: i64 = int_lit.base10_parse().ok()?;
                        let new_value = if value == 0 { 1 } else { 0 };
                        (
                            quote! { #new_value },
                            format!("Changed {} to {}", value, new_value),
                        )
                    }
                    Lit::Bool(bool_lit) => {
                        let new_value = !bool_lit.value;
                        (
                            quote! { #new_value },
                            format!("Changed {} to {}", bool_lit.value, new_value),
                        )
                    }
                    Lit::Str(str_lit) => {
                        let value = &str_lit.value();
                        let new_value = if value.is_empty() { "mutated" } else { "" };
                        (
                            quote! { #new_value },
                            format!("Changed string \"{}\" to \"{}\"", value, new_value),
                        )
                    }
                    _ => return None,
                };

                Some(Mutation {
                    id: format!("mut_const_{}", element.id),
                    operator: MutationOperator::ConstantReplacement,
                    element: element.clone(),
                    original_code: quote! { #expr }.to_string(),
                    mutated_code: mutation_tokens.to_string(),
                    mutation_tokens,
                    description,
                })
            }
            _ => None,
        }
    }

    /// Mutate return values
    fn mutate_return_value(
        &self,
        element: &InstrumentableElement,
        expr: &Expr,
    ) -> Option<Mutation> {
        if element.kind == ElementKind::Return {
            let mutation_tokens = match expr {
                Expr::Lit(lit_expr) => {
                    use syn::Lit;
                    match &lit_expr.lit {
                        Lit::Bool(bool_lit) => {
                            let new_value = !bool_lit.value;
                            quote! { #new_value }
                        }
                        Lit::Int(_) => quote! { 0 },
                        Lit::Str(_) => quote! { "" },
                        _ => return None,
                    }
                }
                Expr::Tuple(tuple_expr) if tuple_expr.elems.is_empty() => {
                    // Unit return - can't mutate
                    return None;
                }
                _ => {
                    // For Option/Result returns, try to return None/Err
                    quote! { None }
                }
            };

            Some(Mutation {
                id: format!("mut_return_{}", element.id),
                operator: MutationOperator::ReturnValueReplacement,
                element: element.clone(),
                original_code: quote! { #expr }.to_string(),
                mutated_code: mutation_tokens.to_string(),
                mutation_tokens,
                description: "Mutated return value".to_string(),
            })
        } else {
            None
        }
    }
}

impl MutationSummary {
    /// Create a new summary from results
    pub fn from_results(results: Vec<MutationResult>) -> Self {
        let total_mutations = results.len();
        let killed = results.iter().filter(|r| r.killed).count();
        let survived = results
            .iter()
            .filter(|r| !r.killed && !r.compile_error && !r.timed_out)
            .count();
        let timed_out = results.iter().filter(|r| r.timed_out).count();
        let compile_errors = results.iter().filter(|r| r.compile_error).count();

        let valid_mutations = total_mutations - compile_errors;
        let mutation_score = if valid_mutations > 0 {
            (killed as f64 / valid_mutations as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_mutations,
            killed,
            survived,
            timed_out,
            compile_errors,
            mutation_score,
            results: results.clone(),
            // Aliases for compatibility
            mutations_killed: killed,
            mutations_survived: survived,
            mutations_timeout: timed_out,
            mutation_results: results,
        }
    }
}
