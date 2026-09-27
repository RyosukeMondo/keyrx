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
        self.evaluate_condition_for_identities(condition, device_id.as_slice())
    }

    /// Evaluates a condition for an event from a device known by several
    /// identities (e.g. its id, name, path and serial): a `DeviceMatches`
    /// pattern holds if it matches ANY of them (ASCII case-insensitive).
    pub fn evaluate_condition_for_identities(
        &self,
        condition: &Condition,
        identities: &[&str],
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
            Condition::DeviceMatches(pattern) => {
                crate::runtime::device_pattern::matches_any(identities, pattern)
            }

            // IME is active
            Condition::ImeActive => self.is_ime_active(),

            // Input language matches
            Condition::InputLanguage(lang) => self.matches_input_language(lang),
        }
    }

    /// Evaluates a single condition item
    ///
    /// Helper method for evaluating ConditionItem in composite conditions.
    pub(super) fn evaluate_condition_item(&self, item: &ConditionItem) -> bool {
        match item {
            ConditionItem::ModifierActive(id) => self.is_modifier_active(*id),
            ConditionItem::LockActive(id) => self.is_lock_active(*id),
            ConditionItem::ImeActive => self.is_ime_active(),
            ConditionItem::InputLanguage(lang) => self.matches_input_language(lang),
        }
    }
}

/// Input language matching
impl DeviceState {
    /// Matches the current input language against a target language tag.
    ///
    /// Uses BCP 47 prefix matching: "ja" matches "ja", "ja-JP", "ja-Kana".
    /// The match is case-insensitive.
    pub(super) fn matches_input_language(&self, target: &str) -> bool {
        let current = self.input_language();
        if current.is_empty() || target.is_empty() {
            return false;
        }
        let current_lower = current.to_lowercase();
        let target_lower = target.to_lowercase();

        if current_lower == target_lower {
            return true;
        }
        // "ja" matches "ja-JP", "ja-Kana", etc.
        if current_lower.starts_with(&target_lower) {
            return current_lower.as_bytes().get(target_lower.len()) == Some(&b'-');
        }
        if target_lower.starts_with(&current_lower) {
            return target_lower.as_bytes().get(current_lower.len()) == Some(&b'-');
        }
        false
    }
}
