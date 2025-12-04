//! Capability registry for O(1) function capability lookup.
//!
//! This module provides a registry that maps function names and KeyCodes to their
//! capability tiers, enabling fast security checks during script execution.

use super::capability::{ScriptCapability, ScriptMode};
use crate::drivers::keycodes::KeyCode;
use std::collections::HashMap;

/// Function capability information.
///
/// Associates a function with its capability tier and metadata.
#[derive(Debug, Clone)]
pub struct FunctionCapability {
    /// Function name
    pub name: String,
    /// Required capability tier
    pub capability: ScriptCapability,
    /// Human-readable description
    pub description: String,
    /// Optional KeyCode this function is associated with
    pub keycode: Option<KeyCode>,
}

impl FunctionCapability {
    /// Create a new function capability.
    pub fn new(
        name: impl Into<String>,
        capability: ScriptCapability,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            capability,
            description: description.into(),
            keycode: None,
        }
    }

    /// Set the associated KeyCode.
    pub fn with_keycode(mut self, keycode: KeyCode) -> Self {
        self.keycode = Some(keycode);
        self
    }

    /// Check if this function is allowed in the given mode.
    #[inline]
    pub fn is_allowed_in(&self, mode: ScriptMode) -> bool {
        self.capability.is_allowed_in(mode)
    }
}

/// Registry mapping functions to capabilities with O(1) lookup.
///
/// The registry maintains two indices:
/// - By function name: for checking function calls
/// - By KeyCode: for discovering KeyCode-related functions
///
/// # Performance
///
/// All lookups are O(1) using HashMap-based indexing.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::registry::{CapabilityRegistry, FunctionCapability};
/// use keyrx_core::scripting::sandbox::capability::{ScriptCapability, ScriptMode};
/// use keyrx_core::drivers::keycodes::KeyCode;
///
/// let mut registry = CapabilityRegistry::new();
///
/// // Register a function
/// registry.register(FunctionCapability::new(
///     "send_key",
///     ScriptCapability::Standard,
///     "Send a key event"
/// ));
///
/// // Check if function is allowed
/// assert!(registry.is_allowed("send_key", ScriptMode::Standard));
/// assert!(!registry.is_allowed("send_key", ScriptMode::Safe));
///
/// // Get function capability
/// assert!(registry.get("send_key").is_some());
/// ```
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    /// Function name to capability mapping (O(1) lookup)
    by_name: HashMap<String, FunctionCapability>,
    /// KeyCode to functions mapping (O(1) lookup)
    by_keycode: HashMap<KeyCode, Vec<String>>,
}

impl CapabilityRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
            by_keycode: HashMap::new(),
        }
    }

    /// Create a new registry with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            by_name: HashMap::with_capacity(capacity),
            by_keycode: HashMap::new(),
        }
    }

    /// Register a function with its capability.
    ///
    /// If a function with the same name already exists, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::registry::{CapabilityRegistry, FunctionCapability};
    /// use keyrx_core::scripting::sandbox::capability::ScriptCapability;
    /// use keyrx_core::drivers::keycodes::KeyCode;
    ///
    /// let mut registry = CapabilityRegistry::new();
    ///
    /// registry.register(FunctionCapability::new(
    ///     "send_key",
    ///     ScriptCapability::Standard,
    ///     "Send a key event"
    /// ).with_keycode(KeyCode::A));
    /// ```
    pub fn register(&mut self, cap: FunctionCapability) {
        let name = cap.name.clone();

        // Index by KeyCode if present
        if let Some(keycode) = cap.keycode {
            self.by_keycode
                .entry(keycode)
                .or_default()
                .push(name.clone());
        }

        // Index by name
        self.by_name.insert(name, cap);
    }

    /// Get capability for a function by name - O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::registry::{CapabilityRegistry, FunctionCapability};
    /// use keyrx_core::scripting::sandbox::capability::ScriptCapability;
    ///
    /// let mut registry = CapabilityRegistry::new();
    /// registry.register(FunctionCapability::new(
    ///     "send_key",
    ///     ScriptCapability::Standard,
    ///     "Send a key event"
    /// ));
    ///
    /// assert!(registry.get("send_key").is_some());
    /// assert!(registry.get("unknown").is_none());
    /// ```
    #[inline]
    pub fn get(&self, name: &str) -> Option<&FunctionCapability> {
        self.by_name.get(name)
    }

    /// Get functions for a KeyCode - O(1).
    ///
    /// Returns function names associated with the given KeyCode.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::registry::{CapabilityRegistry, FunctionCapability};
    /// use keyrx_core::scripting::sandbox::capability::ScriptCapability;
    /// use keyrx_core::drivers::keycodes::KeyCode;
    ///
    /// let mut registry = CapabilityRegistry::new();
    /// registry.register(FunctionCapability::new(
    ///     "key_a",
    ///     ScriptCapability::Safe,
    ///     "KeyCode A constant"
    /// ).with_keycode(KeyCode::A));
    ///
    /// let funcs = registry.for_keycode(KeyCode::A);
    /// assert_eq!(funcs.len(), 1);
    /// ```
    pub fn for_keycode(&self, key: KeyCode) -> Vec<&FunctionCapability> {
        self.by_keycode
            .get(&key)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.by_name.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a function is allowed in the given mode - O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::registry::{CapabilityRegistry, FunctionCapability};
    /// use keyrx_core::scripting::sandbox::capability::{ScriptCapability, ScriptMode};
    ///
    /// let mut registry = CapabilityRegistry::new();
    /// registry.register(FunctionCapability::new(
    ///     "send_key",
    ///     ScriptCapability::Standard,
    ///     "Send a key event"
    /// ));
    ///
    /// assert!(registry.is_allowed("send_key", ScriptMode::Standard));
    /// assert!(!registry.is_allowed("send_key", ScriptMode::Safe));
    /// assert!(!registry.is_allowed("unknown", ScriptMode::Full));
    /// ```
    #[inline]
    pub fn is_allowed(&self, name: &str, mode: ScriptMode) -> bool {
        self.by_name
            .get(name)
            .map(|cap| cap.is_allowed_in(mode))
            .unwrap_or(false)
    }

    /// Get all functions for a capability tier.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::registry::{CapabilityRegistry, FunctionCapability};
    /// use keyrx_core::scripting::sandbox::capability::ScriptCapability;
    ///
    /// let mut registry = CapabilityRegistry::new();
    /// registry.register(FunctionCapability::new(
    ///     "add",
    ///     ScriptCapability::Safe,
    ///     "Add numbers"
    /// ));
    /// registry.register(FunctionCapability::new(
    ///     "send_key",
    ///     ScriptCapability::Standard,
    ///     "Send a key event"
    /// ));
    ///
    /// let safe_funcs = registry.by_tier(ScriptCapability::Safe);
    /// assert_eq!(safe_funcs.len(), 1);
    /// assert_eq!(safe_funcs[0].name, "add");
    /// ```
    pub fn by_tier(&self, tier: ScriptCapability) -> Vec<&FunctionCapability> {
        self.by_name
            .values()
            .filter(|cap| cap.capability == tier)
            .collect()
    }

    /// Get all functions allowed in the given mode.
    pub fn by_mode(&self, mode: ScriptMode) -> Vec<&FunctionCapability> {
        self.by_name
            .values()
            .filter(|cap| cap.is_allowed_in(mode))
            .collect()
    }

    /// Get the number of registered functions.
    #[inline]
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Check if the registry is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Clear all registered functions.
    pub fn clear(&mut self) {
        self.by_name.clear();
        self.by_keycode.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry() {
        let registry = CapabilityRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = CapabilityRegistry::new();

        registry.register(FunctionCapability::new(
            "test_func",
            ScriptCapability::Safe,
            "Test function",
        ));

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let cap = registry.get("test_func").unwrap();
        assert_eq!(cap.name, "test_func");
        assert_eq!(cap.capability, ScriptCapability::Safe);
        assert_eq!(cap.description, "Test function");
    }

    #[test]
    fn test_register_with_keycode() {
        let mut registry = CapabilityRegistry::new();

        registry.register(
            FunctionCapability::new("key_a", ScriptCapability::Safe, "KeyCode A constant")
                .with_keycode(KeyCode::A),
        );

        let funcs = registry.for_keycode(KeyCode::A);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "key_a");
    }

    #[test]
    fn test_for_keycode_empty() {
        let registry = CapabilityRegistry::new();
        let funcs = registry.for_keycode(KeyCode::A);
        assert_eq!(funcs.len(), 0);
    }

    #[test]
    fn test_multiple_functions_per_keycode() {
        let mut registry = CapabilityRegistry::new();

        registry.register(
            FunctionCapability::new("key_a", ScriptCapability::Safe, "KeyCode A constant")
                .with_keycode(KeyCode::A),
        );
        registry.register(
            FunctionCapability::new("send_a", ScriptCapability::Standard, "Send A key")
                .with_keycode(KeyCode::A),
        );

        let funcs = registry.for_keycode(KeyCode::A);
        assert_eq!(funcs.len(), 2);
    }

    #[test]
    fn test_is_allowed() {
        let mut registry = CapabilityRegistry::new();

        registry.register(FunctionCapability::new(
            "safe_func",
            ScriptCapability::Safe,
            "Safe function",
        ));
        registry.register(FunctionCapability::new(
            "std_func",
            ScriptCapability::Standard,
            "Standard function",
        ));
        registry.register(FunctionCapability::new(
            "adv_func",
            ScriptCapability::Advanced,
            "Advanced function",
        ));

        // Safe mode
        assert!(registry.is_allowed("safe_func", ScriptMode::Safe));
        assert!(!registry.is_allowed("std_func", ScriptMode::Safe));
        assert!(!registry.is_allowed("adv_func", ScriptMode::Safe));

        // Standard mode
        assert!(registry.is_allowed("safe_func", ScriptMode::Standard));
        assert!(registry.is_allowed("std_func", ScriptMode::Standard));
        assert!(!registry.is_allowed("adv_func", ScriptMode::Standard));

        // Full mode
        assert!(registry.is_allowed("safe_func", ScriptMode::Full));
        assert!(registry.is_allowed("std_func", ScriptMode::Full));
        assert!(registry.is_allowed("adv_func", ScriptMode::Full));

        // Unknown function
        assert!(!registry.is_allowed("unknown", ScriptMode::Full));
    }

    #[test]
    fn test_by_tier() {
        let mut registry = CapabilityRegistry::new();

        registry.register(FunctionCapability::new(
            "safe1",
            ScriptCapability::Safe,
            "Safe function 1",
        ));
        registry.register(FunctionCapability::new(
            "safe2",
            ScriptCapability::Safe,
            "Safe function 2",
        ));
        registry.register(FunctionCapability::new(
            "std1",
            ScriptCapability::Standard,
            "Standard function",
        ));

        let safe_funcs = registry.by_tier(ScriptCapability::Safe);
        assert_eq!(safe_funcs.len(), 2);

        let std_funcs = registry.by_tier(ScriptCapability::Standard);
        assert_eq!(std_funcs.len(), 1);

        let adv_funcs = registry.by_tier(ScriptCapability::Advanced);
        assert_eq!(adv_funcs.len(), 0);
    }

    #[test]
    fn test_by_mode() {
        let mut registry = CapabilityRegistry::new();

        registry.register(FunctionCapability::new(
            "safe_func",
            ScriptCapability::Safe,
            "Safe function",
        ));
        registry.register(FunctionCapability::new(
            "std_func",
            ScriptCapability::Standard,
            "Standard function",
        ));
        registry.register(FunctionCapability::new(
            "adv_func",
            ScriptCapability::Advanced,
            "Advanced function",
        ));

        let safe_mode = registry.by_mode(ScriptMode::Safe);
        assert_eq!(safe_mode.len(), 1);

        let std_mode = registry.by_mode(ScriptMode::Standard);
        assert_eq!(std_mode.len(), 2);

        let full_mode = registry.by_mode(ScriptMode::Full);
        assert_eq!(full_mode.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut registry = CapabilityRegistry::new();

        registry.register(FunctionCapability::new(
            "test",
            ScriptCapability::Safe,
            "Test",
        ));
        assert_eq!(registry.len(), 1);

        registry.clear();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let registry = CapabilityRegistry::with_capacity(100);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_replace_existing() {
        let mut registry = CapabilityRegistry::new();

        registry.register(FunctionCapability::new(
            "test",
            ScriptCapability::Safe,
            "Original",
        ));
        registry.register(FunctionCapability::new(
            "test",
            ScriptCapability::Standard,
            "Replaced",
        ));

        assert_eq!(registry.len(), 1);
        let cap = registry.get("test").unwrap();
        assert_eq!(cap.capability, ScriptCapability::Standard);
        assert_eq!(cap.description, "Replaced");
    }
}
