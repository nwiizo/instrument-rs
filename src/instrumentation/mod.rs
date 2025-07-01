//! Code instrumentation strategies and implementations

use crate::ast::InstrumentableElement;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, File};

pub mod coverage;
pub mod mutation;
pub mod transform;

/// Trait for instrumentation strategies
pub trait InstrumentationStrategy {
    /// Get the name of this strategy
    fn name(&self) -> &str;

    /// Initialize the strategy with runtime components
    fn initialize(&mut self) -> crate::Result<TokenStream>;

    /// Instrument a specific element
    fn instrument_element(&self, element: &InstrumentableElement, expr: &Expr) -> TokenStream;

    /// Finalize instrumentation with cleanup code
    fn finalize(&self) -> TokenStream;

    /// Get the runtime dependencies needed
    fn runtime_dependencies(&self) -> Vec<&str>;
}

/// Coverage instrumentation context
#[derive(Debug, Clone)]
pub struct InstrumentationContext {
    /// Unique ID for this instrumentation session
    pub session_id: String,

    /// Counter for generating unique probe IDs
    pub probe_counter: usize,

    /// Map of element IDs to probe IDs
    pub probe_map: std::collections::HashMap<String, usize>,

    /// Runtime initialization code
    pub init_code: TokenStream,

    /// Runtime finalization code  
    pub finalize_code: TokenStream,
}

/// A probe point for instrumentation
#[derive(Debug, Clone)]
pub struct Probe {
    /// Unique ID for this probe
    pub id: usize,

    /// Element being probed
    pub element_id: String,

    /// Type of probe
    pub kind: ProbeKind,

    /// Generated instrumentation code
    pub code: TokenStream,
}

/// Types of instrumentation probes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeKind {
    /// Function entry probe
    FunctionEntry,
    /// Function exit probe
    FunctionExit,
    /// Branch/condition probe
    Branch,
    /// Statement execution probe
    Statement,
    /// Loop iteration probe
    LoopIteration,
}

impl InstrumentationContext {
    /// Create a new instrumentation context
    pub fn new() -> Self {
        let session_id = Self::generate_session_id();

        Self {
            session_id,
            probe_counter: 0,
            probe_map: std::collections::HashMap::new(),
            init_code: quote! {},
            finalize_code: quote! {},
        }
    }

    /// Generate a unique session ID
    fn generate_session_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("session_{}", timestamp)
    }

    /// Allocate a new probe ID
    pub fn next_probe_id(&mut self) -> usize {
        let id = self.probe_counter;
        self.probe_counter += 1;
        id
    }

    /// Register a probe for an element
    pub fn register_probe(&mut self, element_id: String, probe_id: usize) {
        self.probe_map.insert(element_id, probe_id);
    }
}

/// Result of instrumenting a file
#[derive(Debug)]
pub struct InstrumentedFile {
    /// The instrumented AST
    pub ast: File,

    /// Probes that were inserted
    pub probes: Vec<Probe>,

    /// Metadata about the instrumentation
    pub metadata: InstrumentationMetadata,
}

/// Metadata about instrumentation
#[derive(Debug, Clone)]
pub struct InstrumentationMetadata {
    /// Session ID
    pub session_id: String,

    /// Original file hash
    pub source_hash: String,

    /// Number of probes inserted
    pub probe_count: usize,

    /// Instrumentation timestamp
    pub timestamp: u64,

    /// Strategy used
    pub strategy: String,
}

/// Common instrumentation utilities
pub mod utils {
    use super::*;
    use quote::quote;

    /// Generate a probe registration call
    pub fn probe_registration(probe_id: usize, element_type: &str) -> TokenStream {
        quote! {
            instrument_rs::runtime::register_probe(#probe_id, #element_type);
        }
    }

    /// Generate a probe hit recording call
    pub fn probe_hit(probe_id: usize) -> TokenStream {
        quote! {
            instrument_rs::runtime::record_hit(#probe_id);
        }
    }

    /// Generate branch coverage recording
    pub fn branch_coverage(probe_id: usize, condition: &TokenStream) -> TokenStream {
        quote! {
            {
                let __condition = #condition;
                instrument_rs::runtime::record_branch(#probe_id, __condition);
                __condition
            }
        }
    }

    /// Wrap a statement with coverage recording
    pub fn wrap_statement(probe_id: usize, stmt: &TokenStream) -> TokenStream {
        quote! {
            {
                instrument_rs::runtime::record_hit(#probe_id);
                #stmt
            }
        }
    }

    /// Generate function entry recording
    pub fn function_entry(probe_id: usize, fn_name: &str) -> TokenStream {
        quote! {
            instrument_rs::runtime::enter_function(#probe_id, #fn_name);
        }
    }

    /// Generate function exit recording
    pub fn function_exit(probe_id: usize) -> TokenStream {
        quote! {
            instrument_rs::runtime::exit_function(#probe_id);
        }
    }
}
