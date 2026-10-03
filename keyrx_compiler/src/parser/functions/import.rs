use rhai::{Engine, EvalAltResult, ImmutableString};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use keyrx_core::parser::limits;

use crate::error::ParseError;
use crate::import_resolver::ImportResolver;
use crate::parser::core::{apply_limits, canonical_or_same, read_source, with_state, ParserState};

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
            // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
            #[allow(clippy::unwrap_used)]
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

            let canonical = canonical_or_same(&resolved_path);
            let runtime = |message: String| {
                Box::new(EvalAltResult::ErrorRuntime(
                    message.into(),
                    rhai::Position::NONE,
                ))
            };
            // Each import evaluates in its own engine on the Rust stack: a
            // file that loads itself (directly or via others) would overflow
            // it and abort the whole process, so refuse cycles and cap depth.
            enter_import(&import_state, canonical).map_err(runtime)?;
            let imported_script = read_source(&resolved_path).map_err(|e| {
                leave_import(&import_state);
                runtime(e.to_string())
            })?;

            // Create a new engine instance that shares the same state
            let mut import_engine = Engine::new();
            apply_limits(&mut import_engine);

            // Register all the same functions
            crate::parser::functions::register_dsl(&mut import_engine, &import_state);

            // Register import function recursively
            register_import_function(
                &mut import_engine,
                Arc::clone(&import_state),
                Arc::new(Mutex::new(resolved_path.clone())),
            );

            // Execute the imported script
            let outcome = import_engine.run(&imported_script);
            leave_import(&import_state);
            outcome.map_err(|e| {
                runtime(format!(
                    "Error executing imported file {}: {}",
                    resolved_path.display(),
                    e
                ))
            })?;

            Ok(())
        },
    );
}

/// Pushes `file` on the import stack, or says why it must not be loaded: it
/// is already being loaded (a cycle) or the nesting is too deep.
fn enter_import(state: &Arc<Mutex<ParserState>>, file: PathBuf) -> Result<(), String> {
    with_state(state, |state| {
        let stack = &mut state.import_stack;
        if let Some(start) = stack.iter().position(|p| *p == file) {
            let chain = stack[start..]
                .iter()
                .cloned()
                .chain(std::iter::once(file))
                .collect();
            return Err(ParseError::CircularImport { chain }.to_string());
        }
        if stack.len() >= limits::MAX_IMPORT_DEPTH {
            return Err(format!(
                "load() nested more than {} files deep at {}",
                limits::MAX_IMPORT_DEPTH,
                file.display()
            ));
        }
        stack.push(file);
        Ok(())
    })
}

fn leave_import(state: &Arc<Mutex<ParserState>>) {
    with_state(state, |state| {
        state.import_stack.pop();
    });
}
