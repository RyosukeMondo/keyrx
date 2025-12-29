//! Keyboard layout management with KLE JSON format support.
//!
//! This module manages keyboard layouts in keyboard-layout-editor.com (KLE) JSON format.
//! It provides builtin layouts embedded in the binary and supports importing custom layouts.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Maximum number of custom layouts allowed
const MAX_CUSTOM_LAYOUTS: usize = 50;

/// Maximum layout name length
const MAX_LAYOUT_NAME_LEN: usize = 32;

/// Maximum layout file size (1 MB)
const MAX_LAYOUT_FILE_SIZE: usize = 1_048_576;

/// Maximum number of keys in a layout
const MAX_KEYS_IN_LAYOUT: usize = 200;

/// Source of a keyboard layout
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutSource {
    /// Builtin layout embedded in binary
    Builtin,
    /// Custom layout imported by user
    Custom,
}

/// A keyboard layout in KLE JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    /// Layout name (max 32 chars)
    pub name: String,
    /// KLE JSON representation
    pub kle_json: JsonValue,
    /// Source of the layout
    pub source: LayoutSource,
}

/// Errors that can occur during layout operations
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("Layout not found: {0}")]
    NotFound(String),

    #[error("Layout name too long (max {MAX_LAYOUT_NAME_LEN} characters): {0}")]
    NameTooLong(String),

    #[error("Layout name contains invalid characters: {0}")]
    InvalidName(String),

    #[error("Cannot overwrite builtin layout: {0}")]
    BuiltinOverwrite(String),

    #[error("Layout file too large (max {MAX_LAYOUT_FILE_SIZE} bytes)")]
    FileTooLarge,

    #[error("Layout too complex (max {MAX_KEYS_IN_LAYOUT} keys)")]
    TooManyKeys,

    #[error("Invalid KLE format: {0}")]
    InvalidKleFormat(String),

    #[error("Maximum {MAX_CUSTOM_LAYOUTS} custom layouts allowed")]
    TooManyLayouts,

    #[error("Layout file not found: {0}")]
    FileNotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for layout operations
pub type Result<T> = std::result::Result<T, LayoutError>;

/// Manages keyboard layouts in KLE JSON format
pub struct LayoutManager {
    /// Directory where custom layouts are stored
    layouts_dir: PathBuf,
    /// Builtin layouts (embedded in binary)
    builtin_layouts: HashMap<String, KeyboardLayout>,
    /// Custom layouts (imported by user)
    custom_layouts: HashMap<String, KeyboardLayout>,
}

impl LayoutManager {
    /// Create a new LayoutManager with the specified layouts directory
    pub fn new(layouts_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            layouts_dir,
            builtin_layouts: HashMap::new(),
            custom_layouts: HashMap::new(),
        };

        manager.load_builtin_layouts()?;
        manager.scan_custom_layouts()?;

        Ok(manager)
    }

    /// Load builtin layouts embedded in the binary
    fn load_builtin_layouts(&mut self) -> Result<()> {
        let builtins = [
            ("ansi_104", include_str!("../../layouts/ansi_104.json")),
            ("iso_105", include_str!("../../layouts/iso_105.json")),
            ("jis_109", include_str!("../../layouts/jis_109.json")),
            ("hhkb", include_str!("../../layouts/hhkb.json")),
            ("numpad", include_str!("../../layouts/numpad.json")),
        ];

        for (name, json_str) in builtins {
            let kle_json: JsonValue = serde_json::from_str(json_str)?;
            Self::validate_kle(&kle_json)?;

            let layout = KeyboardLayout {
                name: name.to_string(),
                kle_json,
                source: LayoutSource::Builtin,
            };

            self.builtin_layouts.insert(name.to_string(), layout);
        }

        Ok(())
    }

    /// Scan the layouts directory for custom layouts
    fn scan_custom_layouts(&mut self) -> Result<()> {
        if !self.layouts_dir.exists() {
            std::fs::create_dir_all(&self.layouts_dir)?;
            return Ok(());
        }

        self.custom_layouts.clear();

        for entry in std::fs::read_dir(&self.layouts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_layout_file(&path, name) {
                        Ok(layout) => {
                            self.custom_layouts.insert(name.to_string(), layout);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load layout {}: {}", name, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a layout from a file
    fn load_layout_file(&self, path: &Path, name: &str) -> Result<KeyboardLayout> {
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_LAYOUT_FILE_SIZE as u64 {
            return Err(LayoutError::FileTooLarge);
        }

        let json_str = std::fs::read_to_string(path)?;
        let kle_json: JsonValue = serde_json::from_str(&json_str)?;
        Self::validate_kle(&kle_json)?;

        Ok(KeyboardLayout {
            name: name.to_string(),
            kle_json,
            source: LayoutSource::Custom,
        })
    }

    /// Validate a KLE JSON structure
    pub fn validate_kle(json: &JsonValue) -> Result<()> {
        let arr = json
            .as_array()
            .ok_or_else(|| LayoutError::InvalidKleFormat("root must be an array".to_string()))?;

        if arr.is_empty() {
            return Err(LayoutError::InvalidKleFormat(
                "array cannot be empty".to_string(),
            ));
        }

        let mut key_count = 0;

        for item in arr {
            if item.is_array() {
                // This is a row of keys
                let row = item.as_array().unwrap();
                for key in row {
                    if key.is_string() || key.is_object() {
                        key_count += 1;
                    }
                }
            }
        }

        if key_count > MAX_KEYS_IN_LAYOUT {
            return Err(LayoutError::TooManyKeys);
        }

        if key_count == 0 {
            return Err(LayoutError::InvalidKleFormat(
                "no keys found in layout".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate a layout name
    fn validate_name(name: &str) -> Result<()> {
        if name.len() > MAX_LAYOUT_NAME_LEN {
            return Err(LayoutError::NameTooLong(name.to_string()));
        }

        if name.is_empty() {
            return Err(LayoutError::InvalidName("name cannot be empty".to_string()));
        }

        // Only allow alphanumeric, dash, and underscore
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(LayoutError::InvalidName(name.to_string()));
        }

        Ok(())
    }

    /// List all available layouts (builtin and custom)
    pub fn list(&self) -> Vec<&KeyboardLayout> {
        let mut layouts: Vec<&KeyboardLayout> = self
            .builtin_layouts
            .values()
            .chain(self.custom_layouts.values())
            .collect();

        layouts.sort_by(|a, b| a.name.cmp(&b.name));
        layouts
    }

    /// Get a layout by name
    pub fn get(&self, name: &str) -> Option<&KeyboardLayout> {
        self.builtin_layouts
            .get(name)
            .or_else(|| self.custom_layouts.get(name))
    }

    /// Import a layout from a file
    pub fn import(&mut self, path: &Path, name: &str) -> Result<KeyboardLayout> {
        Self::validate_name(name)?;

        // Check if trying to overwrite a builtin layout
        if self.builtin_layouts.contains_key(name) {
            return Err(LayoutError::BuiltinOverwrite(name.to_string()));
        }

        // Check custom layout limit
        if self.custom_layouts.len() >= MAX_CUSTOM_LAYOUTS
            && !self.custom_layouts.contains_key(name)
        {
            return Err(LayoutError::TooManyLayouts);
        }

        // Check file exists
        if !path.exists() {
            return Err(LayoutError::FileNotFound(
                path.to_string_lossy().to_string(),
            ));
        }

        // Load and validate the layout
        let layout = self.load_layout_file(path, name)?;

        // Save to layouts directory
        let dest_path = self.layouts_dir.join(format!("{}.json", name));
        let json_str = serde_json::to_string_pretty(&layout.kle_json)?;
        std::fs::write(&dest_path, json_str)?;

        // Add to custom layouts
        self.custom_layouts.insert(name.to_string(), layout.clone());

        Ok(layout)
    }

    /// Delete a custom layout
    pub fn delete(&mut self, name: &str) -> Result<()> {
        // Cannot delete builtin layouts
        if self.builtin_layouts.contains_key(name) {
            return Err(LayoutError::BuiltinOverwrite(name.to_string()));
        }

        // Check if layout exists
        if !self.custom_layouts.contains_key(name) {
            return Err(LayoutError::NotFound(name.to_string()));
        }

        // Remove from memory
        self.custom_layouts.remove(name);

        // Remove file
        let path = self.layouts_dir.join(format!("{}.json", name));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        Ok(())
    }

    /// Get the number of custom layouts
    pub fn custom_count(&self) -> usize {
        self.custom_layouts.len()
    }

    /// Get the number of builtin layouts
    pub fn builtin_count(&self) -> usize {
        self.builtin_layouts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn create_test_kle() -> JsonValue {
        json!([
            [{"w": 1}, "Esc", "1", "2", "3"],
            ["Tab", "Q", "W", "E", "R"]
        ])
    }

    #[test]
    fn test_validate_kle_valid() {
        let kle = create_test_kle();
        assert!(LayoutManager::validate_kle(&kle).is_ok());
    }

    #[test]
    fn test_validate_kle_not_array() {
        let kle = json!({"foo": "bar"});
        assert!(matches!(
            LayoutManager::validate_kle(&kle),
            Err(LayoutError::InvalidKleFormat(_))
        ));
    }

    #[test]
    fn test_validate_kle_empty() {
        let kle = json!([]);
        assert!(matches!(
            LayoutManager::validate_kle(&kle),
            Err(LayoutError::InvalidKleFormat(_))
        ));
    }

    #[test]
    fn test_validate_kle_too_many_keys() {
        let mut rows = vec![];
        for _ in 0..25 {
            let mut row = vec![];
            for _ in 0..10 {
                row.push(json!("A"));
            }
            rows.push(json!(row));
        }
        let kle = json!(rows);
        assert!(matches!(
            LayoutManager::validate_kle(&kle),
            Err(LayoutError::TooManyKeys)
        ));
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(LayoutManager::validate_name("my_layout").is_ok());
        assert!(LayoutManager::validate_name("layout-1").is_ok());
        assert!(LayoutManager::validate_name("LAYOUT_123").is_ok());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(MAX_LAYOUT_NAME_LEN + 1);
        assert!(matches!(
            LayoutManager::validate_name(&long_name),
            Err(LayoutError::NameTooLong(_))
        ));
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        assert!(matches!(
            LayoutManager::validate_name("my layout"),
            Err(LayoutError::InvalidName(_))
        ));
        assert!(matches!(
            LayoutManager::validate_name("my@layout"),
            Err(LayoutError::InvalidName(_))
        ));
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(matches!(
            LayoutManager::validate_name(""),
            Err(LayoutError::InvalidName(_))
        ));
    }

    #[test]
    fn test_import_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let layouts_dir = temp_dir.path().join("layouts");
        std::fs::create_dir_all(&layouts_dir).unwrap();

        let mut manager = LayoutManager {
            layouts_dir: layouts_dir.clone(),
            builtin_layouts: HashMap::new(),
            custom_layouts: HashMap::new(),
        };

        // Create a test layout file
        let test_layout_path = temp_dir.path().join("test.json");
        let kle = create_test_kle();
        std::fs::write(&test_layout_path, serde_json::to_string(&kle).unwrap()).unwrap();

        // Import the layout
        let result = manager.import(&test_layout_path, "test_layout");
        assert!(result.is_ok());

        // Get the layout
        let layout = manager.get("test_layout");
        assert!(layout.is_some());
        assert_eq!(layout.unwrap().name, "test_layout");
        assert_eq!(layout.unwrap().source, LayoutSource::Custom);
    }

    #[test]
    fn test_import_overwrite_builtin() {
        let temp_dir = TempDir::new().unwrap();
        let layouts_dir = temp_dir.path().join("layouts");

        let mut manager = LayoutManager {
            layouts_dir,
            builtin_layouts: HashMap::new(),
            custom_layouts: HashMap::new(),
        };

        // Add a builtin layout
        let kle = create_test_kle();
        manager.builtin_layouts.insert(
            "builtin".to_string(),
            KeyboardLayout {
                name: "builtin".to_string(),
                kle_json: kle.clone(),
                source: LayoutSource::Builtin,
            },
        );

        // Try to import with same name
        let test_path = temp_dir.path().join("test.json");
        std::fs::write(&test_path, serde_json::to_string(&kle).unwrap()).unwrap();

        let result = manager.import(&test_path, "builtin");
        assert!(matches!(result, Err(LayoutError::BuiltinOverwrite(_))));
    }

    #[test]
    fn test_delete_custom() {
        let temp_dir = TempDir::new().unwrap();
        let layouts_dir = temp_dir.path().join("layouts");
        std::fs::create_dir_all(&layouts_dir).unwrap();

        let mut manager = LayoutManager {
            layouts_dir: layouts_dir.clone(),
            builtin_layouts: HashMap::new(),
            custom_layouts: HashMap::new(),
        };

        // Add a custom layout
        let kle = create_test_kle();
        let layout = KeyboardLayout {
            name: "custom".to_string(),
            kle_json: kle,
            source: LayoutSource::Custom,
        };

        let layout_file = layouts_dir.join("custom.json");
        std::fs::write(
            &layout_file,
            serde_json::to_string(&layout.kle_json).unwrap(),
        )
        .unwrap();
        manager.custom_layouts.insert("custom".to_string(), layout);

        // Delete it
        assert!(manager.delete("custom").is_ok());
        assert!(manager.get("custom").is_none());
        assert!(!layout_file.exists());
    }

    #[test]
    fn test_delete_builtin() {
        let temp_dir = TempDir::new().unwrap();
        let layouts_dir = temp_dir.path().join("layouts");

        let mut manager = LayoutManager {
            layouts_dir,
            builtin_layouts: HashMap::new(),
            custom_layouts: HashMap::new(),
        };

        // Add a builtin layout
        let kle = create_test_kle();
        manager.builtin_layouts.insert(
            "builtin".to_string(),
            KeyboardLayout {
                name: "builtin".to_string(),
                kle_json: kle,
                source: LayoutSource::Builtin,
            },
        );

        // Try to delete it
        let result = manager.delete("builtin");
        assert!(matches!(result, Err(LayoutError::BuiltinOverwrite(_))));
    }

    #[test]
    fn test_list_layouts() {
        let temp_dir = TempDir::new().unwrap();
        let layouts_dir = temp_dir.path().join("layouts");

        let mut manager = LayoutManager {
            layouts_dir,
            builtin_layouts: HashMap::new(),
            custom_layouts: HashMap::new(),
        };

        let kle = create_test_kle();

        // Add builtin layouts
        manager.builtin_layouts.insert(
            "builtin1".to_string(),
            KeyboardLayout {
                name: "builtin1".to_string(),
                kle_json: kle.clone(),
                source: LayoutSource::Builtin,
            },
        );

        // Add custom layouts
        manager.custom_layouts.insert(
            "custom1".to_string(),
            KeyboardLayout {
                name: "custom1".to_string(),
                kle_json: kle,
                source: LayoutSource::Custom,
            },
        );

        let layouts = manager.list();
        assert_eq!(layouts.len(), 2);
    }

    #[test]
    fn test_too_many_layouts() {
        let temp_dir = TempDir::new().unwrap();
        let layouts_dir = temp_dir.path().join("layouts");
        std::fs::create_dir_all(&layouts_dir).unwrap();

        let mut manager = LayoutManager {
            layouts_dir: layouts_dir.clone(),
            builtin_layouts: HashMap::new(),
            custom_layouts: HashMap::new(),
        };

        let kle = create_test_kle();

        // Add MAX_CUSTOM_LAYOUTS layouts
        for i in 0..MAX_CUSTOM_LAYOUTS {
            let name = format!("layout{}", i);
            manager.custom_layouts.insert(
                name.clone(),
                KeyboardLayout {
                    name,
                    kle_json: kle.clone(),
                    source: LayoutSource::Custom,
                },
            );
        }

        // Try to add one more
        let test_path = temp_dir.path().join("test.json");
        std::fs::write(&test_path, serde_json::to_string(&kle).unwrap()).unwrap();

        let result = manager.import(&test_path, "new_layout");
        assert!(matches!(result, Err(LayoutError::TooManyLayouts)));
    }
}
