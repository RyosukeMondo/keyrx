//! Device definition library for loading and managing device definitions.
//!
//! This module provides the `DeviceDefinitionLibrary` for loading device
//! definitions from TOML files, indexing them by VID:PID, and providing
//! O(1) lookup operations.

use crate::definitions::types::{DeviceDefinition, DeviceDefinitionError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Error type for library operations
#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Failed to read directory {path}: {source}")]
    DirectoryRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to parse TOML from {path}: {source}")]
    TomlParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("Validation failed for {path}: {source}")]
    ValidationFailed {
        path: PathBuf,
        source: DeviceDefinitionError,
    },

    #[error("Duplicate device definition for {vid:04x}:{pid:04x} found in {path1} and {path2}")]
    DuplicateDefinition {
        vid: u16,
        pid: u16,
        path1: PathBuf,
        path2: PathBuf,
    },
}

/// Device definition library for managing device definitions
///
/// Loads device definitions from TOML files in a directory tree,
/// validates them, and provides O(1) lookup by VID:PID.
pub struct DeviceDefinitionLibrary {
    /// Definitions indexed by (vendor_id, product_id)
    definitions: HashMap<(u16, u16), DeviceDefinition>,

    /// Map VID:PID to the source file path for debugging
    source_paths: HashMap<(u16, u16), PathBuf>,
}

impl DeviceDefinitionLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            source_paths: HashMap::new(),
        }
    }

    /// Load all device definitions from a directory
    ///
    /// Recursively walks the directory tree, loads all .toml files,
    /// validates them, and indexes by VID:PID. Invalid files are
    /// skipped with warnings.
    ///
    /// # Arguments
    /// * `path` - Root directory to search for .toml files
    ///
    /// # Returns
    /// Number of successfully loaded definitions
    pub fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, LibraryError> {
        let path = path.as_ref();
        info!("Loading device definitions from: {}", path.display());

        let mut loaded_count = 0;
        let mut skipped_count = 0;

        // Walk directory tree
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            // Only process .toml files
            if !entry_path.is_file() || entry_path.extension() != Some(std::ffi::OsStr::new("toml"))
            {
                continue;
            }

            // Try to load the definition
            match self.load_definition(entry_path) {
                Ok(()) => {
                    loaded_count += 1;
                }
                Err(e) => {
                    warn!("Skipping invalid definition file: {}", e);
                    skipped_count += 1;
                }
            }
        }

        info!(
            "Loaded {} device definitions ({} skipped)",
            loaded_count, skipped_count
        );

        Ok(loaded_count)
    }

    /// Load a single device definition from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to .toml file
    fn load_definition<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LibraryError> {
        let path = path.as_ref();
        debug!("Loading definition from: {}", path.display());

        // Read file
        let contents = std::fs::read_to_string(path).map_err(|source| LibraryError::FileRead {
            path: path.to_path_buf(),
            source,
        })?;

        // Parse TOML
        let definition: DeviceDefinition =
            toml::from_str(&contents).map_err(|source| LibraryError::TomlParse {
                path: path.to_path_buf(),
                source,
            })?;

        // Validate
        self.validate_definition(&definition, path)?;

        // Check for duplicates
        let key = (definition.vendor_id, definition.product_id);
        if let Some(existing_path) = self.source_paths.get(&key) {
            return Err(LibraryError::DuplicateDefinition {
                vid: definition.vendor_id,
                pid: definition.product_id,
                path1: existing_path.clone(),
                path2: path.to_path_buf(),
            });
        }

        // Store definition
        debug!(
            "Loaded: {} ({:04x}:{:04x})",
            definition.name, definition.vendor_id, definition.product_id
        );
        self.source_paths.insert(key, path.to_path_buf());
        self.definitions.insert(key, definition);

        Ok(())
    }

    /// Validate a device definition
    ///
    /// # Arguments
    /// * `definition` - The definition to validate
    /// * `path` - Source file path (for error reporting)
    fn validate_definition<P: AsRef<Path>>(
        &self,
        definition: &DeviceDefinition,
        path: P,
    ) -> Result<(), LibraryError> {
        definition
            .validate()
            .map_err(|source| LibraryError::ValidationFailed {
                path: path.as_ref().to_path_buf(),
                source,
            })
    }

    /// Find a device definition by VID:PID
    ///
    /// # Arguments
    /// * `vendor_id` - USB Vendor ID
    /// * `product_id` - USB Product ID
    ///
    /// # Returns
    /// Reference to the definition, or None if not found
    pub fn find_definition(&self, vendor_id: u16, product_id: u16) -> Option<&DeviceDefinition> {
        self.definitions.get(&(vendor_id, product_id))
    }

    /// List all loaded device definitions
    ///
    /// # Returns
    /// Iterator over all definitions
    pub fn list_definitions(&self) -> impl Iterator<Item = &DeviceDefinition> {
        self.definitions.values()
    }

    /// Get the number of loaded definitions
    pub fn count(&self) -> usize {
        self.definitions.len()
    }

    /// Check if the library is empty
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Get the source file path for a definition
    ///
    /// # Arguments
    /// * `vendor_id` - USB Vendor ID
    /// * `product_id` - USB Product ID
    ///
    /// # Returns
    /// Path to the source .toml file, or None if not found
    pub fn get_source_path(&self, vendor_id: u16, product_id: u16) -> Option<&Path> {
        self.source_paths
            .get(&(vendor_id, product_id))
            .map(|p| p.as_path())
    }

    /// Clear all loaded definitions
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.source_paths.clear();
    }
}

impl Default for DeviceDefinitionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_definition_toml() -> String {
        r#"
name = "Test Device"
vendor_id = 0x1234
product_id = 0x5678
manufacturer = "Test Manufacturer"

[layout]
layout_type = "matrix"
rows = 2
cols = 3

[matrix_map]
"1" = { row = 0, col = 0 }
"2" = { row = 0, col = 1 }
"3" = { row = 0, col = 2 }
"4" = { row = 1, col = 0 }
"5" = { row = 1, col = 1 }
"6" = { row = 1, col = 2 }

[visual]
key_width = 80
key_height = 80
key_spacing = 4
"#
        .to_string()
    }

    fn create_invalid_toml() -> String {
        r#"
name = "Invalid Device"
vendor_id = 0
product_id = 0x5678
"#
        .to_string()
    }

    #[test]
    fn test_new_library_is_empty() {
        let library = DeviceDefinitionLibrary::new();
        assert!(library.is_empty());
        assert_eq!(library.count(), 0);
    }

    #[test]
    fn test_load_single_definition() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        let count = library.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 1);
        assert_eq!(library.count(), 1);
        assert!(!library.is_empty());
    }

    #[test]
    fn test_find_definition() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        let def = library.find_definition(0x1234, 0x5678);
        assert!(def.is_some());
        assert_eq!(def.unwrap().name, "Test Device");

        let not_found = library.find_definition(0x9999, 0x8888);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_list_definitions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        let definitions: Vec<_> = library.list_definitions().collect();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "Test Device");
    }

    #[test]
    fn test_skip_invalid_definitions() {
        let temp_dir = TempDir::new().unwrap();

        // Create one valid and one invalid file
        let valid_path = temp_dir.path().join("valid.toml");
        std::fs::write(&valid_path, create_test_definition_toml()).unwrap();

        let invalid_path = temp_dir.path().join("invalid.toml");
        std::fs::write(&invalid_path, create_invalid_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        let count = library.load_from_directory(temp_dir.path()).unwrap();

        // Should load only the valid one
        assert_eq!(count, 1);
        assert_eq!(library.count(), 1);
    }

    #[test]
    fn test_recursive_directory_loading() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // Create files in root and subdirectory
        let file1 = temp_dir.path().join("device1.toml");
        std::fs::write(&file1, create_test_definition_toml()).unwrap();

        let file2 = subdir.join("device2.toml");
        let toml2 = create_test_definition_toml().replace("0x5678", "0x9999");
        std::fs::write(&file2, toml2).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        let count = library.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 2);
        assert_eq!(library.count(), 2);
        assert!(library.find_definition(0x1234, 0x5678).is_some());
        assert!(library.find_definition(0x1234, 0x9999).is_some());
    }

    #[test]
    fn test_duplicate_definition_error() {
        let temp_dir = TempDir::new().unwrap();

        // Create two files with same VID:PID
        let file1 = temp_dir.path().join("device1.toml");
        std::fs::write(&file1, create_test_definition_toml()).unwrap();

        let file2 = temp_dir.path().join("device2.toml");
        std::fs::write(&file2, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();

        // Load first file
        library.load_definition(&file1).unwrap();

        // Second file should fail with duplicate error
        let result = library.load_definition(&file2);
        assert!(matches!(
            result,
            Err(LibraryError::DuplicateDefinition { .. })
        ));
    }

    #[test]
    fn test_get_source_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        let source_path = library.get_source_path(0x1234, 0x5678);
        assert!(source_path.is_some());
        assert_eq!(source_path.unwrap(), file_path.as_path());
    }

    #[test]
    fn test_clear() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(library.count(), 1);

        library.clear();

        assert!(library.is_empty());
        assert_eq!(library.count(), 0);
    }

    #[test]
    fn test_ignore_non_toml_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create .toml file
        let toml_file = temp_dir.path().join("device.toml");
        std::fs::write(&toml_file, create_test_definition_toml()).unwrap();

        // Create non-.toml files
        let txt_file = temp_dir.path().join("readme.txt");
        std::fs::write(&txt_file, "This is not a TOML file").unwrap();

        let json_file = temp_dir.path().join("config.json");
        std::fs::write(&json_file, "{}").unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        let count = library.load_from_directory(temp_dir.path()).unwrap();

        // Should only load the .toml file
        assert_eq!(count, 1);
        assert_eq!(library.count(), 1);
    }

    #[test]
    fn test_validation_catches_errors() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.toml");

        // Create TOML with position out of bounds
        let invalid_toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"1" = { row = 5, col = 5 }
"#;
        std::fs::write(&file_path, invalid_toml).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        let count = library.load_from_directory(temp_dir.path()).unwrap();

        // Should skip the invalid file
        assert_eq!(count, 0);
        assert!(library.is_empty());
    }

    #[test]
    fn test_stream_deck_definitions() {
        use std::path::PathBuf;
        let device_defs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("device_definitions");

        let mut library = DeviceDefinitionLibrary::new();
        let result = library.load_from_directory(&device_defs_path);

        assert!(
            result.is_ok(),
            "Failed to load definitions: {:?}",
            result.err()
        );
        let count = result.unwrap();
        assert!(
            count >= 5,
            "Expected at least 5 definitions (ANSI, ISO, 3x Stream Deck), got {}",
            count
        );

        // Verify Stream Deck MK.2
        let mk2 = library.find_definition(0x0fd9, 0x0080);
        assert!(mk2.is_some(), "Stream Deck MK.2 not found");
        assert_eq!(mk2.unwrap().name, "Elgato Stream Deck MK.2");

        // Verify Stream Deck XL
        let xl = library.find_definition(0x0fd9, 0x006c);
        assert!(xl.is_some(), "Stream Deck XL not found");
        assert_eq!(xl.unwrap().name, "Elgato Stream Deck XL");

        // Verify Stream Deck Mini
        let mini = library.find_definition(0x0fd9, 0x0063);
        assert!(mini.is_some(), "Stream Deck Mini not found");
        assert_eq!(mini.unwrap().name, "Elgato Stream Deck Mini");
    }
}
