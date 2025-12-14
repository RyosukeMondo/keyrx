use anyhow::Result;
use std::collections::HashMap;

use crate::types::{FunctionGroup, GroupingStrategy, ModuleAnalysis};

pub struct FunctionGrouper {
    max_size: usize,
}

impl FunctionGrouper {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    pub fn group_functions(&self, analysis: &ModuleAnalysis) -> Result<Vec<FunctionGroup>> {
        // Try different grouping strategies in order of preference
        let groups = if self.has_ffi_exports(analysis) {
            self.group_by_domain(analysis)?
        } else {
            self.group_by_prefix(analysis)?
        };

        // Validate that all groups are within size limits
        let validated_groups = self.validate_and_split_oversized_groups(groups)?;

        Ok(validated_groups)
    }

    fn has_ffi_exports(&self, analysis: &ModuleAnalysis) -> bool {
        analysis.functions.iter().any(|f| f.has_ffi_export)
    }

    fn group_by_domain(&self, analysis: &ModuleAnalysis) -> Result<Vec<FunctionGroup>> {
        let mut domain_groups: HashMap<String, Vec<String>> = HashMap::new();
        let mut domain_lines: HashMap<String, usize> = HashMap::new();

        for func in &analysis.functions {
            let domain = if let Some(d) = &func.domain {
                d.clone()
            } else {
                // For functions without domain, group by prefix
                func.prefix()
            };

            domain_groups
                .entry(domain.clone())
                .or_default()
                .push(func.name.clone());

            *domain_lines.entry(domain).or_default() += func.estimated_lines;
        }

        let mut groups = Vec::new();
        for (domain, functions) in domain_groups {
            let estimated_lines = domain_lines.get(&domain).copied().unwrap_or(0);

            groups.push(FunctionGroup {
                name: domain,
                functions,
                types: Vec::new(),
                estimated_lines,
                grouping_strategy: GroupingStrategy::ByDomain,
            });
        }

        // Sort groups by size (largest first) for better splitting
        groups.sort_by(|a, b| b.estimated_lines.cmp(&a.estimated_lines));

        Ok(groups)
    }

    fn group_by_prefix(&self, analysis: &ModuleAnalysis) -> Result<Vec<FunctionGroup>> {
        let mut prefix_groups: HashMap<String, Vec<String>> = HashMap::new();
        let mut prefix_lines: HashMap<String, usize> = HashMap::new();

        for func in &analysis.functions {
            let prefix = func.prefix();

            prefix_groups
                .entry(prefix.clone())
                .or_default()
                .push(func.name.clone());

            *prefix_lines.entry(prefix).or_default() += func.estimated_lines;
        }

        let mut groups = Vec::new();
        for (prefix, functions) in prefix_groups {
            let estimated_lines = prefix_lines.get(&prefix).copied().unwrap_or(0);

            groups.push(FunctionGroup {
                name: prefix,
                functions,
                types: Vec::new(),
                estimated_lines,
                grouping_strategy: GroupingStrategy::ByPrefix,
            });
        }

        // Sort groups by size (largest first)
        groups.sort_by(|a, b| b.estimated_lines.cmp(&a.estimated_lines));

        Ok(groups)
    }

    fn validate_and_split_oversized_groups(
        &self,
        groups: Vec<FunctionGroup>,
    ) -> Result<Vec<FunctionGroup>> {
        let mut result = Vec::new();

        for group in groups {
            if group.estimated_lines <= self.max_size {
                result.push(group);
            } else {
                // Split oversized group into multiple smaller groups
                let split_groups = self.split_oversized_group(group)?;
                result.extend(split_groups);
            }
        }

        Ok(result)
    }

    fn split_oversized_group(&self, group: FunctionGroup) -> Result<Vec<FunctionGroup>> {
        let functions_per_group = std::cmp::max(
            1,
            (group.functions.len() * self.max_size) / group.estimated_lines,
        );

        let mut split_groups = Vec::new();
        let mut current_group_idx = 1;

        for chunk in group.functions.chunks(functions_per_group) {
            let estimated_lines = (group.estimated_lines * chunk.len()) / group.functions.len();

            split_groups.push(FunctionGroup {
                name: format!("{}_{}", group.name, current_group_idx),
                functions: chunk.to_vec(),
                types: Vec::new(),
                estimated_lines,
                grouping_strategy: group.grouping_strategy.clone(),
            });

            current_group_idx += 1;
        }

        Ok(split_groups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FunctionInfo;
    use std::path::PathBuf;

    fn create_test_function(name: &str, lines: usize, domain: Option<&str>) -> FunctionInfo {
        FunctionInfo {
            name: name.to_string(),
            is_public: true,
            is_async: false,
            line_start: 0,
            line_end: lines,
            estimated_lines: lines,
            has_ffi_export: domain.is_some(),
            domain: domain.map(|s| s.to_string()),
            attributes: Vec::new(),
            doc_comments: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn test_group_by_domain() {
        let analysis = ModuleAnalysis {
            module_path: PathBuf::from("test.rs"),
            total_lines: 1000,
            code_lines: 800,
            functions: vec![
                create_test_function("start_engine", 50, Some("engine")),
                create_test_function("stop_engine", 40, Some("engine")),
                create_test_function("get_state", 30, Some("runtime")),
                create_test_function("set_state", 35, Some("runtime")),
            ],
            types: Vec::new(),
            imports: Vec::new(),
            attributes: Vec::new(),
            module_docs: None,
        };

        let grouper = FunctionGrouper::new(500);
        let groups = grouper.group_functions(&analysis).unwrap();

        assert_eq!(groups.len(), 2);

        let engine_group = groups.iter().find(|g| g.name == "engine").unwrap();
        assert_eq!(engine_group.functions.len(), 2);
        assert_eq!(engine_group.estimated_lines, 90);

        let runtime_group = groups.iter().find(|g| g.name == "runtime").unwrap();
        assert_eq!(runtime_group.functions.len(), 2);
        assert_eq!(runtime_group.estimated_lines, 65);
    }

    #[test]
    fn test_group_by_prefix() {
        let analysis = ModuleAnalysis {
            module_path: PathBuf::from("test.rs"),
            total_lines: 1000,
            code_lines: 800,
            functions: vec![
                create_test_function("get_state", 30, None),
                create_test_function("get_config", 25, None),
                create_test_function("set_state", 35, None),
                create_test_function("set_config", 40, None),
            ],
            types: Vec::new(),
            imports: Vec::new(),
            attributes: Vec::new(),
            module_docs: None,
        };

        let grouper = FunctionGrouper::new(500);
        let groups = grouper.group_functions(&analysis).unwrap();

        assert_eq!(groups.len(), 2);

        let get_group = groups.iter().find(|g| g.name == "get").unwrap();
        assert_eq!(get_group.functions.len(), 2);

        let set_group = groups.iter().find(|g| g.name == "set").unwrap();
        assert_eq!(set_group.functions.len(), 2);
    }

    #[test]
    fn test_split_oversized_group() {
        let analysis = ModuleAnalysis {
            module_path: PathBuf::from("test.rs"),
            total_lines: 2000,
            code_lines: 1800,
            functions: vec![
                create_test_function("func1", 200, Some("engine")),
                create_test_function("func2", 200, Some("engine")),
                create_test_function("func3", 200, Some("engine")),
                create_test_function("func4", 200, Some("engine")),
                create_test_function("func5", 200, Some("engine")),
            ],
            types: Vec::new(),
            imports: Vec::new(),
            attributes: Vec::new(),
            module_docs: None,
        };

        let grouper = FunctionGrouper::new(500);
        let groups = grouper.group_functions(&analysis).unwrap();

        // Should split into multiple groups
        assert!(groups.len() > 1);

        // Each group should be within size limit
        for group in &groups {
            assert!(group.estimated_lines <= 500);
        }
    }
}
