//! Shared mapping builder functions for DSL parsers.
//!
//! These pure functions are the SSOT for mapping creation logic.
//! Both keyrx_core's WASM parser and keyrx_compiler's std parser
//! call these functions, eliminating code duplication.

use crate::config::keys::KeyCode;
use crate::config::BaseKeyMapping;
use crate::parser::validators::{
    parse_lock_id, parse_modifier_id, parse_physical_key, parse_virtual_key,
};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Maximum number of keys in a sequence mapping
pub const MAX_SEQUENCE_LENGTH: usize = 8;

/// Parses the physical key a mapping consumes (what the user presses).
///
/// THE key-naming rule: INPUT keys are bare names (`"CapsLock"`; a `VK_`
/// prefix is tolerated and means the same key), OUTPUT keys must carry
/// `VK_`, and `MD_`/`LK_` name custom modifier/lock states that can only be
/// outputs. `label` names the argument in the error.
fn input_key(label: &str, s: &str) -> Result<KeyCode, String> {
    if s.starts_with("MD_") || s.starts_with("LK_") {
        return Err(format!(
            "Invalid {}: '{}' is a custom modifier/lock, which can only be an output; \
             the key you press must be a physical key name such as \"CapsLock\"",
            label, s
        ));
    }
    parse_physical_key(s).map_err(|e| format!("Invalid {}: {}", label, e))
}

/// Build a simple key remap, modifier, or lock mapping based on the `to` prefix.
/// - `VK_` prefix: Simple remap
/// - `MD_` prefix: Modifier
/// - `LK_` prefix: Lock
pub fn build_map(from: &str, to: &str) -> Result<BaseKeyMapping, String> {
    let from_key = input_key("'from' key", from)?;

    if to.starts_with("VK_") {
        let to_key = parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;
        Ok(BaseKeyMapping::Simple {
            from: from_key,
            to: to_key,
        })
    } else if to.starts_with("MD_") {
        let modifier_id =
            parse_modifier_id(to).map_err(|e| format!("Invalid modifier ID: {}", e))?;
        Ok(BaseKeyMapping::Modifier {
            from: from_key,
            modifier_id,
        })
    } else if to.starts_with("LK_") {
        let lock_id = parse_lock_id(to).map_err(|e| format!("Invalid lock ID: {}", e))?;
        Ok(BaseKeyMapping::Lock {
            from: from_key,
            lock_id,
        })
    } else {
        Err(format!(
            "Output must have VK_, MD_, or LK_ prefix: {} -> use VK_{} for virtual key",
            to, to
        ))
    }
}

/// Build a modified output mapping (key with Shift/Ctrl/Alt/Win modifiers).
pub fn build_modified_map(
    from: &str,
    to_key: KeyCode,
    shift: bool,
    ctrl: bool,
    alt: bool,
    win: bool,
) -> Result<BaseKeyMapping, String> {
    let from_key = input_key("'from' key", from)?;
    Ok(BaseKeyMapping::ModifiedOutput {
        from: from_key,
        to: to_key,
        shift,
        ctrl,
        alt,
        win,
    })
}

/// What a tap-hold/hold-only key does while held.
enum HoldTarget {
    /// `MD_xx`: activate a custom modifier (layer).
    Modifier(u8),
    /// `VK_xx`: press this real key (e.g. `VK_LCtrl`) for the duration.
    Key(KeyCode),
}

fn parse_hold_target(func: &str, hold: &str) -> Result<HoldTarget, String> {
    if hold.starts_with("MD_") {
        let id = parse_modifier_id(hold).map_err(|e| format!("Invalid hold modifier: {}", e))?;
        Ok(HoldTarget::Modifier(id))
    } else if hold.starts_with("VK_") {
        let key = parse_virtual_key(hold).map_err(|e| format!("Invalid hold key: {}", e))?;
        Ok(HoldTarget::Key(key))
    } else {
        Err(format!(
            "{} hold parameter must start with MD_ (custom modifier/layer) or VK_ \
             (a real key such as VK_LCtrl), got: {}",
            func, hold
        ))
    }
}

/// Build a tap-hold mapping. `hold` is `MD_xx` (custom modifier) or `VK_xx`
/// (a real key pressed while held, e.g. `VK_LCtrl`).
pub fn build_tap_hold(
    key: &str,
    tap: &str,
    hold: &str,
    threshold_ms: u16,
) -> Result<BaseKeyMapping, String> {
    let from_key = input_key("key", key)?;

    if !tap.starts_with("VK_") {
        return Err(format!(
            "tap_hold tap parameter must have VK_ prefix, got: {}",
            tap
        ));
    }
    let tap_key = parse_virtual_key(tap).map_err(|e| format!("Invalid tap key: {}", e))?;

    Ok(match parse_hold_target("tap_hold", hold)? {
        HoldTarget::Modifier(hold_modifier) => BaseKeyMapping::TapHold {
            from: from_key,
            tap: tap_key,
            hold_modifier,
            threshold_ms,
        },
        HoldTarget::Key(hold) => BaseKeyMapping::TapHoldKey {
            from: from_key,
            tap: Some(tap_key),
            hold,
            threshold_ms,
        },
    })
}

/// Build a hold-only mapping (tap suppressed). `hold` is `MD_xx` or `VK_xx`
/// as for [`build_tap_hold`].
pub fn build_hold_only(key: &str, hold: &str, threshold_ms: u16) -> Result<BaseKeyMapping, String> {
    let from_key = input_key("key", key)?;

    Ok(match parse_hold_target("hold_only", hold)? {
        HoldTarget::Modifier(hold_modifier) => BaseKeyMapping::HoldOnly {
            from: from_key,
            hold_modifier,
            threshold_ms,
        },
        HoldTarget::Key(hold) => BaseKeyMapping::TapHoldKey {
            from: from_key,
            tap: None,
            hold,
            threshold_ms,
        },
    })
}

/// Build a sequence mapping (one key → multiple keys typed in order).
pub fn build_sequence(key: &str, output_keys: &[String]) -> Result<BaseKeyMapping, String> {
    let from_key = input_key("key", key)?;

    if output_keys.is_empty() {
        return Err("Sequence must have at least one output key".into());
    }
    if output_keys.len() > MAX_SEQUENCE_LENGTH {
        return Err(format!(
            "Sequence too long: {} keys (max {})",
            output_keys.len(),
            MAX_SEQUENCE_LENGTH
        ));
    }

    let mut keys = Vec::new();
    for (i, key_str) in output_keys.iter().enumerate() {
        let parsed =
            parse_virtual_key(key_str).map_err(|e| format!("Invalid sequence key {}: {}", i, e))?;
        keys.push(parsed);
    }

    Ok(BaseKeyMapping::Sequence {
        from: from_key,
        keys,
    })
}
