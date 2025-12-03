//! Build script for keyrx_core
//!
//! Handles build-time tasks:
//! - Tracks error source files to trigger recompilation
//! - Tracks FFI source files to trigger Dart binding regeneration
//!
//! Note: Documentation generation happens via `just docs-errors` command.
//! Note: Dart binding generation happens via `just gen-bindings` command.

use std::path::Path;

fn main() {
    // Track error source files
    let error_files = [
        "src/errors/mod.rs",
        "src/errors/code.rs",
        "src/errors/category.rs",
        "src/errors/definition.rs",
        "src/errors/error.rs",
        "src/errors/registry.rs",
        "src/errors/macros.rs",
        "src/errors/config.rs",
        "src/errors/runtime.rs",
        "src/errors/driver.rs",
        "src/errors/validation.rs",
        "src/errors/ffi.rs",
        "src/errors/internal.rs",
        "src/errors/doc_generator.rs",
    ];

    for file in &error_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    // Track FFI source files for binding generation
    track_ffi_files();

    // Track the doc generator binary
    println!("cargo:rerun-if-changed=src/bin/generate_error_docs.rs");
}

/// Track FFI source files to trigger binding regeneration when they change
fn track_ffi_files() {
    // Track old-style exports files
    if let Ok(entries) = std::fs::read_dir("src/ffi") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("exports_") && name.ends_with(".rs") {
                    println!("cargo:rerun-if-changed=src/ffi/{}", name);
                }
            }
        }
    }

    // Track new-style domain files
    let domains_dir = Path::new("src/ffi/domains");
    if domains_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(domains_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".rs") && name != "mod.rs" {
                        println!("cargo:rerun-if-changed=src/ffi/domains/{}", name);
                    }
                }
            }
        }
    }

    // Track the binding generator script itself
    println!("cargo:rerun-if-changed=../scripts/generate_dart_bindings.py");
}
