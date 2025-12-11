mod contract_adherence;

use keyrx_core::ffi::contract::ContractRegistry;
use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
fn verify_ffi_contract_adherence() {
    // 1. Load all contracts
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let contracts_dir = manifest_dir.join("src/ffi/contracts");
    let registry =
        ContractRegistry::load_from_dir(&contracts_dir).expect("Failed to load contracts");

    // 2. Collect all expected Rust function names from contracts
    let mut expected_functions = HashSet::new();
    for contract in registry.all_contracts().values() {
        for func in &contract.functions {
            // "keyrx_domain_name" is the default convention unless overridden
            let rust_name = func
                .rust_name
                .clone()
                .unwrap_or_else(|| format!("keyrx_{}_{}", contract.domain, func.name));
            expected_functions.insert(rust_name);
        }
    }

    // 3. Scan codebase for #[no_mangle] exports matching these names
    // This is a heuristic check (static analysis) to ensure implementation exists.
    let src_dir = manifest_dir.join("src");
    let mut found_functions = HashSet::new();

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |ext| ext == "rs") {
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();

            // Simple string matching for now.
            // A more robust solution would use `syn` to parse AST, but this catches 99% of cases
            // where we simply forgot to implement the function or mistyped the name.
            for expected in &expected_functions {
                if content.contains(&format!("fn {}(", expected))
                    && content.contains("#[no_mangle]")
                {
                    found_functions.insert(expected.clone());
                }
            }
        }
    }

    // 4. Report missing implementations
    let missing: Vec<&String> = expected_functions.difference(&found_functions).collect();

    if !missing.is_empty() {
        panic!(
            "Missing FFI implementations for contract functions:\n{:#?}\n\n\
            These functions are defined in .ffi-contract.json files but could not be found \
            exported in the Rust codebase. Please implement them or check naming conventions.",
            missing
        );
    }
}
