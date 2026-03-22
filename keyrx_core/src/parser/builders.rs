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

/// Build a simple key remap, modifier, or lock mapping based on the `to` prefix.
/// - `VK_` prefix: Simple remap
/// - `MD_` prefix: Modifier
/// - `LK_` prefix: Lock
pub fn build_map(from: &str, to: &str) -> Result<BaseKeyMapping, String> {
    let from_key = parse_physical_key(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

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
    let from_key = parse_physical_key(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;
    Ok(BaseKeyMapping::ModifiedOutput {
        from: from_key,
        to: to_key,
        shift,
        ctrl,
        alt,
        win,
    })
}

/// Build a tap-hold mapping.
pub fn build_tap_hold(
    key: &str,
    tap: &str,
    hold: &str,
    threshold_ms: u16,
) -> Result<BaseKeyMapping, String> {
    let from_key = parse_physical_key(key).map_err(|e| format!("Invalid key: {}", e))?;

    if !tap.starts_with("VK_") {
        return Err(format!(
            "tap_hold tap parameter must have VK_ prefix, got: {}",
            tap
        ));
    }
    let tap_key = parse_virtual_key(tap).map_err(|e| format!("Invalid tap key: {}", e))?;

    if !hold.starts_with("MD_") {
        return Err(format!(
            "tap_hold hold parameter must have MD_ prefix, got: {}",
            hold
        ));
    }
    let hold_modifier =
        parse_modifier_id(hold).map_err(|e| format!("Invalid hold modifier: {}", e))?;

    Ok(BaseKeyMapping::TapHold {
        from: from_key,
        tap: tap_key,
        hold_modifier,
        threshold_ms,
    })
}

/// Build a hold-only mapping (tap suppressed).
pub fn build_hold_only(key: &str, hold: &str, threshold_ms: u16) -> Result<BaseKeyMapping, String> {
    let from_key = parse_physical_key(key).map_err(|e| format!("Invalid key: {}", e))?;

    if !hold.starts_with("MD_") {
        return Err(format!(
            "hold_only hold parameter must have MD_ prefix, got: {}",
            hold
        ));
    }
    let hold_modifier =
        parse_modifier_id(hold).map_err(|e| format!("Invalid hold modifier: {}", e))?;

    Ok(BaseKeyMapping::HoldOnly {
        from: from_key,
        hold_modifier,
        threshold_ms,
    })
}
