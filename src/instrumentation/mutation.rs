//! Mutation instrumentation strategy

use crate::ast::InstrumentableElement;
use crate::instrumentation::{InstrumentationContext, InstrumentationStrategy};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// Mutation instrumentation strategy
pub struct MutationStrategy {
    context: InstrumentationContext,
}

impl MutationStrategy {
    /// Create a new mutation strategy
    pub fn new() -> Self {
        Self {
            context: InstrumentationContext::new(),
        }
    }
}

impl InstrumentationStrategy for MutationStrategy {
    fn name(&self) -> &str {
        "mutation"
    }

    fn initialize(&mut self) -> crate::Result<TokenStream> {
        Ok(quote! {
            // TODO: Initialize mutation runtime
        })
    }

    fn instrument_element(&self, element: &InstrumentableElement, expr: &Expr) -> TokenStream {
        // TODO: Implement mutation instrumentation
        quote! { #expr }
    }

    fn finalize(&self) -> TokenStream {
        quote! {
            // TODO: Finalize mutation testing
        }
    }

    fn runtime_dependencies(&self) -> Vec<&str> {
        vec![]
    }
}
