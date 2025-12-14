use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syn::{Item, ItemFn, ItemStruct, ItemEnum, ItemType, ItemConst, ItemStatic, ItemTrait, ItemImpl};

/// Analysis result for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAnalysis {
    pub module_path: PathBuf,
    pub total_lines: usize,
    pub code_lines: usize,
    pub functions: Vec<FunctionInfo>,
    pub types: Vec<TypeInfo>,
    pub imports: Vec<String>,
    pub attributes: Vec<String>,
    pub module_docs: Option<String>,
}

/// Information about a function in the module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub is_public: bool,
    pub is_async: bool,
    pub line_start: usize,
    pub line_end: usize,
    pub estimated_lines: usize,
    pub has_ffi_export: bool,
    pub domain: Option<String>,
    pub attributes: Vec<String>,
    pub doc_comments: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Information about a type (struct, enum, type alias, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
    pub is_public: bool,
    pub line_start: usize,
    pub line_end: usize,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TypeKind {
    Struct,
    Enum,
    TypeAlias,
    Const,
    Static,
    Trait,
    Impl,
}

/// A group of functions that should go into the same module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionGroup {
    pub name: String,
    pub functions: Vec<String>,
    pub types: Vec<String>,
    pub estimated_lines: usize,
    pub grouping_strategy: GroupingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupingStrategy {
    /// Group by function name prefix (e.g., "get_*", "set_*")
    ByPrefix,
    /// Group by FFI domain annotation
    ByDomain,
    /// Group by call graph analysis
    ByCallGraph,
    /// Group manually specified
    Manual,
}

/// Result of generating split modules
#[derive(Debug)]
pub struct GenerationResult {
    pub created_files: Vec<PathBuf>,
    pub mod_file: Option<PathBuf>,
    pub backup_file: Option<PathBuf>,
}

impl FunctionInfo {
    pub fn from_item_fn(item: &ItemFn, line_start: usize, _content: &str) -> Self {
        let name = item.sig.ident.to_string();
        let is_public = matches!(item.vis, syn::Visibility::Public(_));
        let is_async = item.sig.asyncness.is_some();

        // Count lines in function
        let func_str = quote::quote!(#item).to_string();
        let estimated_lines = func_str.lines().count();

        // Extract attributes
        let mut has_ffi_export = false;
        let mut domain = None;
        let mut attributes = Vec::new();
        let mut doc_comments = Vec::new();

        for attr in &item.attrs {
            let attr_str = quote::quote!(#attr).to_string();

            if attr_str.contains("ffi_export") {
                has_ffi_export = true;
                // Try to extract domain - simplified approach using string matching
                if let Some(domain_start) = attr_str.find("domain") {
                    if let Some(quote_start) = attr_str[domain_start..].find('"') {
                        let search_start = domain_start + quote_start + 1;
                        if let Some(quote_end) = attr_str[search_start..].find('"') {
                            let domain_value = &attr_str[search_start..search_start + quote_end];
                            domain = Some(domain_value.to_string());
                        }
                    }
                }
            }

            if attr_str.starts_with("# [doc") {
                // Extract doc comment from attribute string
                if let Some(start) = attr_str.find('"') {
                    if let Some(end) = attr_str[start + 1..].find('"') {
                        let doc = &attr_str[start + 1..start + 1 + end];
                        doc_comments.push(doc.to_string());
                    }
                }
            } else {
                attributes.push(attr_str);
            }
        }

        FunctionInfo {
            name,
            is_public,
            is_async,
            line_start,
            line_end: line_start + estimated_lines,
            estimated_lines,
            has_ffi_export,
            domain,
            attributes,
            doc_comments,
            dependencies: Vec::new(),
        }
    }

    /// Extract function name prefix for grouping
    pub fn prefix(&self) -> String {
        let parts: Vec<&str> = self.name.split('_').collect();
        if parts.len() > 1 {
            parts[0].to_string()
        } else {
            "misc".to_string()
        }
    }
}

impl TypeInfo {
    pub fn from_item(item: &Item, line_start: usize) -> Option<Self> {
        match item {
            Item::Struct(s) => Some(Self::from_struct(s, line_start)),
            Item::Enum(e) => Some(Self::from_enum(e, line_start)),
            Item::Type(t) => Some(Self::from_type_alias(t, line_start)),
            Item::Const(c) => Some(Self::from_const(c, line_start)),
            Item::Static(s) => Some(Self::from_static(s, line_start)),
            Item::Trait(t) => Some(Self::from_trait(t, line_start)),
            Item::Impl(i) => Some(Self::from_impl(i, line_start)),
            _ => None,
        }
    }

    fn from_struct(item: &ItemStruct, line_start: usize) -> Self {
        let name = item.ident.to_string();
        let is_public = matches!(item.vis, syn::Visibility::Public(_));
        let estimated_lines = quote::quote!(#item).to_string().lines().count();

        TypeInfo {
            name,
            kind: TypeKind::Struct,
            is_public,
            line_start,
            line_end: line_start + estimated_lines,
            attributes: item.attrs.iter().map(|a| quote::quote!(#a).to_string()).collect(),
        }
    }

    fn from_enum(item: &ItemEnum, line_start: usize) -> Self {
        let name = item.ident.to_string();
        let is_public = matches!(item.vis, syn::Visibility::Public(_));
        let estimated_lines = quote::quote!(#item).to_string().lines().count();

        TypeInfo {
            name,
            kind: TypeKind::Enum,
            is_public,
            line_start,
            line_end: line_start + estimated_lines,
            attributes: item.attrs.iter().map(|a| quote::quote!(#a).to_string()).collect(),
        }
    }

    fn from_type_alias(item: &ItemType, line_start: usize) -> Self {
        let name = item.ident.to_string();
        let is_public = matches!(item.vis, syn::Visibility::Public(_));
        let estimated_lines = quote::quote!(#item).to_string().lines().count();

        TypeInfo {
            name,
            kind: TypeKind::TypeAlias,
            is_public,
            line_start,
            line_end: line_start + estimated_lines,
            attributes: item.attrs.iter().map(|a| quote::quote!(#a).to_string()).collect(),
        }
    }

    fn from_const(item: &ItemConst, line_start: usize) -> Self {
        let name = item.ident.to_string();
        let is_public = matches!(item.vis, syn::Visibility::Public(_));
        let estimated_lines = quote::quote!(#item).to_string().lines().count();

        TypeInfo {
            name,
            kind: TypeKind::Const,
            is_public,
            line_start,
            line_end: line_start + estimated_lines,
            attributes: item.attrs.iter().map(|a| quote::quote!(#a).to_string()).collect(),
        }
    }

    fn from_static(item: &ItemStatic, line_start: usize) -> Self {
        let name = item.ident.to_string();
        let is_public = matches!(item.vis, syn::Visibility::Public(_));
        let estimated_lines = quote::quote!(#item).to_string().lines().count();

        TypeInfo {
            name,
            kind: TypeKind::Static,
            is_public,
            line_start,
            line_end: line_start + estimated_lines,
            attributes: item.attrs.iter().map(|a| quote::quote!(#a).to_string()).collect(),
        }
    }

    fn from_trait(item: &ItemTrait, line_start: usize) -> Self {
        let name = item.ident.to_string();
        let is_public = matches!(item.vis, syn::Visibility::Public(_));
        let estimated_lines = quote::quote!(#item).to_string().lines().count();

        TypeInfo {
            name,
            kind: TypeKind::Trait,
            is_public,
            line_start,
            line_end: line_start + estimated_lines,
            attributes: item.attrs.iter().map(|a| quote::quote!(#a).to_string()).collect(),
        }
    }

    fn from_impl(item: &ItemImpl, line_start: usize) -> Self {
        let name = if let Some((_, path, _)) = &item.trait_ {
            quote::quote!(#path).to_string()
        } else {
            quote::quote!(#item.self_ty).to_string()
        };
        let estimated_lines = quote::quote!(#item).to_string().lines().count();

        TypeInfo {
            name,
            kind: TypeKind::Impl,
            is_public: false,
            line_start,
            line_end: line_start + estimated_lines,
            attributes: item.attrs.iter().map(|a| quote::quote!(#a).to_string()).collect(),
        }
    }
}
