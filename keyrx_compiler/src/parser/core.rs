use rhai::{Engine, EvalAltResult, NativeCallContext, Scope};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::error::ParseError;
use keyrx_core::config::{ConfigRoot, DeviceConfig, Metadata, Version};

use keyrx_core::config::{BaseKeyMapping, Condition, KeyMapping};
use keyrx_core::parser::scopes::{Line, MappingScopes};
use keyrx_core::parser::usage;

/// Parser state shared across Rhai custom functions
#[derive(Debug, Clone, Default)]
pub struct ParserState {
    pub devices: Vec<DeviceConfig>,
    pub current_device: Option<DeviceConfig>,
    /// Stack of (Condition, mappings) pairs being collected for conditional blocks
    /// When non-empty, map() adds to the top of this stack instead of current_device
    pub conditional_stack: Vec<(Condition, Vec<BaseKeyMapping>)>,
    /// Duplicate-key / unclosed-block bookkeeping (shared with keyrx_core's parser)
    pub scopes: MappingScopes,
    /// Files being evaluated right now, outermost first (the main script,
    /// then each `load()` below it). Detects import cycles and bounds depth.
    pub import_stack: Vec<PathBuf>,
}

impl ParserState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `mapping` to the innermost open scope: the current conditional
    /// block, else the current device. `func` names the DSL function for the
    /// error when no `device_start` is open. A source key already mapped in
    /// that scope is an error naming both lines.
    pub fn push_mapping(
        &mut self,
        mapping: BaseKeyMapping,
        func: &str,
        line: Line,
    ) -> Result<(), Box<EvalAltResult>> {
        if self.current_device.is_none() {
            return Err(format!("{}() must be called inside a device_start() block", func).into());
        }
        self.scopes.record(mapping.source_key(), line)?;
        if let Some((_condition, mappings)) = self.conditional_stack.last_mut() {
            mappings.push(mapping);
        } else if let Some(device) = self.current_device.as_mut() {
            device.mappings.push(KeyMapping::Base(mapping));
        }
        Ok(())
    }
}

/// Runs `f` on the shared parser state (a poisoned lock still holds valid
/// state: no function panics while holding it).
pub fn with_state<T>(state: &Arc<Mutex<ParserState>>, f: impl FnOnce(&mut ParserState) -> T) -> T {
    let mut guard = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

/// The source line of the DSL call being run, when Rhai knows it.
pub fn call_line(ctx: &NativeCallContext) -> Line {
    ctx.call_position().line()
}

/// Applies the shared resource limits (`keyrx_core::parser::limits`) to an
/// engine. The main script and every `load()`ed file use this one function.
pub fn apply_limits(engine: &mut Engine) {
    use keyrx_core::parser::limits;
    engine.set_max_operations(limits::MAX_OPERATIONS);
    engine.set_max_expr_depths(limits::MAX_EXPR_DEPTH, limits::MAX_EXPR_DEPTH);
    engine.set_max_call_levels(limits::MAX_CALL_LEVELS);
}

/// Reads a script: bounded size, valid UTF-8, no byte-order mark. The one
/// reader for the main file and imports, so every failure says what is wrong
/// with the file instead of pretending it was not found.
pub fn read_source(path: &Path) -> Result<String, ParseError> {
    use keyrx_core::parser::limits;
    let unreadable = |reason: String| ParseError::SourceUnreadable {
        path: path.to_path_buf(),
        reason,
    };
    let metadata = std::fs::metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ParseError::ImportNotFound {
                path: path.to_path_buf(),
                searched_paths: vec![path.to_path_buf()],
                import_chain: Vec::new(),
            }
        } else {
            unreadable(e.to_string())
        }
    })?;
    if !metadata.is_file() {
        return Err(unreadable("not a regular file".to_string()));
    }
    if metadata.len() > limits::MAX_SOURCE_BYTES {
        return Err(unreadable(format!(
            "{} bytes is over the {} byte limit for a script",
            metadata.len(),
            limits::MAX_SOURCE_BYTES
        )));
    }
    let bytes = std::fs::read(path).map_err(|e| unreadable(e.to_string()))?;
    let text = String::from_utf8(bytes).map_err(|e| {
        unreadable(format!(
            "not valid UTF-8 (invalid byte at offset {})",
            e.utf8_error().valid_up_to()
        ))
    })?;
    Ok(limits::strip_bom(&text).to_string())
}

/// `path` made absolute and symlink-free when possible, so the same file
/// reached two ways is recognised as one.
pub fn canonical_or_same(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Main parser for Rhai DSL
pub struct Parser {
    pub engine: Engine,
    pub state: Arc<Mutex<ParserState>>,
    /// Current source file being parsed (for import resolution)
    source_file: Arc<Mutex<PathBuf>>,
}

impl Parser {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let state = Arc::new(Mutex::new(ParserState::new()));
        let source_file = Arc::new(Mutex::new(PathBuf::new()));

        apply_limits(&mut engine);

        crate::parser::functions::register_dsl(&mut engine, &state);
        crate::parser::functions::import::register_import_function(
            &mut engine,
            Arc::clone(&state),
            Arc::clone(&source_file),
        );

        Self {
            engine,
            state,
            source_file,
        }
    }

    pub fn parse_script(&mut self, path: &Path) -> Result<ConfigRoot, ParseError> {
        // Update the source file path for import resolution
        // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
        #[allow(clippy::unwrap_used)]
        {
            *self.source_file.lock().unwrap() = path.to_path_buf();
        }

        let script = read_source(path)?;
        with_state(&self.state, |state| {
            state.import_stack = vec![canonical_or_same(path)];
        });

        self.parse_string(&script, path)
    }

    pub fn parse_string(
        &mut self,
        script: &str,
        source_path: &Path,
    ) -> Result<ConfigRoot, ParseError> {
        let start_time = SystemTime::now();

        let mut scope = Scope::new();
        self.engine
            .run_with_scope(&mut scope, script)
            .map_err(|e| Self::convert_rhai_error(e, source_path))?;

        self.validate_timeout(start_time)?;

        // Hash the script content for traceability
        let source_bytes = script.as_bytes();
        self.finalize_config(source_path, source_bytes)
    }

    fn validate_timeout(&self, start_time: SystemTime) -> Result<(), ParseError> {
        let timeout = Duration::from_secs(10);
        if SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::ZERO)
            > timeout
        {
            return Err(ParseError::ResourceLimitExceeded {
                limit_type: "execution timeout (10 seconds)".to_string(),
                import_chain: Vec::new(),
            });
        }
        Ok(())
    }

    fn finalize_config(
        &self,
        source_path: &Path,
        source_bytes: &[u8],
    ) -> Result<ConfigRoot, ParseError> {
        // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
        #[allow(clippy::unwrap_used)]
        let state = self.state.lock().unwrap();
        if state.current_device.is_some() {
            let message =
                state.scopes.check_device_closed().err().unwrap_or_else(|| {
                    "device_start() is never closed: add device_end();".to_string()
                });
            return Err(ParseError::SyntaxError {
                file: source_path.to_path_buf(),
                line: state.scopes.open_device_line().flatten().unwrap_or(0),
                column: 1,
                message,
                import_chain: Vec::new(),
            });
        }

        if let Err(message) = state.scopes.check_all_closed() {
            return Err(ParseError::SyntaxError {
                file: source_path.to_path_buf(),
                line: 0,
                column: 0,
                message,
                import_chain: Vec::new(),
            });
        }

        // Calculate SHA256 hash of source script for traceability
        let mut hasher = Sha256::new();
        hasher.update(source_bytes);
        let hash_result = hasher.finalize();
        let source_hash = hex::encode(hash_result);

        let compilation_timestamp = build_timestamp();

        let metadata = Metadata {
            compilation_timestamp,
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            source_hash,
        };

        Ok(ConfigRoot {
            version: Version::current(),
            devices: state.devices.clone(),
            metadata,
        })
    }

    fn convert_rhai_error(err: Box<EvalAltResult>, path: &Path) -> ParseError {
        let position = err.position();
        ParseError::SyntaxError {
            file: path.to_path_buf(),
            line: position.line().unwrap_or(0),
            column: position.position().unwrap_or(0),
            message: friendly_message(&err),
            import_chain: Vec::new(),
        }
    }
}

/// An evaluation error as a user-facing message: a wrong-arity call to a
/// DSL function says what the function needs, and Rhai's own prefixes and
/// position suffix (reported separately) are dropped.
fn friendly_message(err: &EvalAltResult) -> String {
    if let EvalAltResult::ErrorFunctionNotFound(signature, _) = err {
        if let Some(message) = usage::explain_function_not_found(signature) {
            return message;
        }
    }
    usage::clean_rhai_message(&err.to_string())
}

/// Timestamp embedded in the .krx metadata.
///
/// Output must be byte-identical for identical input, so the wall clock is
/// never used. Honours the reproducible-builds convention `SOURCE_DATE_EPOCH`
/// and is otherwise `0`.
pub(crate) fn build_timestamp() -> u64 {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
