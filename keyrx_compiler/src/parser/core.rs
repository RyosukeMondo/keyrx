use rhai::{Engine, EvalAltResult, NativeCallContext, Scope};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::error::ParseError;
use keyrx_core::config::{ConfigRoot, DeviceConfig, Metadata, Version};

use keyrx_core::config::{BaseKeyMapping, Condition, KeyMapping};
use keyrx_core::parser::scopes::{Line, MappingScopes};

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

        engine.set_max_operations(10_000);
        engine.set_max_expr_depths(100, 100);
        engine.set_max_call_levels(100);

        crate::parser::functions::map::register_map_function(&mut engine, Arc::clone(&state));
        crate::parser::functions::tap_hold::register_tap_hold_function(
            &mut engine,
            Arc::clone(&state),
        );
        crate::parser::functions::hold_only::register_hold_only_function(
            &mut engine,
            Arc::clone(&state),
        );
        crate::parser::functions::conditional::register_when_functions(
            &mut engine,
            Arc::clone(&state),
        );
        crate::parser::functions::sequence::register_sequence_function(
            &mut engine,
            Arc::clone(&state),
        );
        crate::parser::functions::modifiers::register_modifier_functions(&mut engine);
        crate::parser::functions::device::register_device_function(&mut engine, Arc::clone(&state));
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

        let script = std::fs::read_to_string(path).map_err(|_e| ParseError::ImportNotFound {
            path: path.to_path_buf(),
            searched_paths: vec![path.to_path_buf()],
            import_chain: Vec::new(),
        })?;

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
            return Err(ParseError::SyntaxError {
                file: source_path.to_path_buf(),
                line: 0,
                column: 0,
                message: "Unclosed device() block".to_string(),
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
            message: err.to_string(),
            import_chain: Vec::new(),
        }
    }
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
