use rhai::{Engine, EvalAltResult, Scope};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::ParseError;
use keyrx_core::config::{ConfigRoot, DeviceConfig, Metadata, Version};

use keyrx_core::config::{BaseKeyMapping, Condition};

/// Parser state shared across Rhai custom functions
#[derive(Debug, Clone, Default)]
pub struct ParserState {
    pub devices: Vec<DeviceConfig>,
    pub current_device: Option<DeviceConfig>,
    /// Stack of (Condition, mappings) pairs being collected for conditional blocks
    /// When non-empty, map() adds to the top of this stack instead of current_device
    pub conditional_stack: Vec<(Condition, Vec<BaseKeyMapping>)>,
}

impl ParserState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Main parser for Rhai DSL
pub struct Parser {
    pub engine: Engine,
    pub state: Arc<Mutex<ParserState>>,
}

impl Parser {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let state = Arc::new(Mutex::new(ParserState::new()));

        engine.set_max_operations(10_000);
        engine.set_max_expr_depths(100, 100);
        engine.set_max_call_levels(100);

        crate::parser::functions::map::register_map_function(&mut engine, Arc::clone(&state));
        crate::parser::functions::tap_hold::register_tap_hold_function(
            &mut engine,
            Arc::clone(&state),
        );
        crate::parser::functions::conditional::register_when_functions(
            &mut engine,
            Arc::clone(&state),
        );
        crate::parser::functions::modifiers::register_modifier_functions(&mut engine);
        crate::parser::functions::device::register_device_function(&mut engine, Arc::clone(&state));

        Self { engine, state }
    }

    pub fn parse_script(&mut self, path: &Path) -> Result<ConfigRoot, ParseError> {
        let script = std::fs::read_to_string(path).map_err(|_e| ParseError::ImportNotFound {
            path: path.to_path_buf(),
            searched_paths: vec![path.to_path_buf()],
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
        self.finalize_config(source_path)
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
            });
        }
        Ok(())
    }

    fn finalize_config(&self, source_path: &Path) -> Result<ConfigRoot, ParseError> {
        let state = self.state.lock().unwrap();
        if state.current_device.is_some() {
            return Err(ParseError::SyntaxError {
                file: source_path.to_path_buf(),
                line: 0,
                column: 0,
                message: "Unclosed device() block".to_string(),
            });
        }

        let metadata = Metadata {
            compilation_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            source_hash: "TODO".to_string(),
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
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
