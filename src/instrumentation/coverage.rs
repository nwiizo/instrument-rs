//! Coverage instrumentation strategy

use crate::ast::InstrumentableElement;
use crate::instrumentation::{InstrumentationContext, InstrumentationStrategy};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// Coverage instrumentation strategy
pub struct CoverageStrategy {
    context: InstrumentationContext,
}

impl CoverageStrategy {
    /// Create a new coverage strategy
    pub fn new() -> Self {
        Self {
            context: InstrumentationContext::new(),
        }
    }
}

impl InstrumentationStrategy for CoverageStrategy {
    fn name(&self) -> &str {
        "coverage"
    }

    fn initialize(&mut self) -> crate::Result<TokenStream> {
        Ok(quote! {
            // TODO: Initialize coverage runtime
        })
    }

    fn instrument_element(&self, element: &InstrumentableElement, expr: &Expr) -> TokenStream {
        // TODO: Implement element instrumentation
        quote! { #expr }
    }

    fn finalize(&self) -> TokenStream {
        quote! {
            // TODO: Finalize coverage collection
        }
    }

    fn runtime_dependencies(&self) -> Vec<&str> {
        vec![]
    }
}
