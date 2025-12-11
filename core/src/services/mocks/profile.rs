//! Mock implementation of ProfileServiceTrait for testing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config::models::{HardwareProfile, Keymap, VirtualLayout};
use crate::services::profile::ProfileServiceError;
use crate::services::traits::ProfileServiceTrait;

/// Mock implementation of [`ProfileServiceTrait`] for testing.
///
/// Provides configurable responses and call tracking for all profile operations.
/// All operations are pure in-memory with no I/O.
///
/// This mock supports actual CRUD operations in memory:
/// - `save_*` methods add or update items in internal collections
/// - `delete_*` methods remove items from collections
/// - `list_*` methods return the current collection state
///
/// # Example
///
/// ```rust,ignore
/// let mock = MockProfileService::new()
///     .with_virtual_layouts(vec![test_layout("layout-1")]);
///
/// // List returns configured data
/// let layouts = mock.list_virtual_layouts().unwrap();
/// assert_eq!(layouts.len(), 1);
///
/// // Save adds to the collection
/// mock.save_virtual_layout(test_layout("layout-2")).unwrap();
/// let layouts = mock.list_virtual_layouts().unwrap();
/// assert_eq!(layouts.len(), 2);
///
/// // Delete removes from the collection
/// mock.delete_virtual_layout("layout-1").unwrap();
/// let layouts = mock.list_virtual_layouts().unwrap();
/// assert_eq!(layouts.len(), 1);
/// ```
pub struct MockProfileService {
    /// Virtual layouts to store and return
    virtual_layouts: Arc<Mutex<Vec<VirtualLayout>>>,
    /// Hardware profiles to store and return
    hardware_profiles: Arc<Mutex<Vec<HardwareProfile>>>,
    /// Keymaps to store and return
    keymaps: Arc<Mutex<Vec<Keymap>>>,
    /// Error to return from list_virtual_layouts
    list_layouts_error: Option<String>,
    /// Error to return from save_virtual_layout
    save_layout_error: Option<String>,
    /// Error to return from delete_virtual_layout
    delete_layout_error: Option<String>,
    /// Error to return from list_hardware_profiles
    list_profiles_error: Option<String>,
    /// Error to return from save_hardware_profile
    save_profile_error: Option<String>,
    /// Error to return from delete_hardware_profile
    delete_profile_error: Option<String>,
    /// Error to return from list_keymaps
    list_keymaps_error: Option<String>,
    /// Error to return from save_keymap
    save_keymap_error: Option<String>,
    /// Error to return from delete_keymap
    delete_keymap_error: Option<String>,
    /// Tracks method call counts for verification
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MockProfileService {
    /// Creates a new empty MockProfileService.
    pub fn new() -> Self {
        Self {
            virtual_layouts: Arc::new(Mutex::new(Vec::new())),
            hardware_profiles: Arc::new(Mutex::new(Vec::new())),
            keymaps: Arc::new(Mutex::new(Vec::new())),
            list_layouts_error: None,
            save_layout_error: None,
            delete_layout_error: None,
            list_profiles_error: None,
            save_profile_error: None,
            delete_profile_error: None,
            list_keymaps_error: None,
            save_keymap_error: None,
            delete_keymap_error: None,
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configures the virtual layouts to return.
    pub fn with_virtual_layouts(self, layouts: Vec<VirtualLayout>) -> Self {
        *self.virtual_layouts.lock().unwrap() = layouts;
        self
    }

    /// Configures the hardware profiles to return.
    pub fn with_hardware_profiles(self, profiles: Vec<HardwareProfile>) -> Self {
        *self.hardware_profiles.lock().unwrap() = profiles;
        self
    }

    /// Configures the keymaps to return.
    pub fn with_keymaps(self, keymaps: Vec<Keymap>) -> Self {
        *self.keymaps.lock().unwrap() = keymaps;
        self
    }

    /// Configures an error to return from list_virtual_layouts.
    pub fn with_list_layouts_error(mut self, error: &str) -> Self {
        self.list_layouts_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from save_virtual_layout.
    pub fn with_save_layout_error(mut self, error: &str) -> Self {
        self.save_layout_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from delete_virtual_layout.
    pub fn with_delete_layout_error(mut self, error: &str) -> Self {
        self.delete_layout_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from list_hardware_profiles.
    pub fn with_list_profiles_error(mut self, error: &str) -> Self {
        self.list_profiles_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from save_hardware_profile.
    pub fn with_save_profile_error(mut self, error: &str) -> Self {
        self.save_profile_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from delete_hardware_profile.
    pub fn with_delete_profile_error(mut self, error: &str) -> Self {
        self.delete_profile_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from list_keymaps.
    pub fn with_list_keymaps_error(mut self, error: &str) -> Self {
        self.list_keymaps_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from save_keymap.
    pub fn with_save_keymap_error(mut self, error: &str) -> Self {
        self.save_keymap_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from delete_keymap.
    pub fn with_delete_keymap_error(mut self, error: &str) -> Self {
        self.delete_keymap_error = Some(error.to_string());
        self
    }

    /// Returns the number of times a method was called.
    pub fn get_call_count(&self, method: &str) -> usize {
        self.call_counts
            .lock()
            .unwrap()
            .get(method)
            .copied()
            .unwrap_or(0)
    }

    fn increment_call(&self, method: &str) {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
}

impl Default for MockProfileService {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create profile errors from strings
fn make_profile_error(msg: &str) -> ProfileServiceError {
    ProfileServiceError::NotFound(msg.to_string())
}

impl ProfileServiceTrait for MockProfileService {
    fn list_virtual_layouts(&self) -> Result<Vec<VirtualLayout>, ProfileServiceError> {
        self.increment_call("list_virtual_layouts");
        if let Some(ref error) = self.list_layouts_error {
            return Err(make_profile_error(error));
        }
        Ok(self.virtual_layouts.lock().unwrap().clone())
    }

    fn save_virtual_layout(
        &self,
        layout: VirtualLayout,
    ) -> Result<VirtualLayout, ProfileServiceError> {
        self.increment_call("save_virtual_layout");
        if let Some(ref error) = self.save_layout_error {
            return Err(make_profile_error(error));
        }
        let mut layouts = self.virtual_layouts.lock().unwrap();
        // Update or add
        if let Some(existing) = layouts.iter_mut().find(|l| l.id == layout.id) {
            *existing = layout.clone();
        } else {
            layouts.push(layout.clone());
        }
        Ok(layout)
    }

    fn delete_virtual_layout(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.increment_call("delete_virtual_layout");
        if let Some(ref error) = self.delete_layout_error {
            return Err(make_profile_error(error));
        }
        let mut layouts = self.virtual_layouts.lock().unwrap();
        layouts.retain(|l| l.id != id);
        Ok(())
    }

    fn list_hardware_profiles(&self) -> Result<Vec<HardwareProfile>, ProfileServiceError> {
        self.increment_call("list_hardware_profiles");
        if let Some(ref error) = self.list_profiles_error {
            return Err(make_profile_error(error));
        }
        Ok(self.hardware_profiles.lock().unwrap().clone())
    }

    fn save_hardware_profile(
        &self,
        profile: HardwareProfile,
    ) -> Result<HardwareProfile, ProfileServiceError> {
        self.increment_call("save_hardware_profile");
        if let Some(ref error) = self.save_profile_error {
            return Err(make_profile_error(error));
        }
        let mut profiles = self.hardware_profiles.lock().unwrap();
        if let Some(existing) = profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile.clone();
        } else {
            profiles.push(profile.clone());
        }
        Ok(profile)
    }

    fn delete_hardware_profile(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.increment_call("delete_hardware_profile");
        if let Some(ref error) = self.delete_profile_error {
            return Err(make_profile_error(error));
        }
        let mut profiles = self.hardware_profiles.lock().unwrap();
        profiles.retain(|p| p.id != id);
        Ok(())
    }

    fn list_keymaps(&self) -> Result<Vec<Keymap>, ProfileServiceError> {
        self.increment_call("list_keymaps");
        if let Some(ref error) = self.list_keymaps_error {
            return Err(make_profile_error(error));
        }
        Ok(self.keymaps.lock().unwrap().clone())
    }

    fn save_keymap(&self, keymap: Keymap) -> Result<Keymap, ProfileServiceError> {
        self.increment_call("save_keymap");
        if let Some(ref error) = self.save_keymap_error {
            return Err(make_profile_error(error));
        }
        let mut keymaps = self.keymaps.lock().unwrap();
        if let Some(existing) = keymaps.iter_mut().find(|k| k.id == keymap.id) {
            *existing = keymap.clone();
        } else {
            keymaps.push(keymap.clone());
        }
        Ok(keymap)
    }

    fn delete_keymap(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.increment_call("delete_keymap");
        if let Some(ref error) = self.delete_keymap_error {
            return Err(make_profile_error(error));
        }
        let mut keymaps = self.keymaps.lock().unwrap();
        keymaps.retain(|k| k.id != id);
        Ok(())
    }
}
