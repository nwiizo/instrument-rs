//! Symbol resolution for function references

use std::collections::HashMap;
use std::path::PathBuf;
use syn::{Item, ItemFn, ItemMod, Path};

/// Resolves symbols and function references in the codebase
#[derive(Debug)]
pub struct SymbolResolver {
    /// Map from symbol name to its resolved information
    symbols: HashMap<String, ResolvedSymbol>,
    /// Current module path during resolution
    current_module_path: Vec<String>,
    /// Map of use statements for import resolution
    imports: HashMap<String, String>,
}

impl SymbolResolver {
    /// Creates a new symbol resolver
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            current_module_path: Vec::new(),
            imports: HashMap::new(),
        }
    }

    /// Registers a function symbol
    ///
    /// # Arguments
    ///
    /// * `name` - The function name
    /// * `module_path` - The module path to the function
    /// * `file_path` - The file where the function is defined
    /// * `is_public` - Whether the function is public
    pub fn register_function(
        &mut self,
        name: &str,
        module_path: Vec<String>,
        file_path: PathBuf,
        is_public: bool,
    ) {
        let full_path = if module_path.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", module_path.join("::"), name)
        };

        let symbol = ResolvedSymbol {
            name: name.to_string(),
            full_path: full_path.clone(),
            module_path,
            file_path,
            kind: SymbolKind::Function,
            is_public,
            is_external: false,
        };

        self.symbols.insert(full_path.clone(), symbol.clone());

        // Also register by short name if unique
        if !self.symbols.contains_key(name) {
            self.symbols.insert(name.to_string(), symbol);
        }
    }

    /// Registers a use statement
    ///
    /// # Arguments
    ///
    /// * `local_name` - The local name used in code
    /// * `full_path` - The full path being imported
    pub fn register_import(&mut self, local_name: &str, full_path: &str) {
        self.imports
            .insert(local_name.to_string(), full_path.to_string());
    }

    /// Enters a module scope
    pub fn enter_module(&mut self, module_name: &str) {
        self.current_module_path.push(module_name.to_string());
    }

    /// Exits the current module scope
    pub fn exit_module(&mut self) {
        self.current_module_path.pop();
    }

    /// Resolves a function path to its full qualified name
    ///
    /// # Arguments
    ///
    /// * `path` - The path to resolve
    ///
    /// # Returns
    ///
    /// The resolved symbol information, or None if not found
    pub fn resolve_path(&self, path: &Path) -> Option<ResolvedSymbol> {
        let path_str = quote::quote!(#path).to_string();

        // Try direct lookup
        if let Some(symbol) = self.symbols.get(&path_str) {
            return Some(symbol.clone());
        }

        // Try resolving through imports
        let segments: Vec<_> = path.segments.iter().collect();
        if let Some(first_segment) = segments.first() {
            let first_name = first_segment.ident.to_string();

            // Check if it's an import
            if let Some(imported_path) = self.imports.get(&first_name) {
                let mut full_path = imported_path.clone();
                for segment in segments.iter().skip(1) {
                    full_path.push_str("::");
                    full_path.push_str(&segment.ident.to_string());
                }

                if let Some(symbol) = self.symbols.get(&full_path) {
                    return Some(symbol.clone());
                }
            }
        }

        // Try resolving relative to current module
        if !self.current_module_path.is_empty() {
            let mut search_path = self.current_module_path.clone();

            // Try each parent module
            while !search_path.is_empty() {
                let mut candidate = search_path.join("::");
                candidate.push_str("::");
                candidate.push_str(&path_str);

                if let Some(symbol) = self.symbols.get(&candidate) {
                    return Some(symbol.clone());
                }

                search_path.pop();
            }
        }

        // Check if it's an external symbol
        if self.is_external_path(path) {
            return Some(self.create_external_symbol(path));
        }

        None
    }

    /// Checks if a path refers to an external crate or standard library
    fn is_external_path(&self, path: &Path) -> bool {
        if let Some(first_segment) = path.segments.first() {
            let first_name = first_segment.ident.to_string();

            // Common external crates and std modules
            matches!(
                first_name.as_str(),
                "std"
                    | "core"
                    | "alloc"
                    | "proc_macro"
                    | "tokio"
                    | "async_trait"
                    | "serde"
                    | "anyhow"
                    | "thiserror"
                    | "log"
                    | "tracing"
                    | "futures"
            ) || !self.symbols.contains_key(&first_name)
        } else {
            false
        }
    }

    /// Creates an external symbol representation
    fn create_external_symbol(&self, path: &Path) -> ResolvedSymbol {
        let path_str = quote::quote!(#path).to_string();
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect();

        let name = segments
            .last()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        ResolvedSymbol {
            name,
            full_path: path_str,
            module_path: segments,
            file_path: PathBuf::new(), // External symbols don't have local file paths
            kind: SymbolKind::Function,
            is_public: true, // Assume external symbols are public
            is_external: true,
        }
    }

    /// Processes an item to extract symbols
    pub fn process_item(&mut self, item: &Item, file_path: &PathBuf) {
        match item {
            Item::Fn(item_fn) => {
                self.process_function(item_fn, file_path);
            }
            Item::Mod(item_mod) => {
                self.process_module(item_mod, file_path);
            }
            Item::Use(item_use) => {
                self.process_use(&item_use.tree);
            }
            _ => {}
        }
    }

    /// Processes a function item
    fn process_function(&mut self, item_fn: &ItemFn, file_path: &PathBuf) {
        let is_public = matches!(item_fn.vis, syn::Visibility::Public(_));

        self.register_function(
            &item_fn.sig.ident.to_string(),
            self.current_module_path.clone(),
            file_path.clone(),
            is_public,
        );
    }

    /// Processes a module item
    fn process_module(&mut self, item_mod: &ItemMod, file_path: &PathBuf) {
        let module_name = item_mod.ident.to_string();
        self.enter_module(&module_name);

        if let Some(content) = &item_mod.content {
            for item in &content.1 {
                self.process_item(item, file_path);
            }
        }

        self.exit_module();
    }

    /// Processes a use statement
    fn process_use(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(use_path) => {
                let mut path = vec![use_path.ident.to_string()];
                self.collect_use_path(&use_path.tree, &mut path);
            }
            syn::UseTree::Name(use_name) => {
                let name = use_name.ident.to_string();
                self.imports.insert(name.clone(), name);
            }
            syn::UseTree::Rename(use_rename) => {
                let original = use_rename.ident.to_string();
                let renamed = use_rename.rename.to_string();
                self.imports.insert(renamed, original);
            }
            syn::UseTree::Group(use_group) => {
                for tree in &use_group.items {
                    self.process_use(tree);
                }
            }
            _ => {}
        }
    }

    /// Collects the full path from a use tree
    fn collect_use_path(&mut self, tree: &syn::UseTree, path: &mut Vec<String>) {
        match tree {
            syn::UseTree::Path(use_path) => {
                path.push(use_path.ident.to_string());
                self.collect_use_path(&use_path.tree, path);
            }
            syn::UseTree::Name(use_name) => {
                let local_name = use_name.ident.to_string();
                let full_path = path.join("::");
                self.imports
                    .insert(local_name, format!("{}::{}", full_path, use_name.ident));
            }
            syn::UseTree::Rename(use_rename) => {
                let renamed = use_rename.rename.to_string();
                let full_path = path.join("::");
                self.imports
                    .insert(renamed, format!("{}::{}", full_path, use_rename.ident));
            }
            _ => {}
        }
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a resolved symbol
#[derive(Debug, Clone)]
pub struct ResolvedSymbol {
    /// The symbol name
    pub name: String,
    /// The full qualified path
    pub full_path: String,
    /// Module path components
    pub module_path: Vec<String>,
    /// File where the symbol is defined
    pub file_path: PathBuf,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// Whether the symbol is public
    pub is_public: bool,
    /// Whether this is an external symbol
    pub is_external: bool,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Function symbol
    Function,
    /// Struct symbol
    Struct,
    /// Trait symbol
    Trait,
    /// Module symbol
    Module,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_registration() {
        let mut resolver = SymbolResolver::new();

        resolver.register_function(
            "test_fn",
            vec!["module".to_string()],
            PathBuf::from("src/module.rs"),
            true,
        );

        assert!(resolver.symbols.contains_key("module::test_fn"));
        assert!(resolver.symbols.contains_key("test_fn"));

        let symbol = &resolver.symbols["module::test_fn"];
        assert_eq!(symbol.name, "test_fn");
        assert_eq!(symbol.full_path, "module::test_fn");
        assert!(symbol.is_public);
        assert!(!symbol.is_external);
    }

    #[test]
    fn test_import_registration() {
        let mut resolver = SymbolResolver::new();

        resolver.register_import("HashMap", "std::collections::HashMap");
        assert_eq!(resolver.imports["HashMap"], "std::collections::HashMap");
    }

    #[test]
    fn test_module_scoping() {
        let mut resolver = SymbolResolver::new();

        assert!(resolver.current_module_path.is_empty());

        resolver.enter_module("outer");
        assert_eq!(resolver.current_module_path, vec!["outer"]);

        resolver.enter_module("inner");
        assert_eq!(resolver.current_module_path, vec!["outer", "inner"]);

        resolver.exit_module();
        assert_eq!(resolver.current_module_path, vec!["outer"]);

        resolver.exit_module();
        assert!(resolver.current_module_path.is_empty());
    }

    #[test]
    fn test_external_path_detection() {
        let resolver = SymbolResolver::new();

        let std_path: Path = syn::parse_quote!(std::collections::HashMap::new);
        assert!(resolver.is_external_path(&std_path));

        let tokio_path: Path = syn::parse_quote!(tokio::spawn);
        assert!(resolver.is_external_path(&tokio_path));
    }

    #[test]
    fn test_external_symbol_creation() {
        let resolver = SymbolResolver::new();

        let path: Path = syn::parse_quote!(std::collections::HashMap::new);
        let symbol = resolver.create_external_symbol(&path);

        assert_eq!(symbol.name, "new");
        assert_eq!(symbol.full_path, "std :: collections :: HashMap :: new");
        assert!(symbol.is_external);
        assert!(symbol.is_public);
    }
}
