//! Build script for keyrx_core
//!
//! Tracks error source files to trigger recompilation when they change.
//! Actual documentation generation happens via the `just docs-errors` command.

fn main() {
    // Only regenerate docs when error source files change
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

    // Tell cargo to rerun this script if any error files change
    for file in &error_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    // Also rerun if the doc generator binary changes
    println!("cargo:rerun-if-changed=src/bin/generate_error_docs.rs");

    // Note: We don't generate docs here to avoid circular dependencies.
    // Instead, run `just docs-errors` or `just check` to regenerate docs.
}
