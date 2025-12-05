//! Device definition data structures for revolutionary mapping.
//!
//! This module defines the data structures for device layout definitions,
//! which are loaded from TOML files and used to translate scancodes to
//! physical positions (row, col) for layout-aware remapping.

use crate::registry::PhysicalPosition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error type for device definition operations
#[derive(Debug, Error)]
pub enum DeviceDefinitionError {
    #[error("Invalid vendor_id: {0} (must be non-zero)")]
    InvalidVendorId(u16),

    #[error("Invalid product_id: {0} (must be non-zero)")]
    InvalidProductId(u16),

    #[error("Invalid layout dimensions: rows={rows}, cols={cols:?} (must be non-zero)")]
    InvalidDimensions { rows: u8, cols: Option<u8> },

    #[error("Invalid layout type: {0} (must be 'matrix', 'standard', or 'split')")]
    InvalidLayoutType(String),

    #[error("Matrix map contains invalid position: scancode={scancode}, position=({row}, {col})")]
    InvalidMatrixPosition {
        scancode: u16,
        row: u8,
        col: u8,
    },

    #[error("Matrix map is empty (at least one scancode mapping required)")]
    EmptyMatrixMap,

    #[error("Scancode {0} appears multiple times in matrix_map")]
    DuplicateScancode(u16),

    #[error("Position ({row}, {col}) appears multiple times in matrix_map")]
    DuplicatePosition { row: u8, col: u8 },
}

/// Complete device definition loaded from TOML
///
/// Contains all information needed to understand a device's physical layout,
/// translate scancodes to positions, and render the device in the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceDefinition {
    /// Human-readable device name (e.g., "Elgato Stream Deck MK.2")
    pub name: String,

    /// USB vendor ID (must be non-zero)
    pub vendor_id: u16,

    /// USB product ID (must be non-zero)
    pub product_id: u16,

    /// Optional manufacturer name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,

    /// Layout definition describing device structure
    pub layout: LayoutDefinition,

    /// Scancode to physical position mapping
    /// Key: scancode/HID usage ID, Value: (row, col) position
    pub matrix_map: HashMap<u16, PhysicalPosition>,

    /// Optional visual metadata for UI rendering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual: Option<VisualMetadata>,
}

impl DeviceDefinition {
    /// Validate this device definition
    ///
    /// Checks:
    /// - vendor_id and product_id are non-zero
    /// - layout dimensions are valid
    /// - matrix_map is non-empty
    /// - all matrix positions are within layout bounds
    /// - no duplicate scancodes or positions
    pub fn validate(&self) -> Result<(), DeviceDefinitionError> {
        // Validate vendor_id
        if self.vendor_id == 0 {
            return Err(DeviceDefinitionError::InvalidVendorId(self.vendor_id));
        }

        // Validate product_id
        if self.product_id == 0 {
            return Err(DeviceDefinitionError::InvalidProductId(self.product_id));
        }

        // Validate layout
        self.layout.validate()?;

        // Validate matrix_map is non-empty
        if self.matrix_map.is_empty() {
            return Err(DeviceDefinitionError::EmptyMatrixMap);
        }

        // Check for duplicate scancodes (shouldn't happen with HashMap, but be explicit)
        let mut seen_positions: HashMap<PhysicalPosition, u16> = HashMap::new();

        for (&scancode, &position) in &self.matrix_map {
            // Validate position is within bounds
            if !self.layout.contains_position(position) {
                return Err(DeviceDefinitionError::InvalidMatrixPosition {
                    scancode,
                    row: position.row,
                    col: position.col,
                });
            }

            // Check for duplicate positions
            if seen_positions.contains_key(&position) {
                return Err(DeviceDefinitionError::DuplicatePosition {
                    row: position.row,
                    col: position.col,
                });
            }
            seen_positions.insert(position, scancode);
        }

        Ok(())
    }

    /// Get the layout type as a string
    pub fn layout_type_str(&self) -> &str {
        &self.layout.layout_type
    }

    /// Translate a scancode to a physical position
    pub fn scancode_to_position(&self, scancode: u16) -> Option<PhysicalPosition> {
        self.matrix_map.get(&scancode).copied()
    }

    /// Get a unique key for this device (VID:PID)
    pub fn device_key(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor_id, self.product_id)
    }
}

/// Layout definition describing device structure
///
/// Defines the physical organization of keys/buttons on the device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutDefinition {
    /// Layout type: "matrix", "standard", or "split"
    pub layout_type: String,

    /// Number of rows
    pub rows: u8,

    /// Number of columns (for matrix layouts)
    /// None for irregular layouts that use cols_per_row
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<u8>,

    /// Columns per row (for irregular layouts)
    /// Used when different rows have different column counts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols_per_row: Option<Vec<u8>>,
}

impl LayoutDefinition {
    /// Validate this layout definition
    pub fn validate(&self) -> Result<(), DeviceDefinitionError> {
        // Validate layout_type
        match self.layout_type.as_str() {
            "matrix" | "standard" | "split" => {}
            _ => {
                return Err(DeviceDefinitionError::InvalidLayoutType(
                    self.layout_type.clone(),
                ))
            }
        }

        // Validate rows
        if self.rows == 0 {
            return Err(DeviceDefinitionError::InvalidDimensions {
                rows: self.rows,
                cols: self.cols,
            });
        }

        // Validate cols (if present)
        if let Some(cols) = self.cols {
            if cols == 0 {
                return Err(DeviceDefinitionError::InvalidDimensions {
                    rows: self.rows,
                    cols: self.cols,
                });
            }
        }

        // Validate cols_per_row (if present)
        if let Some(ref cols_per_row) = self.cols_per_row {
            if cols_per_row.len() != self.rows as usize {
                return Err(DeviceDefinitionError::InvalidDimensions {
                    rows: self.rows,
                    cols: self.cols,
                });
            }
            for &cols in cols_per_row {
                if cols == 0 {
                    return Err(DeviceDefinitionError::InvalidDimensions {
                        rows: self.rows,
                        cols: Some(cols),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check if a position is within the bounds of this layout
    pub fn contains_position(&self, pos: PhysicalPosition) -> bool {
        // Check row
        if pos.row >= self.rows {
            return false;
        }

        // Check column
        if let Some(cols) = self.cols {
            // Uniform column count
            pos.col < cols
        } else if let Some(ref cols_per_row) = self.cols_per_row {
            // Irregular layout
            if let Some(&max_cols) = cols_per_row.get(pos.row as usize) {
                pos.col < max_cols
            } else {
                false
            }
        } else {
            // No column constraint specified - accept any column
            true
        }
    }

    /// Get the maximum number of columns in this layout
    pub fn max_cols(&self) -> Option<u8> {
        self.cols.or_else(|| {
            self.cols_per_row
                .as_ref()
                .and_then(|cpr| cpr.iter().max().copied())
        })
    }
}

/// Visual metadata for UI rendering
///
/// Provides hints for how to render the device in the visual editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualMetadata {
    /// Key width in pixels
    pub key_width: u16,

    /// Key height in pixels
    pub key_height: u16,

    /// Spacing between keys in pixels
    pub key_spacing: u8,
}

impl VisualMetadata {
    /// Create new visual metadata with default spacing
    pub fn new(key_width: u16, key_height: u16) -> Self {
        Self {
            key_width,
            key_height,
            key_spacing: 4, // Default 4px spacing
        }
    }

    /// Create new visual metadata with custom spacing
    pub fn with_spacing(key_width: u16, key_height: u16, key_spacing: u8) -> Self {
        Self {
            key_width,
            key_height,
            key_spacing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_definition() -> DeviceDefinition {
        let mut matrix_map = HashMap::new();
        matrix_map.insert(1, PhysicalPosition::new(0, 0));
        matrix_map.insert(2, PhysicalPosition::new(0, 1));
        matrix_map.insert(3, PhysicalPosition::new(1, 0));

        DeviceDefinition {
            name: "Test Device".to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
            manufacturer: Some("Test Manufacturer".to_string()),
            layout: LayoutDefinition {
                layout_type: "matrix".to_string(),
                rows: 2,
                cols: Some(2),
                cols_per_row: None,
            },
            matrix_map,
            visual: Some(VisualMetadata::new(80, 80)),
        }
    }

    #[test]
    fn test_valid_definition() {
        let def = create_valid_definition();
        assert!(def.validate().is_ok());
    }

    #[test]
    fn test_invalid_vendor_id() {
        let mut def = create_valid_definition();
        def.vendor_id = 0;
        assert!(matches!(
            def.validate(),
            Err(DeviceDefinitionError::InvalidVendorId(0))
        ));
    }

    #[test]
    fn test_invalid_product_id() {
        let mut def = create_valid_definition();
        def.product_id = 0;
        assert!(matches!(
            def.validate(),
            Err(DeviceDefinitionError::InvalidProductId(0))
        ));
    }

    #[test]
    fn test_empty_matrix_map() {
        let mut def = create_valid_definition();
        def.matrix_map.clear();
        assert!(matches!(
            def.validate(),
            Err(DeviceDefinitionError::EmptyMatrixMap)
        ));
    }

    #[test]
    fn test_position_out_of_bounds() {
        let mut def = create_valid_definition();
        def.matrix_map.insert(10, PhysicalPosition::new(5, 5)); // Beyond 2x2 layout
        assert!(matches!(
            def.validate(),
            Err(DeviceDefinitionError::InvalidMatrixPosition { .. })
        ));
    }

    #[test]
    fn test_invalid_layout_type() {
        let mut def = create_valid_definition();
        def.layout.layout_type = "invalid".to_string();
        assert!(matches!(
            def.validate(),
            Err(DeviceDefinitionError::InvalidLayoutType(_))
        ));
    }

    #[test]
    fn test_scancode_to_position() {
        let def = create_valid_definition();
        assert_eq!(
            def.scancode_to_position(1),
            Some(PhysicalPosition::new(0, 0))
        );
        assert_eq!(
            def.scancode_to_position(2),
            Some(PhysicalPosition::new(0, 1))
        );
        assert_eq!(def.scancode_to_position(999), None);
    }

    #[test]
    fn test_device_key() {
        let def = create_valid_definition();
        assert_eq!(def.device_key(), "1234:5678");
    }

    #[test]
    fn test_layout_contains_position() {
        let layout = LayoutDefinition {
            layout_type: "matrix".to_string(),
            rows: 3,
            cols: Some(5),
            cols_per_row: None,
        };

        assert!(layout.contains_position(PhysicalPosition::new(0, 0)));
        assert!(layout.contains_position(PhysicalPosition::new(2, 4)));
        assert!(!layout.contains_position(PhysicalPosition::new(3, 0))); // row out of bounds
        assert!(!layout.contains_position(PhysicalPosition::new(0, 5))); // col out of bounds
    }

    #[test]
    fn test_layout_irregular_cols() {
        let layout = LayoutDefinition {
            layout_type: "standard".to_string(),
            rows: 3,
            cols: None,
            cols_per_row: Some(vec![10, 12, 8]), // Different cols per row
        };

        assert!(layout.validate().is_ok());
        assert!(layout.contains_position(PhysicalPosition::new(0, 9))); // Row 0 has 10 cols
        assert!(!layout.contains_position(PhysicalPosition::new(0, 10)));
        assert!(layout.contains_position(PhysicalPosition::new(1, 11))); // Row 1 has 12 cols
        assert!(!layout.contains_position(PhysicalPosition::new(2, 8))); // Row 2 has 8 cols
    }

    #[test]
    fn test_layout_max_cols() {
        let layout1 = LayoutDefinition {
            layout_type: "matrix".to_string(),
            rows: 3,
            cols: Some(5),
            cols_per_row: None,
        };
        assert_eq!(layout1.max_cols(), Some(5));

        let layout2 = LayoutDefinition {
            layout_type: "standard".to_string(),
            rows: 3,
            cols: None,
            cols_per_row: Some(vec![10, 12, 8]),
        };
        assert_eq!(layout2.max_cols(), Some(12));
    }

    #[test]
    fn test_visual_metadata() {
        let visual = VisualMetadata::new(80, 80);
        assert_eq!(visual.key_width, 80);
        assert_eq!(visual.key_height, 80);
        assert_eq!(visual.key_spacing, 4);

        let visual2 = VisualMetadata::with_spacing(100, 50, 8);
        assert_eq!(visual2.key_width, 100);
        assert_eq!(visual2.key_height, 50);
        assert_eq!(visual2.key_spacing, 8);
    }
}
