//! Built-in types and helper functions for the Rhai scripting runtime.
//!
//! This module contains:
//! - Pending operation types for deferred registry updates
//! - Layer and modifier preview structures
//! - Timing configuration types
//! - Utility functions for validation and error creation
//! - Parsing functions for arrays and layer actions

use super::helpers::parse_key_or_error;
use crate::config::MAX_TIMEOUT_MS;
use crate::engine::{
    HoldAction, KeyCode, LayerStack, Modifier, ModifierState, TimingConfig, VirtualModifiers,
};
use crate::scripting::sandbox::validation::InputValidator;
use crate::scripting::sandbox::validators::{LengthValidator, NonEmptyValidator, RangeValidator};
use crate::scripting::RemapRegistry;
use rhai::{Array, EvalAltResult, Position};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-safe pending operations storage.
pub type PendingOps = Arc<Mutex<Vec<PendingOp>>>;
pub type LayerView = Arc<Mutex<LayerStack>>;
pub type ModifierView = Arc<Mutex<ModifierPreview>>;

/// Preview state for modifiers during script execution.
#[derive(Debug, Clone)]
pub struct ModifierPreview {
    names: HashMap<String, u8>,
    next_id: u16,
    state: ModifierState,
}

impl ModifierPreview {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            next_id: 0,
            state: ModifierState::new(),
        }
    }

    pub fn define(&mut self, name: &str) -> Result<u8, Box<EvalAltResult>> {
        if let Some(&id) = self.names.get(name) {
            return Ok(id);
        }

        if self.next_id > VirtualModifiers::MAX_ID as u16 {
            return Err(modifier_error(
                "define_modifier",
                format!(
                    "maximum virtual modifiers reached (0-{})",
                    VirtualModifiers::MAX_ID
                ),
            ));
        }

        let id = self.next_id as u8;
        self.next_id = self.next_id.saturating_add(1);
        self.names.insert(name.to_string(), id);
        Ok(id)
    }

    pub fn id_for(&self, name: &str, fn_name: &str) -> Result<u8, Box<EvalAltResult>> {
        self.names
            .get(name)
            .copied()
            .ok_or_else(|| modifier_error(fn_name, format!("modifier '{}' is not defined", name)))
    }

    pub fn activate(&mut self, id: u8) {
        self.state.activate(Modifier::Virtual(id));
    }

    pub fn deactivate(&mut self, id: u8) {
        self.state.deactivate(Modifier::Virtual(id));
    }

    pub fn one_shot(&mut self, id: u8) {
        self.state.arm_one_shot(Modifier::Virtual(id));
    }

    pub fn is_active(&self, id: u8) -> bool {
        self.state.is_active(Modifier::Virtual(id))
    }

    pub fn sync_from_registry(&mut self, registry: &RemapRegistry) {
        self.names = registry.modifier_names().clone();
        self.next_id = registry.next_modifier_id();
        self.state = registry.modifier_state();
    }
}

impl Default for ModifierPreview {
    fn default() -> Self {
        Self::new()
    }
}

/// A pending operation to be applied to the registry after script execution.
#[derive(Debug, Clone)]
pub enum PendingOp {
    Remap {
        from: KeyCode,
        to: KeyCode,
    },
    Block {
        key: KeyCode,
    },
    Pass {
        key: KeyCode,
    },
    TapHold {
        key: KeyCode,
        tap: KeyCode,
        hold: HoldAction,
    },
    Combo {
        keys: Vec<KeyCode>,
        action: crate::engine::LayerAction,
    },
    LayerDefine {
        name: String,
        transparent: bool,
    },
    LayerMap {
        layer: String,
        key: KeyCode,
        action: LayerMapAction,
    },
    LayerPush {
        name: String,
    },
    LayerToggle {
        name: String,
    },
    LayerPop,
    DefineModifier {
        name: String,
        id: u8,
    },
    ModifierActivate {
        name: String,
        id: u8,
    },
    ModifierDeactivate {
        name: String,
        id: u8,
    },
    ModifierOneShot {
        name: String,
        id: u8,
    },
    SetTiming(TimingUpdate),
}

/// Action for layer key mappings.
#[derive(Debug, Clone)]
pub enum LayerMapAction {
    Remap(KeyCode),
    Block,
    Pass,
    TapHold { tap: KeyCode, hold: HoldAction },
    LayerPush(String),
    LayerToggle(String),
    LayerPop,
}

/// Timing configuration update types.
#[derive(Debug, Clone, Copy)]
pub enum TimingUpdate {
    TapTimeout(u32),
    ComboTimeout(u32),
    HoldDelay(u32),
    EagerTap(bool),
    PermissiveHold(bool),
    RetroTap(bool),
}

/// Apply a timing update to a timing configuration.
pub fn apply_timing_update(timing: &mut TimingConfig, update: TimingUpdate) {
    match update {
        TimingUpdate::TapTimeout(ms) => timing.tap_timeout_ms = ms,
        TimingUpdate::ComboTimeout(ms) => timing.combo_timeout_ms = ms,
        TimingUpdate::HoldDelay(ms) => timing.hold_delay_ms = ms,
        TimingUpdate::EagerTap(enabled) => timing.eager_tap = enabled,
        TimingUpdate::PermissiveHold(enabled) => timing.permissive_hold = enabled,
        TimingUpdate::RetroTap(enabled) => timing.retro_tap = enabled,
    }
}

// Error creation helpers

/// Create a timing-related error.
pub fn timing_error(fn_name: &str, message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(
        format!("{}: {}", fn_name, message.into()).into(),
        Position::NONE,
    ))
}

/// Create a layer-related error.
pub fn layer_error(fn_name: &str, message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(
        format!("{}: {}", fn_name, message.into()).into(),
        Position::NONE,
    ))
}

/// Create a modifier-related error.
pub fn modifier_error(fn_name: &str, message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(
        format!("{}: {}", fn_name, message.into()).into(),
        Position::NONE,
    ))
}

/// Create an array parsing error.
pub fn parse_array_error(index: usize, value_type: &str) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(
        format!(
            "combo: keys must be strings, got {} at index {}",
            value_type, index
        )
        .into(),
        Position::NONE,
    ))
}

// Validation helpers

/// Validate a timeout value is within acceptable range.
///
/// Uses the InputValidator trait system for validation.
pub fn validate_timeout(
    ms: i64,
    fn_name: &str,
    allow_zero: bool,
) -> Result<u32, Box<EvalAltResult>> {
    let min = if allow_zero { 0 } else { 1 };

    // Use RangeValidator
    let validator = RangeValidator::new(min, MAX_TIMEOUT_MS);
    validator.validate(&ms).map_err(|e| {
        timing_error(
            fn_name,
            format!("{} expects {}..={} ms: {}", fn_name, min, MAX_TIMEOUT_MS, e),
        )
    })?;

    Ok(ms as u32)
}

/// Normalize and validate a layer name.
///
/// Uses the InputValidator trait system for validation.
pub fn normalize_layer_name(name: &str, fn_name: &str) -> Result<String, Box<EvalAltResult>> {
    let trimmed = name.trim();

    // Validate non-empty
    NonEmptyValidator
        .validate(trimmed)
        .map_err(|e| layer_error(fn_name, format!("layer name {}", e)))?;

    // Validate no colons (custom validation)
    if trimmed.contains(':') {
        return Err(layer_error(fn_name, "layer name cannot contain ':'"));
    }

    // Validate reasonable length
    LengthValidator::new(1, 64)
        .validate(trimmed)
        .map_err(|e| layer_error(fn_name, format!("layer name {}", e)))?;

    Ok(trimmed.to_string())
}

/// Normalize and validate a modifier name.
///
/// Uses the InputValidator trait system for validation.
pub fn normalize_modifier_name(name: &str, fn_name: &str) -> Result<String, Box<EvalAltResult>> {
    let trimmed = name.trim();

    // Validate non-empty
    NonEmptyValidator
        .validate(trimmed)
        .map_err(|e| modifier_error(fn_name, format!("modifier name {}", e)))?;

    // Validate no colons (custom validation)
    if trimmed.contains(':') {
        return Err(modifier_error(fn_name, "modifier name cannot contain ':'"));
    }

    // Validate reasonable length
    LengthValidator::new(1, 64)
        .validate(trimmed)
        .map_err(|e| modifier_error(fn_name, format!("modifier name {}", e)))?;

    Ok(trimmed.to_string())
}

/// Execute a closure with the layer view locked.
pub fn with_layer_view<R, F>(view: &LayerView, f: F) -> Result<R, Box<EvalAltResult>>
where
    F: FnOnce(&mut LayerStack) -> Result<R, Box<EvalAltResult>>,
{
    let mut guard = view
        .lock()
        .map_err(|_| layer_error("layer_view", "failed to lock layer view"))?;
    f(&mut guard)
}

/// Execute a closure with the modifier view locked.
pub fn with_modifier_view<R, F>(view: &ModifierView, f: F) -> Result<R, Box<EvalAltResult>>
where
    F: FnOnce(&mut ModifierPreview) -> Result<R, Box<EvalAltResult>>,
{
    let mut guard = view
        .lock()
        .map_err(|_| modifier_error("modifier_view", "failed to lock modifier view"))?;
    f(&mut guard)
}

/// Ensure a layer exists in the view.
pub fn ensure_layer_exists(
    view: &LayerView,
    name: &str,
    fn_name: &str,
) -> Result<(), Box<EvalAltResult>> {
    with_layer_view(view, |stack| {
        if stack.layer_id_by_name(name).is_none() {
            Err(layer_error(
                fn_name,
                format!("layer '{}' is not defined", name),
            ))
        } else {
            Ok(())
        }
    })
}

// Parsing functions

/// Parse a Rhai array of key names into a vector of KeyCodes.
pub fn parse_keys_array(keys: Array) -> Result<Vec<KeyCode>, Box<EvalAltResult>> {
    let mut parsed = Vec::with_capacity(keys.len());

    for (idx, value) in keys.into_iter().enumerate() {
        let key_name = value
            .clone()
            .try_cast::<String>()
            .ok_or_else(|| parse_array_error(idx, value.type_name()))?;

        let key_code = parse_key_or_error(&key_name, "combo")?;
        parsed.push(key_code);
    }

    Ok(parsed)
}

/// Parse a layer action string into a LayerMapAction.
pub fn parse_layer_action(
    action: &str,
    fn_name: &str,
    view: &LayerView,
) -> Result<LayerMapAction, Box<EvalAltResult>> {
    let trimmed = action.trim();
    if trimmed.is_empty() {
        return Err(layer_error(fn_name, "action cannot be empty"));
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower == "block" {
        return Ok(LayerMapAction::Block);
    }
    if lower == "pass" {
        return Ok(LayerMapAction::Pass);
    }
    if lower == "layer_pop" {
        return Ok(LayerMapAction::LayerPop);
    }

    if lower.starts_with("remap:") {
        let target = trimmed["remap:".len()..].trim();
        let key = parse_key_or_error(target, fn_name)?;
        return Ok(LayerMapAction::Remap(key));
    }

    if lower.starts_with("layer_push:") {
        let target = trimmed["layer_push:".len()..].trim();
        let normalized = normalize_layer_name(target, fn_name)?;
        ensure_layer_exists(view, &normalized, fn_name)?;
        return Ok(LayerMapAction::LayerPush(normalized));
    }

    if lower.starts_with("layer_toggle:") {
        let target = trimmed["layer_toggle:".len()..].trim();
        let normalized = normalize_layer_name(target, fn_name)?;
        ensure_layer_exists(view, &normalized, fn_name)?;
        return Ok(LayerMapAction::LayerToggle(normalized));
    }

    if lower.starts_with("tap_hold:") {
        let parts: Vec<&str> = trimmed["tap_hold:".len()..].split(':').collect();
        if parts.len() != 2 {
            return Err(layer_error(
                fn_name,
                "tap_hold action requires tap and hold values",
            ));
        }
        let tap = parse_key_or_error(parts[0].trim(), fn_name)?;
        let hold = parse_key_or_error(parts[1].trim(), fn_name)?;
        return Ok(LayerMapAction::TapHold {
            tap,
            hold: HoldAction::Key(hold),
        });
    }

    if lower.starts_with("tap_hold_mod:") {
        let parts: Vec<&str> = trimmed["tap_hold_mod:".len()..].split(':').collect();
        if parts.len() != 2 {
            return Err(layer_error(
                fn_name,
                "tap_hold_mod action requires tap and modifier id",
            ));
        }
        let tap = parse_key_or_error(parts[0].trim(), fn_name)?;
        let modifier_id = parts[1].trim().parse::<u8>().map_err(|_| {
            layer_error(
                fn_name,
                format!(
                    "tap_hold_mod modifier id must be 0-{}, got '{}'",
                    VirtualModifiers::MAX_ID,
                    parts[1].trim()
                ),
            )
        })?;
        return Ok(LayerMapAction::TapHold {
            tap,
            hold: HoldAction::Modifier(modifier_id),
        });
    }

    // Default: treat as key remap.
    let remap_target = parse_key_or_error(trimmed, fn_name)?;
    Ok(LayerMapAction::Remap(remap_target))
}
