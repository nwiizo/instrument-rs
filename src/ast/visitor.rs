//! AST visitor implementation for traversing Rust syntax trees

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, BinOp, Block, Expr, File, ImplItem, Item, ItemFn, ReturnType, Stmt, Type,
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::ast::{
    AnalysisResult, CallInfo, ComplexityMetrics, ElementKind, ErrorHandlingInfo, FunctionInfo,
    InstrumentableElement, Location, ModuleInfo, SourceFile,
};

/// Context for tracking the current position in the AST
#[derive(Debug, Clone)]
struct VisitorContext {
    /// Current module path
    module_path: Vec<String>,

    /// Current function context (if inside a function)
    current_function: Option<FunctionContext>,

    /// Current nesting depth
    nesting_depth: usize,

    /// Whether we're in test code
    in_test_context: bool,

    /// ID counter for generating unique IDs
    id_counter: usize,
}

/// Context for the currently visited function
#[derive(Debug, Clone)]
struct FunctionContext {
    /// Function info being built
    info: FunctionInfo,

    /// Complexity metrics being tracked
    complexity: ComplexityMetrics,

    /// Error handling info being tracked
    error_handling: ErrorHandlingInfo,

    /// Calls made from this function
    calls: Vec<CallInfo>,
}

/// Main AST visitor for collecting analysis information
pub struct AstVisitor {
    /// Analysis result being built
    result: AnalysisResult,

    /// Current context
    context: VisitorContext,

    /// Source lines for location mapping
    source_lines: Vec<String>,
}

impl AstVisitor {
    /// Create a new AST visitor for the given source file
    pub fn new(source_file: SourceFile) -> Self {
        let source_lines: Vec<String> = source_file.source.lines().map(String::from).collect();

        Self {
            result: AnalysisResult {
                source_file,
                elements: Vec::new(),
                functions: Vec::new(),
                test_functions: Vec::new(),
                modules: Vec::new(),
            },
            context: VisitorContext {
                module_path: Vec::new(),
                current_function: None,
                nesting_depth: 0,
                in_test_context: false,
                id_counter: 0,
            },
            source_lines,
        }
    }

    /// Visit the AST and collect analysis information
    pub fn analyze(mut self) -> AnalysisResult {
        self.visit_file(&self.result.source_file.syntax_tree.clone());
        self.result
    }

    /// Generate a unique ID for an element
    fn generate_id(&mut self, kind: ElementKind) -> String {
        let id = format!("{:?}_{}", kind, self.context.id_counter);
        self.context.id_counter += 1;
        id
    }

    /// Get the current module path as a string
    fn current_module_path(&self) -> String {
        self.context.module_path.join("::")
    }

    /// Create a location from a span using `proc_macro2`'s span locations
    fn location_from_span(&self, span: Span) -> Location {
        // proc_macro2 provides line/column info via start() and end()
        let start = span.start();
        let end = span.end();
        Location::new(start.line, start.column, end.line, end.column)
    }

    /// Check if an item has a test attribute
    fn is_test_item(&self, attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .get_ident()
                .is_some_and(|ident| ident == "test" || ident == "tokio_test")
        })
    }

    /// Check if an item has a cfg(test) attribute
    fn is_cfg_test(&self, attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            if attr.path().get_ident().is_some_and(|i| i == "cfg") {
                // Simple check for cfg(test)
                // Note: This is a simplified check. For production use,
                // we should properly parse the attribute tokens.
                attr.to_token_stream().to_string().contains("test")
            } else {
                false
            }
        })
    }

    /// Extract return type as a string
    fn return_type_to_string(&self, ret: &ReturnType) -> Option<String> {
        match ret {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(self.type_to_string(ty)),
        }
    }

    /// Convert a type to string representation
    fn type_to_string(&self, ty: &Type) -> String {
        // Use quote to get a string representation
        quote! { #ty }.to_string()
    }

    /// Check if a type is Result or Option
    fn is_result_or_option_type(&self, ty: &ReturnType) -> (bool, bool) {
        match ty {
            ReturnType::Default => (false, false),
            ReturnType::Type(_, ty) => {
                let type_str = self.type_to_string(ty);
                let is_result =
                    type_str.contains("Result") || type_str.contains("std::result::Result");
                let is_option =
                    type_str.contains("Option") || type_str.contains("std::option::Option");
                (is_result, is_option)
            }
        }
    }

    /// Add an instrumentable element
    fn add_element(&mut self, kind: ElementKind, span: Span) {
        let element = InstrumentableElement {
            id: self.generate_id(kind),
            kind,
            location: self.location_from_span(span),
            parent_id: self
                .context
                .current_function
                .as_ref()
                .map(|f| f.info.id.clone()),
            is_test: self.context.in_test_context,
        };
        self.result.elements.push(element);
    }

    /// Process a function and extract its information
    fn process_function(&mut self, item_fn: &ItemFn, is_method: bool) {
        let name = item_fn.sig.ident.to_string();
        let is_test = self.is_test_item(&item_fn.attrs);
        let was_in_test = self.context.in_test_context;

        if is_test {
            self.context.in_test_context = true;
        }

        // Build function info
        let full_path = if self.current_module_path().is_empty() {
            name.clone()
        } else {
            format!("{}::{}", self.current_module_path(), name)
        };

        let param_count = item_fn.sig.inputs.len() - usize::from(is_method);
        let return_type = self.return_type_to_string(&item_fn.sig.output);

        // Initialize error handling info based on return type
        let mut error_handling = ErrorHandlingInfo::default();
        if return_type.is_some() {
            let (is_result, is_option) = self.is_result_or_option_type(&item_fn.sig.output);
            if is_result {
                error_handling.result_returns = 1;
            }
            if is_option {
                error_handling.option_returns = 1;
            }
        }

        let function_info = FunctionInfo {
            id: self.generate_id(ElementKind::Function),
            name: name.clone(),
            full_path,
            is_async: item_fn.sig.asyncness.is_some(),
            is_unsafe: item_fn.sig.unsafety.is_some(),
            is_test,
            is_generic: !item_fn.sig.generics.params.is_empty(),
            param_count,
            return_type: return_type.clone(),
            calls: Vec::new(),
            error_handling: error_handling.clone(),
            complexity: ComplexityMetrics::default(),
            location: self.location_from_span(item_fn.sig.ident.span()),
        };

        // Set up function context
        let function_context = FunctionContext {
            info: function_info.clone(),
            complexity: ComplexityMetrics {
                cyclomatic: 1, // Base complexity for a function
                cognitive: 0,
                lines_of_code: 0, // Will be calculated
                statement_count: 0,
                max_nesting_depth: 0,
                branch_count: 0,
                loop_count: 0,
            },
            error_handling,
            calls: Vec::new(),
        };

        let previous_function = self.context.current_function.replace(function_context);

        // Visit the function body
        self.visit_block(&item_fn.block);

        // Finalize function info
        if let Some(mut func_context) = self.context.current_function.take() {
            func_context.info.calls = func_context.calls;
            func_context.info.error_handling = func_context.error_handling;
            func_context.info.complexity = func_context.complexity;

            if is_test {
                self.result.test_functions.push(func_context.info);
            } else {
                self.result.functions.push(func_context.info);
            }
        }

        self.context.current_function = previous_function;
        self.context.in_test_context = was_in_test;
    }

    /// Update complexity metrics for control flow
    fn increment_complexity(&mut self, cognitive_increment: usize) {
        if let Some(ref mut func) = self.context.current_function {
            func.complexity.cyclomatic += 1;
            func.complexity.cognitive += cognitive_increment;
            func.complexity.branch_count += 1;
        }
    }

    /// Track nesting depth
    fn with_nesting<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.context.nesting_depth += 1;

        if let Some(ref mut func) = self.context.current_function {
            if self.context.nesting_depth > func.complexity.max_nesting_depth {
                func.complexity.max_nesting_depth = self.context.nesting_depth;
            }
        }

        f(self);

        self.context.nesting_depth -= 1;
    }
}

impl<'ast> Visit<'ast> for AstVisitor {
    fn visit_file(&mut self, node: &'ast File) {
        visit::visit_file(self, node);
    }

    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Fn(item_fn) => {
                self.process_function(item_fn, false);
            }
            Item::Mod(item_mod) => {
                let is_test = self.is_cfg_test(&item_mod.attrs);
                let was_in_test = self.context.in_test_context;

                if is_test {
                    self.context.in_test_context = true;
                }

                self.context.module_path.push(item_mod.ident.to_string());

                let module_info = ModuleInfo {
                    name: item_mod.ident.to_string(),
                    path: self.context.module_path.clone(),
                    is_test,
                    location: self.location_from_span(item_mod.ident.span()),
                };
                self.result.modules.push(module_info);

                visit::visit_item_mod(self, item_mod);

                self.context.module_path.pop();
                self.context.in_test_context = was_in_test;
            }
            Item::Impl(item_impl) => {
                visit::visit_item_impl(self, item_impl);
            }
            _ => {
                visit::visit_item(self, item);
            }
        }
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        if let ImplItem::Fn(method) = item {
            self.process_function(
                &ItemFn {
                    attrs: method.attrs.clone(),
                    vis: method.vis.clone(),
                    sig: method.sig.clone(),
                    block: Box::new(method.block.clone()),
                },
                true,
            );
        } else {
            visit::visit_impl_item(self, item);
        }
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            Expr::Call(call) => {
                self.add_element(ElementKind::FunctionCall, call.func.span());

                // Track the call
                if let Expr::Path(path) = &*call.func {
                    let callee = quote! { #path }.to_string();
                    let location = self.location_from_span(call.func.span());
                    if let Some(ref mut func) = self.context.current_function {
                        func.calls.push(CallInfo {
                            callee,
                            is_method: false,
                            location,
                        });
                    }
                }

                visit::visit_expr(self, &call.func);
                for arg in &call.args {
                    visit::visit_expr(self, arg);
                }
            }
            Expr::MethodCall(call) => {
                self.add_element(ElementKind::MethodCall, call.method.span());

                // Track the method call
                let method_name = call.method.to_string();
                let location = self.location_from_span(call.method.span());
                if let Some(ref mut func) = self.context.current_function {
                    func.calls.push(CallInfo {
                        callee: method_name.clone(),
                        is_method: true,
                        location,
                    });

                    // Track error handling patterns
                    if method_name == "unwrap" {
                        func.error_handling.unwrap_calls += 1;
                    } else if method_name == "expect" {
                        func.error_handling.expect_calls += 1;
                    }
                }

                visit::visit_expr(self, &call.receiver);
                for arg in &call.args {
                    visit::visit_expr(self, arg);
                }
            }
            Expr::If(expr_if) => {
                self.add_element(ElementKind::IfExpression, expr_if.if_token.span);
                self.increment_complexity(1);

                self.with_nesting(|visitor| {
                    visit::visit_expr(visitor, &expr_if.cond);
                    visit::visit_block(visitor, &expr_if.then_branch);
                    if let Some((_, ref else_branch)) = expr_if.else_branch {
                        visit::visit_expr(visitor, else_branch);
                    }
                });
            }
            Expr::Match(expr_match) => {
                self.add_element(ElementKind::MatchExpression, expr_match.match_token.span);

                // Check if matching on Result or Option
                if let Some(ref mut func) = self.context.current_function {
                    let match_expr_str = quote! { #expr_match }.to_string();
                    if match_expr_str.contains("Result") || match_expr_str.contains("Option") {
                        func.error_handling.error_matches += 1;
                    }
                }

                let arm_count = expr_match.arms.len();
                if arm_count > 1 {
                    self.increment_complexity(arm_count - 1);
                }

                self.with_nesting(|visitor| {
                    visit::visit_expr(visitor, &expr_match.expr);
                    for arm in &expr_match.arms {
                        visitor.add_element(ElementKind::MatchArm, arm.pat.span());
                        visit::visit_pat(visitor, &arm.pat);
                        if let Some(ref guard) = arm.guard {
                            visit::visit_expr(visitor, &guard.1);
                        }
                        visit::visit_expr(visitor, &arm.body);
                    }
                });
            }
            Expr::Loop(expr_loop) => {
                self.add_element(ElementKind::Loop, expr_loop.loop_token.span);
                self.increment_complexity(1);

                if let Some(ref mut func) = self.context.current_function {
                    func.complexity.loop_count += 1;
                }

                self.with_nesting(|visitor| {
                    visit::visit_block(visitor, &expr_loop.body);
                });
            }
            Expr::While(expr_while) => {
                self.add_element(ElementKind::Loop, expr_while.while_token.span);
                self.increment_complexity(1);

                if let Some(ref mut func) = self.context.current_function {
                    func.complexity.loop_count += 1;
                }

                self.with_nesting(|visitor| {
                    visit::visit_expr(visitor, &expr_while.cond);
                    visit::visit_block(visitor, &expr_while.body);
                });
            }
            Expr::ForLoop(expr_for) => {
                self.add_element(ElementKind::Loop, expr_for.for_token.span);
                self.increment_complexity(1);

                if let Some(ref mut func) = self.context.current_function {
                    func.complexity.loop_count += 1;
                }

                self.with_nesting(|visitor| {
                    visit::visit_pat(visitor, &expr_for.pat);
                    visit::visit_expr(visitor, &expr_for.expr);
                    visit::visit_block(visitor, &expr_for.body);
                });
            }
            Expr::Return(expr_return) => {
                self.add_element(ElementKind::Return, expr_return.return_token.span);
                if let Some(ref expr) = expr_return.expr {
                    visit::visit_expr(self, expr);
                }
            }
            Expr::Try(expr_try) => {
                if let Some(ref mut func) = self.context.current_function {
                    func.error_handling.question_mark_ops += 1;
                }
                visit::visit_expr(self, &expr_try.expr);
            }
            Expr::Closure(closure) => {
                self.add_element(ElementKind::Closure, closure.or1_token.span);
                self.increment_complexity(1);

                self.with_nesting(|visitor| {
                    visit::visit_expr(visitor, &closure.body);
                });
            }
            Expr::Binary(binary) => {
                self.add_element(ElementKind::BinaryOp, expr.span());

                // Check for && and || which increase cyclomatic complexity
                match binary.op {
                    BinOp::And(_) | BinOp::Or(_) => {
                        self.increment_complexity(1);
                    }
                    _ => {}
                }

                visit::visit_expr(self, &binary.left);
                visit::visit_expr(self, &binary.right);
            }
            Expr::Unary(unary) => {
                self.add_element(ElementKind::UnaryOp, expr.span());
                visit::visit_expr(self, &unary.expr);
            }
            Expr::Assign(assign) => {
                self.add_element(ElementKind::Assignment, assign.eq_token.span);
                visit::visit_expr(self, &assign.left);
                visit::visit_expr(self, &assign.right);
            }
            _ => {
                visit::visit_expr(self, expr);
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if let Some(ref mut func) = self.context.current_function {
            func.complexity.statement_count += 1;
        }

        match stmt {
            Stmt::Expr(expr, _) => {
                self.add_element(ElementKind::Statement, expr.span());
            }
            _ => {}
        }

        visit::visit_stmt(self, stmt);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        // Count non-empty lines in the block for lines_of_code
        if let Some(ref mut func) = self.context.current_function {
            let block_str = quote! { #block }.to_string();
            let lines: Vec<&str> = block_str
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with("//")
                })
                .collect();
            func.complexity.lines_of_code += lines.len();
        }

        visit::visit_block(self, block);
    }
}

/// Public interface for performing AST analysis
pub fn analyze_ast(source_file: SourceFile) -> AnalysisResult {
    let visitor = AstVisitor::new(source_file);
    visitor.analyze()
}

// ✅ Source location mapping implemented using proc_macro2's span locations
// The location_from_span function now properly extracts line/column info
