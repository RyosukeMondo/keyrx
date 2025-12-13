// Allow test-specific lints - tests need panic/unwrap/expect for failure assertions
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::expect_fun_call,
    clippy::print_stdout,
    clippy::print_stderr
)]

//! Integration tests for the Dart binding code generator
//!
//! These tests run the full generation pipeline with real contracts
//! and verify that the output is valid Dart code.

use generate_dart_bindings::{
    bindings_gen::{generate_ffi_signatures, generate_wrapper_functions},
    loader::load_all_contracts,
    models_gen::generate_all_models,
    templates::{to_camel_case, to_pascal_case},
};
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the contracts directory
fn contracts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/ffi/contracts")
}

/// Get a temporary directory for test outputs
fn temp_output_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Check if dart is available on the system
fn dart_available() -> bool {
    Command::new("dart").arg("--version").output().is_ok()
}

#[test]
fn test_load_all_real_contracts() {
    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");

    // We expect at least 4 contracts: config, engine, runtime, discovery
    assert!(
        contracts.len() >= 4,
        "Expected at least 4 contracts, found {}",
        contracts.len()
    );

    // Verify expected domains exist
    let domains: Vec<&str> = contracts.iter().map(|c| c.domain.as_str()).collect();
    assert!(domains.contains(&"config"), "Missing config domain");
    assert!(domains.contains(&"engine"), "Missing engine domain");
    assert!(domains.contains(&"runtime"), "Missing runtime domain");
    assert!(domains.contains(&"discovery"), "Missing discovery domain");
}

#[test]
fn test_generate_ffi_signatures_for_all_contracts() {
    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");

    for contract in &contracts {
        let signatures = generate_ffi_signatures(contract);
        assert!(
            signatures.is_ok(),
            "Failed to generate FFI signatures for domain '{}': {:?}",
            contract.domain,
            signatures.err()
        );

        let sigs = signatures.unwrap();
        // Each contract should have at least some signatures if it has functions
        if !contract.functions.is_empty() {
            assert!(
                !sigs.is_empty(),
                "Contract '{}' has functions but no signatures generated",
                contract.domain
            );
        }
    }
}

#[test]
fn test_generate_wrapper_functions_for_all_contracts() {
    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");

    for contract in &contracts {
        let wrappers = generate_wrapper_functions(contract);
        assert!(
            wrappers.is_ok(),
            "Failed to generate wrapper functions for domain '{}': {:?}",
            contract.domain,
            wrappers.err()
        );

        let wraps = wrappers.unwrap();
        // Each function in the contract should have a wrapper
        assert_eq!(
            wraps.len(),
            contract.functions.len(),
            "Wrapper count mismatch for domain '{}'",
            contract.domain
        );
    }
}

#[test]
fn test_generate_models_for_all_contracts() {
    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");

    for contract in &contracts {
        let models = generate_all_models(contract);
        assert!(
            models.is_ok(),
            "Failed to generate models for domain '{}': {:?}",
            contract.domain,
            models.err()
        );
    }
}

#[test]
fn test_full_generation_pipeline_produces_valid_dart() {
    use generate_dart_bindings::{
        bindings_gen::{
            generate_function_pointers_block, generate_typedefs_block, generate_wrappers_block,
        },
        header::{generate_bindings_header, generate_models_header},
        models_gen::generate_models_block,
    };

    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");
    let temp_dir = temp_output_dir();

    // Generate bindings file
    let mut bindings_parts = Vec::new();
    bindings_parts.push(generate_bindings_header());

    for contract in &contracts {
        let signatures = generate_ffi_signatures(contract).expect("Failed to generate signatures");
        bindings_parts.push(format!(
            "// {} Domain Bindings",
            to_pascal_case(&contract.domain)
        ));
        bindings_parts.push(generate_typedefs_block(&signatures));

        let class_name = format!("{}Bindings", to_pascal_case(&contract.domain));
        bindings_parts.push(format!("class {class_name} {{"));
        bindings_parts.push("  final DynamicLibrary _lib;".to_string());
        bindings_parts.push(format!("  {class_name}(this._lib);"));
        bindings_parts.push(generate_function_pointers_block(&signatures));

        let wrappers = generate_wrapper_functions(contract).expect("Failed to generate wrappers");
        bindings_parts.push(generate_wrappers_block(&wrappers));
        bindings_parts.push("}".to_string());
    }

    let bindings_code = bindings_parts.join("\n\n");
    let bindings_path = temp_dir.path().join("generated_bindings.dart");
    std::fs::write(&bindings_path, &bindings_code).expect("Failed to write bindings file");

    // Generate models file
    let mut models_parts = Vec::new();
    models_parts.push(generate_models_header());

    for contract in &contracts {
        let models = generate_all_models(contract).expect("Failed to generate models");
        if !models.is_empty() {
            models_parts.push(generate_models_block(&models));
        }
    }

    let models_code = models_parts.join("\n\n");
    let models_path = temp_dir.path().join("generated_models.dart");
    std::fs::write(&models_path, &models_code).expect("Failed to write models file");

    // Verify files were created and have content
    assert!(bindings_path.exists(), "Bindings file was not created");
    assert!(models_path.exists(), "Models file was not created");

    let bindings_content =
        std::fs::read_to_string(&bindings_path).expect("Failed to read bindings");
    let models_content = std::fs::read_to_string(&models_path).expect("Failed to read models");

    assert!(!bindings_content.is_empty(), "Bindings file is empty");
    assert!(!models_content.is_empty(), "Models file is empty");

    // Check for expected Dart code patterns
    assert!(
        bindings_content.contains("GENERATED CODE - DO NOT EDIT"),
        "Bindings missing GENERATED CODE header"
    );
    assert!(
        bindings_content.contains("import 'dart:ffi'"),
        "Bindings missing dart:ffi import"
    );
    assert!(
        bindings_content.contains("typedef"),
        "Bindings missing typedef declarations"
    );
    assert!(
        bindings_content.contains("class"),
        "Bindings missing class declarations"
    );

    assert!(
        models_content.contains("GENERATED CODE - DO NOT EDIT"),
        "Models missing GENERATED CODE header"
    );
}

#[test]
fn test_dart_analyze_on_generated_code() {
    if !dart_available() {
        eprintln!("Skipping dart analyze test: dart not available");
        return;
    }

    use generate_dart_bindings::{
        bindings_gen::{
            generate_function_pointers_block, generate_typedefs_block, generate_wrappers_block,
        },
        header::{generate_bindings_header, generate_models_header},
        models_gen::generate_models_block,
    };

    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");
    let temp_dir = temp_output_dir();

    // Generate bindings file
    let mut bindings_parts = Vec::new();
    bindings_parts.push(generate_bindings_header());

    for contract in &contracts {
        let signatures = generate_ffi_signatures(contract).expect("Failed to generate signatures");
        bindings_parts.push(format!(
            "// {} Domain Bindings",
            to_pascal_case(&contract.domain)
        ));
        bindings_parts.push(generate_typedefs_block(&signatures));

        let class_name = format!("{}Bindings", to_pascal_case(&contract.domain));
        bindings_parts.push(format!("class {class_name} {{"));
        bindings_parts.push("  final DynamicLibrary _lib;".to_string());
        bindings_parts.push(format!("  {class_name}(this._lib);"));
        bindings_parts.push(generate_function_pointers_block(&signatures));

        let wrappers = generate_wrapper_functions(contract).expect("Failed to generate wrappers");
        bindings_parts.push(generate_wrappers_block(&wrappers));
        bindings_parts.push("}".to_string());
    }

    let bindings_code = bindings_parts.join("\n\n");
    let bindings_path = temp_dir.path().join("generated_bindings.dart");
    std::fs::write(&bindings_path, &bindings_code).expect("Failed to write bindings file");

    // Generate models file
    let mut models_parts = Vec::new();
    models_parts.push(generate_models_header());

    for contract in &contracts {
        let models = generate_all_models(contract).expect("Failed to generate models");
        if !models.is_empty() {
            models_parts.push(generate_models_block(&models));
        }
    }

    let models_code = models_parts.join("\n\n");
    let models_path = temp_dir.path().join("generated_models.dart");
    std::fs::write(&models_path, &models_code).expect("Failed to write models file");

    // Format the generated files first
    let _ = Command::new("dart")
        .args(["format", bindings_path.to_str().unwrap()])
        .output();
    let _ = Command::new("dart")
        .args(["format", models_path.to_str().unwrap()])
        .output();

    // Run dart analyze on bindings
    let bindings_analyze = Command::new("dart")
        .args(["analyze", bindings_path.to_str().unwrap()])
        .output()
        .expect("Failed to run dart analyze on bindings");

    // Run dart analyze on models
    let models_analyze = Command::new("dart")
        .args(["analyze", models_path.to_str().unwrap()])
        .output()
        .expect("Failed to run dart analyze on models");

    // Check for errors (warnings are acceptable)
    let bindings_stderr = String::from_utf8_lossy(&bindings_analyze.stderr);
    let models_stderr = String::from_utf8_lossy(&models_analyze.stderr);

    // dart analyze returns errors in stderr, check that there are no error-level issues
    let has_bindings_errors =
        bindings_stderr.contains("error •") || bindings_stderr.contains("error -");
    let has_models_errors = models_stderr.contains("error •") || models_stderr.contains("error -");

    if has_bindings_errors {
        eprintln!("Bindings analyze errors:\n{}", bindings_stderr);
        eprintln!("Bindings content:\n{}", bindings_code);
    }

    if has_models_errors {
        eprintln!("Models analyze errors:\n{}", models_stderr);
        eprintln!("Models content:\n{}", models_code);
    }

    assert!(
        !has_bindings_errors,
        "dart analyze found errors in generated bindings"
    );
    assert!(
        !has_models_errors,
        "dart analyze found errors in generated models"
    );
}

#[test]
fn test_generated_code_contains_all_contract_functions() {
    use generate_dart_bindings::bindings_gen::generate_wrappers_block;

    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");

    for contract in &contracts {
        let wrappers = generate_wrapper_functions(contract).expect("Failed to generate wrappers");
        let wrappers_code = generate_wrappers_block(&wrappers);

        // Each function should appear in the generated code (in camelCase format)
        for func in &contract.functions {
            let camel_name = to_camel_case(&func.name);
            assert!(
                wrappers_code.contains(&camel_name),
                "Function '{}' (camelCase: '{}') from domain '{}' not found in generated code",
                func.name,
                camel_name,
                contract.domain
            );
        }
    }
}

#[test]
fn test_generated_typedefs_naming_convention() {
    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");

    for contract in &contracts {
        let signatures = generate_ffi_signatures(contract).expect("Failed to generate signatures");

        for sig in &signatures {
            // Native typedef should contain "_native" in the typedef name
            assert!(
                sig.native_typedef.contains("_native"),
                "Native typedef '{}' should contain '_native'",
                sig.native_typedef
            );

            // Native typedef should start with "typedef _"
            assert!(
                sig.native_typedef.starts_with("typedef _"),
                "Native typedef should start with 'typedef _': {}",
                sig.native_typedef
            );

            // Dart typedef should start with "typedef _" and contain the function name
            assert!(
                sig.dart_typedef.starts_with("typedef _"),
                "Dart typedef should start with 'typedef _': {}",
                sig.dart_typedef
            );

            // Both typedefs should contain "Function("
            assert!(
                sig.native_typedef.contains("Function("),
                "Native typedef should contain 'Function(': {}",
                sig.native_typedef
            );
            assert!(
                sig.dart_typedef.contains("Function("),
                "Dart typedef should contain 'Function(': {}",
                sig.dart_typedef
            );
        }
    }
}

#[test]
fn test_idempotent_generation() {
    use generate_dart_bindings::{
        bindings_gen::{
            generate_function_pointers_block, generate_typedefs_block, generate_wrappers_block,
        },
        header::generate_bindings_header,
    };

    let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load contracts");

    // Generate code twice
    let generate = || {
        let mut parts = Vec::new();
        parts.push(generate_bindings_header());

        for contract in &contracts {
            let signatures =
                generate_ffi_signatures(contract).expect("Failed to generate signatures");
            parts.push(generate_typedefs_block(&signatures));
            parts.push(generate_function_pointers_block(&signatures));

            let wrappers =
                generate_wrapper_functions(contract).expect("Failed to generate wrappers");
            parts.push(generate_wrappers_block(&wrappers));
        }

        // Remove timestamp line for comparison
        parts
            .join("\n")
            .lines()
            .filter(|l| !l.contains("Generation time:"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let first = generate();
    let second = generate();

    assert_eq!(
        first, second,
        "Code generation is not idempotent - outputs differ between runs"
    );
}
