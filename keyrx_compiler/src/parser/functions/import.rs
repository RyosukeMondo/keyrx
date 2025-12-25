use rhai::{Engine, EvalAltResult, ImmutableString};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::import_resolver::ImportResolver;
use crate::parser::core::ParserState;

/// Registers the load() function in the Rhai engine.
///
/// The load() function allows importing external Rhai files at runtime.
/// When called within a conditional or device block, the imported file's
/// mappings inherit that context.
///
/// # Example
/// ```rhai
/// when_start("MD_00");
///     load("shift.rhai");  // All mappings in shift.rhai apply to MD_00
/// when_end();
/// ```
pub fn register_import_function(
    engine: &mut Engine,
    state: Arc<Mutex<ParserState>>,
    source_file: Arc<Mutex<PathBuf>>,
) {
    let import_state = Arc::clone(&state);
    let import_source = Arc::clone(&source_file);

    engine.register_fn(
        "load",
        move |import_path: ImmutableString| -> Result<(), Box<EvalAltResult>> {
            // Lock the source file path to get the current file's directory
            let source_path = import_source.lock().unwrap();
            let current_dir = source_path.parent().ok_or_else(|| {
                Box::new(EvalAltResult::ErrorRuntime(
                    format!(
                        "Cannot determine directory of source file: {}",
                        source_path.display()
                    )
                    .into(),
                    rhai::Position::NONE,
                ))
            })?;

            // Resolve the import path
            let resolver = ImportResolver::new();
            let resolved_path = resolver
                .resolve_path_from_dir(&import_path, current_dir)
                .map_err(|e| {
                    Box::new(EvalAltResult::ErrorRuntime(
                        format!("Import failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))
                })?;

            // Read the imported file
            let imported_script = std::fs::read_to_string(&resolved_path).map_err(|e| {
                Box::new(EvalAltResult::ErrorRuntime(
                    format!(
                        "Failed to read imported file {}: {}",
                        resolved_path.display(),
                        e
                    )
                    .into(),
                    rhai::Position::NONE,
                ))
            })?;

            // Create a new engine instance that shares the same state
            let mut import_engine = Engine::new();
            import_engine.set_max_operations(10_000);
            import_engine.set_max_expr_depths(100, 100);
            import_engine.set_max_call_levels(100);

            // Register all the same functions
            crate::parser::functions::map::register_map_function(
                &mut import_engine,
                Arc::clone(&import_state),
            );
            crate::parser::functions::tap_hold::register_tap_hold_function(
                &mut import_engine,
                Arc::clone(&import_state),
            );
            crate::parser::functions::conditional::register_when_functions(
                &mut import_engine,
                Arc::clone(&import_state),
            );
            crate::parser::functions::modifiers::register_modifier_functions(&mut import_engine);
            crate::parser::functions::device::register_device_function(
                &mut import_engine,
                Arc::clone(&import_state),
            );

            // Register import function recursively
            register_import_function(
                &mut import_engine,
                Arc::clone(&import_state),
                Arc::new(Mutex::new(resolved_path.clone())),
            );

            // Execute the imported script
            import_engine.run(&imported_script).map_err(|e| {
                Box::new(EvalAltResult::ErrorRuntime(
                    format!(
                        "Error executing imported file {}: {}",
                        resolved_path.display(),
                        e
                    )
                    .into(),
                    rhai::Position::NONE,
                ))
            })?;

            Ok(())
        },
    );
}
