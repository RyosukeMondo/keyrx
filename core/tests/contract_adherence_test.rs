mod contract_adherence;

use contract_adherence::parser::parse_ffi_exports;
use contract_adherence::reporter::generate_full_report;
use contract_adherence::validator::validate_all_functions;
use keyrx_core::ffi::contract::ContractRegistry;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
fn verify_ffi_contract_adherence() {
    // 1. Load all contracts from the contracts directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let contracts_dir = manifest_dir.join("src/ffi/contracts");
    let registry =
        ContractRegistry::load_from_dir(&contracts_dir).expect("Failed to load contracts");

    // 2. Parse all FFI exports from Rust source files using AST parsing
    let src_dir = manifest_dir.join("src");
    let mut parsed_functions = Vec::new();

    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "rs") {
            match parse_ffi_exports(path) {
                Ok(funcs) => parsed_functions.extend(funcs),
                Err(e) => {
                    // Log parse errors but continue - some files may have syntax issues
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    // 3. Collect all contracts for validation
    let contracts: Vec<_> = registry.all_contracts().values().cloned().collect();

    // 4. Run enhanced validation comparing contracts against parsed implementations
    let report = validate_all_functions(&contracts, &parsed_functions);

    // 5. Generate and display report if validation failed
    if !report.is_success() {
        let error_report = generate_full_report(&report);
        panic!(
            "\n{}\n\nFFI contract validation failed. \
             Fix the errors above to ensure contract compliance.",
            error_report
        );
    }

    // Success: print summary
    println!(
        "FFI Contract Validation: {} functions validated successfully",
        report.passed
    );
}
