//! Condition evaluation logic
//!
//! Evaluates conditions against device state, including modifier/lock checks,
//! composite conditions (AND/NOT), and device pattern matching.

extern crate alloc;

use super::core::DeviceState;
use crate::config::{Condition, ConditionItem};

impl DeviceState {
    /// Evaluates a condition against the current device state
    ///
    /// This is a convenience method that calls `evaluate_condition_with_device`
    /// with `device_id = None`. Use this for conditions that don't involve
    /// device matching (ModifierActive, LockActive, AllActive, NotActive).
    ///
    /// Note: DeviceMatches conditions will always return false when called
    /// without a device_id. Use `evaluate_condition_with_device` for those.
    ///
    /// # Arguments
    ///
    /// * `condition` - The condition to evaluate
    ///
    /// # Returns
    ///
    /// Returns `true` if the condition is satisfied, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use keyrx_core::runtime::DeviceState;
    /// use keyrx_core::config::{Condition, ConditionItem};
    ///
    /// let mut state = DeviceState::new();
    /// state.set_modifier(0);
    ///
    /// // Single modifier active
    /// assert!(state.evaluate_condition(&Condition::ModifierActive(0)));
    ///
    /// // All conditions must be true
    /// state.toggle_lock(1);
    /// let all_cond = Condition::AllActive(vec![
    ///     ConditionItem::ModifierActive(0),
    ///     ConditionItem::LockActive(1),
    /// ]);
    /// assert!(state.evaluate_condition(&all_cond));
    ///
    /// // Not active
    /// let not_cond = Condition::NotActive(vec![ConditionItem::ModifierActive(2)]);
    /// assert!(state.evaluate_condition(&not_cond)); // MD_02 is not active
    /// ```
    pub fn evaluate_condition(&self, condition: &Condition) -> bool {
        self.evaluate_condition_with_device(condition, None)
    }

    /// Evaluates a condition against the current device state and optional device ID
    ///
    /// This is the full version of condition evaluation that supports device matching.
    /// For conditions that don't involve device matching, you can use `evaluate_condition()`.
    ///
    /// # Arguments
    ///
    /// * `condition` - The condition to evaluate
    /// * `device_id` - Optional device ID from the current event
    ///
    /// # Returns
    ///
    /// Returns `true` if the condition is satisfied, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use keyrx_core::runtime::DeviceState;
    /// use keyrx_core::config::Condition;
    ///
    /// let state = DeviceState::new();
    ///
    /// // Device matching condition
    /// let cond = Condition::DeviceMatches("numpad".to_string());
    /// assert!(state.evaluate_condition_with_device(&cond, Some("numpad")));
    /// assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));
    /// assert!(!state.evaluate_condition_with_device(&cond, None));
    /// ```
    pub fn evaluate_condition_with_device(
        &self,
        condition: &Condition,
        device_id: Option<&str>,
    ) -> bool {
        match condition {
            // Single modifier active
            Condition::ModifierActive(id) => self.is_modifier_active(*id),

            // Single lock active
            Condition::LockActive(id) => self.is_lock_active(*id),

            // All conditions must be true (AND logic)
            Condition::AllActive(items) => {
                items.iter().all(|item| self.evaluate_condition_item(item))
            }

            // All conditions must be false (NOT logic)
            Condition::NotActive(items) => {
                items.iter().all(|item| !self.evaluate_condition_item(item))
            }

            // Device ID matches pattern
            Condition::DeviceMatches(pattern) => Self::matches_device_pattern(device_id, pattern),
        }
    }

    /// Evaluates a single condition item
    ///
    /// Helper method for evaluating ConditionItem in composite conditions.
    pub(super) fn evaluate_condition_item(&self, item: &ConditionItem) -> bool {
        match item {
            ConditionItem::ModifierActive(id) => self.is_modifier_active(*id),
            ConditionItem::LockActive(id) => self.is_lock_active(*id),
        }
    }
}

/// Device pattern matching module
impl DeviceState {
    /// Matches a device ID against a pattern
    ///
    /// Supports simple glob patterns with `*` wildcard:
    /// - Exact match: "device-123" matches only "device-123"
    /// - Prefix: "usb-*" matches "usb-keyboard", "usb-numpad", etc.
    /// - Suffix: "*-keyboard" matches "usb-keyboard", "bt-keyboard", etc.
    /// - Contains: "*numpad*" matches "usb-numpad-123", "my-numpad", etc.
    ///
    /// Returns false if device_id is None.
    pub(super) fn matches_device_pattern(device_id: Option<&str>, pattern: &str) -> bool {
        let Some(id) = device_id else {
            return false;
        };

        // Handle glob patterns with *
        if pattern.contains('*') {
            let parts: alloc::vec::Vec<&str> = pattern.split('*').collect();
            match parts.len() {
                1 => {
                    // No actual * (shouldn't happen but handle it)
                    id == pattern
                }
                2 => {
                    // Single * - either prefix, suffix, or empty on one side
                    let (prefix, suffix) = (parts[0], parts[1]);
                    if prefix.is_empty() && suffix.is_empty() {
                        // Pattern is just "*" - matches everything
                        true
                    } else if prefix.is_empty() {
                        // *suffix
                        id.ends_with(suffix)
                    } else if suffix.is_empty() {
                        // prefix*
                        id.starts_with(prefix)
                    } else {
                        // prefix*suffix
                        id.starts_with(prefix) && id.ends_with(suffix)
                    }
                }
                3 => {
                    // Two *s - typically *contains*
                    let (prefix, middle, suffix) = (parts[0], parts[1], parts[2]);
                    if prefix.is_empty() && suffix.is_empty() {
                        // *middle*
                        id.contains(middle)
                    } else {
                        // More complex pattern - do simple check
                        id.starts_with(prefix) && id.ends_with(suffix) && id.contains(middle)
                    }
                }
                _ => {
                    // Complex pattern with multiple * - just check if all parts exist in order
                    // This is a simplified implementation
                    let mut remaining = id;
                    for (i, part) in parts.iter().enumerate() {
                        if part.is_empty() {
                            continue;
                        }
                        if i == 0 {
                            // First part must be prefix
                            if !remaining.starts_with(part) {
                                return false;
                            }
                            remaining = &remaining[part.len()..];
                        } else if i == parts.len() - 1 {
                            // Last part must be suffix
                            if !remaining.ends_with(part) {
                                return false;
                            }
                        } else {
                            // Middle parts must exist somewhere
                            if let Some(pos) = remaining.find(part) {
                                remaining = &remaining[pos + part.len()..];
                            } else {
                                return false;
                            }
                        }
                    }
                    true
                }
            }
        } else {
            // Exact match
            id == pattern
        }
    }
}
