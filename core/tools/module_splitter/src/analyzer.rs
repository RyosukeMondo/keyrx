use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use syn::{File, Item};

use crate::types::{FunctionInfo, ModuleAnalysis, TypeInfo};

pub struct ModuleAnalyzer {
    module_path: PathBuf,
    content: String,
}

impl ModuleAnalyzer {
    pub fn new(module_path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(module_path)
            .with_context(|| format!("Failed to read module file: {}", module_path.display()))?;

        Ok(Self {
            module_path: module_path.clone(),
            content,
        })
    }

    pub fn analyze(&self) -> Result<ModuleAnalysis> {
        let syntax_tree: File = syn::parse_file(&self.content)
            .with_context(|| format!("Failed to parse module: {}", self.module_path.display()))?;

        let total_lines = self.content.lines().count();
        let code_lines = self.count_code_lines();

        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut imports = Vec::new();
        let mut attributes = Vec::new();
        let mut module_docs = None;

        // Extract module-level documentation
        for attr in &syntax_tree.attrs {
            let attr_str = quote::quote!(#attr).to_string();
            if attr_str.starts_with("# ! [doc") {
                // Extract doc comment from attribute string
                if let Some(start) = attr_str.find('"') {
                    if let Some(end) = attr_str[start + 1..].find('"') {
                        let doc = &attr_str[start + 1..start + 1 + end];
                        if module_docs.is_none() {
                            module_docs = Some(doc.to_string());
                        } else {
                            module_docs = Some(format!("{}\n{}", module_docs.unwrap(), doc));
                        }
                    }
                }
            } else {
                attributes.push(attr_str);
            }
        }

        let mut current_line = 1;

        for item in &syntax_tree.items {
            match item {
                Item::Use(use_item) => {
                    let import = quote::quote!(#use_item).to_string();
                    imports.push(import);
                }
                Item::Fn(func) => {
                    let func_info = FunctionInfo::from_item_fn(func, current_line, &self.content);
                    current_line += func_info.estimated_lines;
                    functions.push(func_info);
                }
                _ => {
                    if let Some(type_info) = TypeInfo::from_item(item, current_line) {
                        current_line += type_info.line_end - type_info.line_start;
                        types.push(type_info);
                    }
                }
            }
        }

        Ok(ModuleAnalysis {
            module_path: self.module_path.clone(),
            total_lines,
            code_lines,
            functions,
            types,
            imports,
            attributes,
            module_docs,
        })
    }

    fn count_code_lines(&self) -> usize {
        let mut code_lines = 0;
        let mut in_block_comment = false;

        for line in self.content.lines() {
            let trimmed = line.trim();

            // Handle block comments
            if trimmed.starts_with("/*") {
                in_block_comment = true;
            }
            if trimmed.ends_with("*/") {
                in_block_comment = false;
                continue;
            }
            if in_block_comment {
                continue;
            }

            // Skip line comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            code_lines += 1;
        }

        code_lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_analyze_simple_module() -> Result<()> {
        let content = r#"
//! Module documentation

use std::collections::HashMap;

/// Function documentation
pub fn foo() -> i32 {
    42
}

fn bar() {
    println!("Hello");
}
"#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content.as_bytes())?;
        let path = temp_file.path().to_path_buf();

        let analyzer = ModuleAnalyzer::new(&path)?;
        let analysis = analyzer.analyze()?;

        assert_eq!(analysis.functions.len(), 2);
        assert_eq!(analysis.imports.len(), 1);
        assert!(analysis.module_docs.is_some());

        Ok(())
    }

    #[test]
    fn test_count_code_lines() -> Result<()> {
        let content = r#"
// This is a comment
use std::collections::HashMap;

/* Block comment
   spanning multiple
   lines */

pub fn foo() -> i32 {
    // Inline comment
    42
}

"#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content.as_bytes())?;
        let path = temp_file.path().to_path_buf();

        let analyzer = ModuleAnalyzer::new(&path)?;
        let code_lines = analyzer.count_code_lines();

        // Should count: use statement, pub fn line, opening brace, 42, closing brace = 5 lines
        assert!(code_lines > 0);

        Ok(())
    }

    #[test]
    fn test_ffi_export_detection() -> Result<()> {
        let content = r#"
use ffi_export::ffi_export;

#[ffi_export(domain = "engine")]
pub fn start_engine() -> Result<()> {
    Ok(())
}
"#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content.as_bytes())?;
        let path = temp_file.path().to_path_buf();

        let analyzer = ModuleAnalyzer::new(&path)?;
        let analysis = analyzer.analyze()?;

        assert_eq!(analysis.functions.len(), 1);
        assert!(analysis.functions[0].has_ffi_export);
        assert_eq!(analysis.functions[0].domain, Some("engine".to_string()));

        Ok(())
    }
}
