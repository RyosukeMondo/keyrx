use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use syn::{File, Item};

use crate::types::{FunctionGroup, GenerationResult, ModuleAnalysis};

pub struct ModuleGenerator<'a> {
    module_path: &'a PathBuf,
    #[allow(dead_code)]
    analysis: &'a ModuleAnalysis,
    #[allow(dead_code)]
    content: String,
    syntax_tree: File,
}

impl<'a> ModuleGenerator<'a> {
    pub fn new(module_path: &'a PathBuf, analysis: &'a ModuleAnalysis) -> Self {
        let content = fs::read_to_string(module_path)
            .expect("Failed to read module file");

        let syntax_tree: File = syn::parse_file(&content)
            .expect("Failed to parse module");

        Self {
            module_path,
            analysis,
            content,
            syntax_tree,
        }
    }

    pub fn generate_modules(&self, groups: &[FunctionGroup]) -> Result<GenerationResult> {
        // Create output directory
        let output_dir = self.create_output_directory()?;

        // Create backup of original file
        let backup_file = self.create_backup()?;

        // Extract common imports and module attributes
        let common_imports = self.extract_imports();
        let module_attrs = self.extract_module_attributes();

        // Generate individual module files
        let mut created_files = Vec::new();

        for group in groups {
            let module_file = self.generate_group_module(
                &output_dir,
                group,
                &common_imports,
                &module_attrs,
            )?;
            created_files.push(module_file);
        }

        // Generate mod.rs
        let mod_file = self.generate_mod_file(&output_dir, groups)?;

        Ok(GenerationResult {
            created_files,
            mod_file: Some(mod_file),
            backup_file: Some(backup_file),
        })
    }

    fn create_output_directory(&self) -> Result<PathBuf> {
        let parent = self.module_path.parent()
            .context("Module has no parent directory")?;

        let module_stem = self.module_path.file_stem()
            .context("Module has no file name")?
            .to_string_lossy();

        // Remove "exports_" prefix if present for cleaner naming
        let dir_name = module_stem.strip_prefix("exports_")
            .unwrap_or(&module_stem);

        let output_dir = parent.join(dir_name);

        fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

        Ok(output_dir)
    }

    fn create_backup(&self) -> Result<PathBuf> {
        let backup_path = self.module_path.with_extension("rs.backup");

        fs::copy(self.module_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;

        Ok(backup_path)
    }

    fn extract_imports(&self) -> Vec<String> {
        let mut imports = Vec::new();

        for item in &self.syntax_tree.items {
            if let Item::Use(use_item) = item {
                let import = quote::quote!(#use_item).to_string();
                imports.push(import);
            }
        }

        imports
    }

    fn extract_module_attributes(&self) -> Vec<String> {
        self.syntax_tree
            .attrs
            .iter()
            .filter(|attr| {
                let attr_str = quote::quote!(#attr).to_string();
                // Only include #![allow(...)] and similar module-level attributes
                attr_str.starts_with("# !") && !attr_str.contains("doc")
            })
            .map(|attr| quote::quote!(#attr).to_string())
            .collect()
    }

    fn generate_group_module(
        &self,
        output_dir: &PathBuf,
        group: &FunctionGroup,
        common_imports: &[String],
        module_attrs: &[String],
    ) -> Result<PathBuf> {
        let mut content = String::new();

        // Add module documentation
        content.push_str(&format!("//! {} functions\n", group.name));
        content.push_str("//!\n");
        content.push_str(&format!("//! This module contains {} functions grouped by {}.\n",
            group.functions.len(),
            match group.grouping_strategy {
                crate::types::GroupingStrategy::ByDomain => "domain",
                crate::types::GroupingStrategy::ByPrefix => "naming prefix",
                _ => "manual grouping",
            }
        ));
        content.push('\n');

        // Add module attributes
        for attr in module_attrs {
            content.push_str(&attr);
            content.push('\n');
        }
        if !module_attrs.is_empty() {
            content.push('\n');
        }

        // Add imports
        for import in common_imports {
            content.push_str(&import);
            content.push('\n');
        }
        content.push('\n');

        // Extract and add functions
        let function_set: HashSet<&String> = group.functions.iter().collect();

        for item in &self.syntax_tree.items {
            if let Item::Fn(func) = item {
                let func_name = func.sig.ident.to_string();

                if function_set.contains(&func_name) {
                    let func_str = quote::quote!(#func).to_string();
                    content.push_str(&func_str);
                    content.push_str("\n\n");
                }
            }
        }

        // Also extract related types if any
        // For now, we'll skip type extraction to keep it simple

        // Write the module file
        let module_file = output_dir.join(format!("{}.rs", sanitize_module_name(&group.name)));

        fs::write(&module_file, content)
            .with_context(|| format!("Failed to write module file: {}", module_file.display()))?;

        Ok(module_file)
    }

    fn generate_mod_file(&self, output_dir: &PathBuf, groups: &[FunctionGroup]) -> Result<PathBuf> {
        let mut content = String::new();

        // Add module documentation
        content.push_str("//! Module exports\n");
        content.push_str("//!\n");
        content.push_str("//! This module re-exports all functions from submodules.\n");
        content.push_str("//! Auto-generated by module_splitter tool.\n");
        content.push('\n');

        // Add module declarations
        for group in groups {
            let module_name = sanitize_module_name(&group.name);
            content.push_str(&format!("pub mod {};\n", module_name));
        }
        content.push('\n');

        // Add re-exports
        content.push_str("// Re-export all functions\n");
        for group in groups {
            let module_name = sanitize_module_name(&group.name);
            content.push_str(&format!("pub use {}::*;\n", module_name));
        }

        let mod_file = output_dir.join("mod.rs");

        fs::write(&mod_file, content)
            .with_context(|| format!("Failed to write mod.rs: {}", mod_file.display()))?;

        Ok(mod_file)
    }
}

fn sanitize_module_name(name: &str) -> String {
    name.to_lowercase()
        .replace('-', "_")
        .replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_module_name() {
        assert_eq!(sanitize_module_name("FooBar"), "foobar");
        assert_eq!(sanitize_module_name("foo-bar"), "foo_bar");
        assert_eq!(sanitize_module_name("Foo Bar"), "foo_bar");
        assert_eq!(sanitize_module_name("get_state"), "get_state");
    }
}
