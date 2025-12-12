use super::models::{
    HardwareProfile, HardwareProfileId, Keymap, KeymapId, RuntimeConfig, VirtualLayout,
    VirtualLayoutId,
};
use super::paths::config_dir;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// File-system backed storage for layout, wiring, and keymap resources.
///
/// Data is persisted as JSON inside the KeyRx config directory:
/// - `~/.config/keyrx/layouts/*.json`
/// - `~/.config/keyrx/hardware/*.json`
/// - `~/.config/keyrx/keymaps/*.json`
/// - `~/.config/keyrx/runtime.json`
#[derive(Debug, Clone)]
pub struct ConfigManager {
    root: PathBuf,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self { root: config_dir() }
    }
}

impl ConfigManager {
    /// Create a manager anchored at a specific root (useful for tests).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the root configuration directory path.
    pub fn root_path(&self) -> &Path {
        &self.root
    }

    /// Load all persisted resources from disk.
    pub fn load_all(&self) -> Result<StoredResources, StorageError> {
        self.ensure_directories()?;
        Ok(StoredResources {
            layouts: self.load_virtual_layouts()?,
            hardware_profiles: self.load_hardware_profiles()?,
            keymaps: self.load_keymaps()?,
            runtime: self.load_runtime_config()?,
        })
    }

    /// Save or update a virtual layout.
    pub fn save_virtual_layout(&self, layout: &VirtualLayout) -> Result<PathBuf, StorageError> {
        self.ensure_directories()?;
        let path = self.layouts_dir().join(json_name(&layout.id));
        write_json(&path, layout)?;
        Ok(path)
    }

    /// Save or update a hardware profile.
    pub fn save_hardware_profile(
        &self,
        profile: &HardwareProfile,
    ) -> Result<PathBuf, StorageError> {
        self.ensure_directories()?;
        let path = self.hardware_dir().join(json_name(&profile.id));
        write_json(&path, profile)?;
        Ok(path)
    }

    /// Save or update a keymap.
    pub fn save_keymap(&self, keymap: &Keymap) -> Result<PathBuf, StorageError> {
        self.ensure_directories()?;
        let path = self.keymaps_dir().join(json_name(&keymap.id));
        write_json(&path, keymap)?;
        Ok(path)
    }

    /// Persist runtime configuration (`runtime.json`).
    pub fn save_runtime_config(&self, runtime: &RuntimeConfig) -> Result<PathBuf, StorageError> {
        self.ensure_directories()?;
        let path = self.runtime_config_path();
        write_json(&path, runtime)?;
        Ok(path)
    }

    /// Load all virtual layouts from `layouts/`. Returns empty map if none exist.
    pub fn load_virtual_layouts(
        &self,
    ) -> Result<HashMap<VirtualLayoutId, VirtualLayout>, StorageError> {
        self.load_collection(&self.layouts_dir(), |layout: &VirtualLayout| {
            layout.id.clone()
        })
    }

    /// Load all hardware profiles from `hardware/`. Returns empty map if none exist.
    pub fn load_hardware_profiles(
        &self,
    ) -> Result<HashMap<HardwareProfileId, HardwareProfile>, StorageError> {
        self.load_collection(&self.hardware_dir(), |profile: &HardwareProfile| {
            profile.id.clone()
        })
    }

    /// Load all keymaps from `keymaps/`. Returns empty map if none exist.
    pub fn load_keymaps(&self) -> Result<HashMap<KeymapId, Keymap>, StorageError> {
        self.load_collection(&self.keymaps_dir(), |keymap: &Keymap| keymap.id.clone())
    }

    /// Load runtime configuration; returns empty configuration if the file is absent.
    pub fn load_runtime_config(&self) -> Result<RuntimeConfig, StorageError> {
        let path = self.runtime_config_path();
        if !path.exists() {
            return Ok(RuntimeConfig::default());
        }
        read_json(&path)
    }

    /// Remove a stored virtual layout by id.
    pub fn delete_virtual_layout(&self, id: &str) -> Result<(), StorageError> {
        self.delete_file(&self.layouts_dir().join(json_name(id)))
    }

    /// Remove a stored hardware profile by id.
    pub fn delete_hardware_profile(&self, id: &str) -> Result<(), StorageError> {
        self.delete_file(&self.hardware_dir().join(json_name(id)))
    }

    /// Remove a stored keymap by id.
    pub fn delete_keymap(&self, id: &str) -> Result<(), StorageError> {
        self.delete_file(&self.keymaps_dir().join(json_name(id)))
    }

    fn load_collection<T, K>(
        &self,
        dir: &Path,
        key_fn: impl Fn(&T) -> K,
    ) -> Result<HashMap<K, T>, StorageError>
    where
        T: DeserializeOwned,
        K: std::cmp::Eq + std::hash::Hash,
    {
        self.ensure_directories()?;
        if !dir.exists() {
            return Ok(HashMap::new());
        }

        let mut items = HashMap::new();
        for entry in fs::read_dir(dir).map_err(|e| StorageError::ReadDir(dir.to_path_buf(), e))? {
            let entry = entry.map_err(|e| StorageError::ReadDir(dir.to_path_buf(), e))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let value: T = read_json(&path)?;
            items.insert(key_fn(&value), value);
        }

        Ok(items)
    }

    fn ensure_directories(&self) -> Result<(), StorageError> {
        for dir in [
            self.root.clone(),
            self.layouts_dir(),
            self.hardware_dir(),
            self.keymaps_dir(),
        ] {
            fs::create_dir_all(&dir).map_err(|e| StorageError::CreateDir(dir.clone(), e))?;
        }
        Ok(())
    }

    fn delete_file(&self, path: &Path) -> Result<(), StorageError> {
        if path.exists() {
            fs::remove_file(path).map_err(|e| StorageError::WriteFile(path.to_path_buf(), e))?;
        }
        Ok(())
    }

    fn layouts_dir(&self) -> PathBuf {
        self.root.join("layouts")
    }

    fn hardware_dir(&self) -> PathBuf {
        self.root.join("hardware")
    }

    fn keymaps_dir(&self) -> PathBuf {
        self.root.join("keymaps")
    }

    fn runtime_config_path(&self) -> PathBuf {
        self.root.join("runtime.json")
    }
}

/// In-memory representation of all persisted resources.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredResources {
    pub layouts: HashMap<VirtualLayoutId, VirtualLayout>,
    pub hardware_profiles: HashMap<HardwareProfileId, HardwareProfile>,
    pub keymaps: HashMap<KeymapId, Keymap>,
    pub runtime: RuntimeConfig,
}

/// Errors produced while reading/writing configuration resources.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to create directory {0}: {1}")]
    CreateDir(PathBuf, #[source] std::io::Error),
    #[error("failed to read directory {0}: {1}")]
    ReadDir(PathBuf, #[source] std::io::Error),
    #[error("failed to read file {0}: {1}")]
    ReadFile(PathBuf, #[source] std::io::Error),
    #[error("failed to write file {0}: {1}")]
    WriteFile(PathBuf, #[source] std::io::Error),
    #[error("failed to parse JSON {0}: {1}")]
    Parse(PathBuf, #[source] serde_json::Error),
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, StorageError> {
    let content =
        fs::read_to_string(path).map_err(|e| StorageError::ReadFile(path.to_path_buf(), e))?;
    serde_json::from_str(&content).map_err(|e| StorageError::Parse(path.to_path_buf(), e))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), StorageError> {
    let content = serde_json::to_string_pretty(value)
        .map_err(|e| StorageError::Parse(path.to_path_buf(), e))?;
    fs::write(path, content).map_err(|e| StorageError::WriteFile(path.to_path_buf(), e))
}

fn json_name(id: &str) -> String {
    format!("{id}.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn sample_layout() -> VirtualLayout {
        VirtualLayout {
            id: "layout-1".into(),
            name: "Grid 2x2".into(),
            layout_type: super::super::models::LayoutType::Matrix,
            keys: vec![],
        }
    }

    fn sample_hardware() -> HardwareProfile {
        HardwareProfile {
            id: "hw-1".into(),
            vendor_id: 0x1,
            product_id: 0x2,
            name: Some("Test Hardware".into()),
            virtual_layout_id: "layout-1".into(),
            wiring: HashMap::from([(4, "VK_A".into())]),
        }
    }

    fn sample_keymap() -> Keymap {
        Keymap {
            id: "km-1".into(),
            name: "Base".into(),
            virtual_layout_id: "layout-1".into(),
            layers: vec![super::super::models::KeymapLayer {
                name: "default".into(),
                bindings: HashMap::new(),
            }],
            combos: vec![],
        }
    }

    #[test]
    fn saves_and_loads_resources() {
        let tmp = TempDir::new().expect("tempdir");
        let manager = ConfigManager::new(tmp.path());

        let layout = sample_layout();
        let hardware = sample_hardware();
        let keymap = sample_keymap();
        let runtime = RuntimeConfig {
            devices: vec![super::super::models::DeviceSlots {
                device: super::super::models::DeviceInstanceId {
                    vendor_id: 0x1,
                    product_id: 0x2,
                    serial: Some("abc".into()),
                },
                slots: vec![super::super::models::ProfileSlot {
                    id: "slot-1".into(),
                    hardware_profile_id: hardware.id.clone(),
                    keymap_id: keymap.id.clone(),
                    active: true,
                    priority: 1,
                }],
            }],
        };

        manager.save_virtual_layout(&layout).expect("save layout");
        manager
            .save_hardware_profile(&hardware)
            .expect("save hardware");
        manager.save_keymap(&keymap).expect("save keymap");
        manager.save_runtime_config(&runtime).expect("save runtime");

        let resources = manager.load_all().expect("load all");

        assert_eq!(resources.layouts.get("layout-1"), Some(&layout));
        assert_eq!(resources.hardware_profiles.get("hw-1"), Some(&hardware));
        assert_eq!(resources.keymaps.get("km-1"), Some(&keymap));
        assert_eq!(resources.runtime, runtime);
    }

    #[test]
    fn returns_empty_maps_when_directories_missing() {
        let tmp = TempDir::new().expect("tempdir");
        let manager = ConfigManager::new(tmp.path());

        let resources = manager.load_all().expect("load all");

        assert!(resources.layouts.is_empty());
        assert!(resources.hardware_profiles.is_empty());
        assert!(resources.keymaps.is_empty());
        assert!(resources.runtime.devices.is_empty());
    }
}
