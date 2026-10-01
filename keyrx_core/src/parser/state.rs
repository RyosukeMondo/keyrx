//! Parser state shared across Rhai custom functions.

use crate::config::{BaseKeyMapping, Condition, DeviceConfig};
use crate::parser::scopes::{Line, MappingScopes};
use alloc::boxed::Box;
use alloc::vec::Vec;
use rhai::{EvalAltResult, NativeCallContext};

/// Parser state shared across Rhai custom functions.
#[derive(Debug, Clone, Default)]
pub struct ParserState {
    /// Collected device configurations
    pub devices: Vec<DeviceConfig>,
    /// Current device being configured (between device_start and device_end)
    pub current_device: Option<DeviceConfig>,
    /// Stack of (Condition, mappings) pairs being collected for conditional blocks
    /// When non-empty, map() adds to the top of this stack instead of current_device
    pub conditional_stack: Vec<(Condition, Vec<BaseKeyMapping>)>,
    /// Duplicate-key / unclosed-block bookkeeping (see [`MappingScopes`]).
    pub scopes: MappingScopes,
}

/// The source line of the DSL call being run, when Rhai knows it.
pub fn call_line(ctx: &NativeCallContext) -> Line {
    ctx.call_position().line()
}

impl ParserState {
    /// Create a new empty parser state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `mapping` to the innermost open scope: the current conditional
    /// block, else the current device. `func` names the DSL function for
    /// the error when no `device_start` is open. A source key already mapped
    /// in that scope is an error (see [`MappingScopes::record`]).
    pub fn push_mapping(
        &mut self,
        mapping: BaseKeyMapping,
        func: &str,
        line: Line,
    ) -> Result<(), Box<EvalAltResult>> {
        if self.current_device.is_none() {
            return Err(
                alloc::format!("{}() must be called inside a device_start() block", func).into(),
            );
        }
        self.scopes.record(mapping.source_key(), line)?;
        if let Some((_condition, mappings)) = self.conditional_stack.last_mut() {
            mappings.push(mapping);
        } else if let Some(device) = self.current_device.as_mut() {
            device
                .mappings
                .push(crate::config::KeyMapping::Base(mapping));
        }
        Ok(())
    }
}
