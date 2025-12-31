//! Daemon state management for keyrx.
//!
//! This module provides state management utilities including:
//!
//! - [`ReloadState`]: Tracks configuration reload requests
//! - Configuration conversion helpers for rkyv-archived types

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};

// Import the archived types from their modules
use keyrx_core::config::conditions::{ArchivedCondition, ArchivedConditionItem};
use keyrx_core::config::keys::ArchivedKeyCode;
use keyrx_core::config::mappings::{
    ArchivedBaseKeyMapping, ArchivedDeviceConfig, ArchivedKeyMapping,
};

/// Reload request state.
///
/// This struct tracks whether a configuration reload has been requested
/// (typically via SIGHUP signal).
#[derive(Debug, Clone)]
pub struct ReloadState {
    /// Flag indicating a reload has been requested.
    reload_requested: Arc<AtomicBool>,
}

impl ReloadState {
    /// Creates a new reload state.
    pub fn new() -> Self {
        Self {
            reload_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the underlying atomic flag for signal handler registration.
    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.reload_requested)
    }

    /// Checks if a reload has been requested and clears the flag.
    ///
    /// Returns `true` if a reload was requested since the last check.
    pub fn check_and_clear(&self) -> bool {
        self.reload_requested.swap(false, Ordering::SeqCst)
    }

    /// Requests a reload (for testing purposes).
    #[cfg(test)]
    pub fn request_reload(&self) {
        self.reload_requested.store(true, Ordering::SeqCst);
    }
}

impl Default for ReloadState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Archived Config Conversion Helpers
// ============================================================================
//
// These functions convert rkyv-archived configuration types to owned types.
// This is necessary because DeviceManager and KeyLookup operate on owned types.

/// Converts an archived KeyCode to an owned KeyCode.
///
/// Uses rkyv's Deserialize trait for safe conversion.
#[allow(dead_code)]
pub(crate) fn convert_archived_keycode(archived: &ArchivedKeyCode) -> KeyCode {
    use rkyv::Deserialize;
    archived
        .deserialize(&mut rkyv::Infallible)
        .expect("KeyCode deserialization is infallible")
}

/// Converts an archived ConditionItem to an owned ConditionItem.
#[allow(dead_code)]
pub(crate) fn convert_archived_condition_item(archived: &ArchivedConditionItem) -> ConditionItem {
    match archived {
        ArchivedConditionItem::ModifierActive(id) => ConditionItem::ModifierActive(*id),
        ArchivedConditionItem::LockActive(id) => ConditionItem::LockActive(*id),
    }
}

/// Converts an archived Condition to an owned Condition.
#[allow(dead_code)]
pub(crate) fn convert_archived_condition(archived: &ArchivedCondition) -> Condition {
    match archived {
        ArchivedCondition::ModifierActive(id) => Condition::ModifierActive(*id),
        ArchivedCondition::LockActive(id) => Condition::LockActive(*id),
        ArchivedCondition::DeviceMatches(id) => {
            use rkyv::Deserialize;
            Condition::DeviceMatches(Deserialize::deserialize(id, &mut rkyv::Infallible).unwrap())
        }
        ArchivedCondition::AllActive(items) => {
            Condition::AllActive(items.iter().map(convert_archived_condition_item).collect())
        }
        ArchivedCondition::NotActive(items) => {
            Condition::NotActive(items.iter().map(convert_archived_condition_item).collect())
        }
    }
}

/// Converts an archived BaseKeyMapping to an owned BaseKeyMapping.
#[allow(dead_code)]
pub(crate) fn convert_archived_base_mapping(archived: &ArchivedBaseKeyMapping) -> BaseKeyMapping {
    match archived {
        ArchivedBaseKeyMapping::Simple { from, to } => BaseKeyMapping::Simple {
            from: convert_archived_keycode(from),
            to: convert_archived_keycode(to),
        },
        ArchivedBaseKeyMapping::Modifier { from, modifier_id } => BaseKeyMapping::Modifier {
            from: convert_archived_keycode(from),
            modifier_id: *modifier_id,
        },
        ArchivedBaseKeyMapping::Lock { from, lock_id } => BaseKeyMapping::Lock {
            from: convert_archived_keycode(from),
            lock_id: *lock_id,
        },
        ArchivedBaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        } => BaseKeyMapping::TapHold {
            from: convert_archived_keycode(from),
            tap: convert_archived_keycode(tap),
            hold_modifier: *hold_modifier,
            threshold_ms: *threshold_ms,
        },
        ArchivedBaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        } => BaseKeyMapping::ModifiedOutput {
            from: convert_archived_keycode(from),
            to: convert_archived_keycode(to),
            shift: *shift,
            ctrl: *ctrl,
            alt: *alt,
            win: *win,
        },
    }
}

/// Converts an archived KeyMapping to an owned KeyMapping.
#[allow(dead_code)]
pub(crate) fn convert_archived_key_mapping(archived: &ArchivedKeyMapping) -> KeyMapping {
    match archived {
        ArchivedKeyMapping::Base(base) => KeyMapping::Base(convert_archived_base_mapping(base)),
        ArchivedKeyMapping::Conditional {
            condition,
            mappings,
        } => KeyMapping::Conditional {
            condition: convert_archived_condition(condition),
            mappings: mappings.iter().map(convert_archived_base_mapping).collect(),
        },
    }
}

/// Converts an archived DeviceConfig to an owned DeviceConfig.
#[allow(dead_code)]
pub(crate) fn convert_archived_device_config(archived: &ArchivedDeviceConfig) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: archived.identifier.pattern.to_string(),
        },
        mappings: archived
            .mappings
            .iter()
            .map(convert_archived_key_mapping)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reload_state_new() {
        let state = ReloadState::new();
        assert!(!state.check_and_clear());
    }

    #[test]
    fn test_reload_state_check_and_clear() {
        let state = ReloadState::new();

        // Initially no reload requested
        assert!(!state.check_and_clear());

        // Request reload
        state.request_reload();

        // Check and clear should return true once
        assert!(state.check_and_clear());

        // Subsequent checks should return false
        assert!(!state.check_and_clear());
    }

    #[test]
    fn test_reload_state_flag_sharing() {
        let state = ReloadState::new();
        let flag = state.flag();

        // Set flag via external reference
        flag.store(true, Ordering::SeqCst);

        // Should be detectable via check_and_clear
        assert!(state.check_and_clear());
    }

    #[test]
    fn test_reload_state_default() {
        let state = ReloadState::default();
        assert!(!state.check_and_clear());
    }
}
