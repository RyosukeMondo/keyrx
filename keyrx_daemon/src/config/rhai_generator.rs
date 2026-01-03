//! RhaiGenerator: Programmatic modification of Rhai configuration files
//!
//! This module provides structured manipulation of Rhai DSL configurations
//! while maintaining syntactic correctness. Instead of raw string concatenation,
//! it parses the structure, validates modifications, and regenerates valid code.

use rhai::Engine;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Layer already exists: {0}")]
    LayerExists(String),

    #[error("Invalid layer ID: {0}")]
    InvalidLayerId(String),

    #[error("Invalid key name: {0}")]
    InvalidKeyName(String),

    #[error("Syntax error in generated code: {0}")]
    SyntaxError(String),

    #[error("Device block not found")]
    DeviceNotFound,

    #[error("Unclosed when block for layer: {0}")]
    UnclosedWhenBlock(String),
}

/// Key action types for mapping
#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    /// Simple key remap: map("VK_A", "VK_B")
    SimpleRemap { output: String },

    /// Tap-hold: tap_hold("VK_Space", "VK_Space", "MD_00", 200)
    TapHold {
        tap: String,
        hold: String,
        threshold_ms: u16,
    },

    /// Macro sequence
    Macro { sequence: Vec<MacroStep> },

    /// Conditional mapping (when blocks handle this differently)
    Conditional {
        condition: String,
        then_action: Box<KeyAction>,
        else_action: Option<Box<KeyAction>>,
    },
}

/// Macro step (press/release/wait)
#[derive(Debug, Clone, PartialEq)]
pub enum MacroStep {
    Press(String),
    Release(String),
    Wait(u16), // milliseconds
}

/// Layer mode (for when_start blocks)
#[derive(Debug, Clone, PartialEq)]
pub enum LayerMode {
    /// Single modifier: when_start("MD_00")
    Single,
    /// Multiple modifiers: when_start(["MD_00", "MD_01"])
    Multiple,
}

/// Represents a parsed Rhai configuration file structure
#[derive(Debug)]
pub struct RhaiGenerator {
    /// Lines before device_start (comments, imports)
    header: Vec<String>,
    /// Device ID (from device_start)
    device_id: String,
    /// Mappings before any when blocks
    base_mappings: Vec<String>,
    /// Layer blocks: layer_id -> lines
    layers: HashMap<String, Vec<String>>,
    /// Lines after device_end (footer comments)
    footer: Vec<String>,
    /// Track layer order for consistent output
    layer_order: Vec<String>,
}

impl RhaiGenerator {
    /// Load and parse a Rhai file
    pub fn load(path: &Path) -> Result<Self, GeneratorError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse Rhai source into structured representation
    fn parse(source: &str) -> Result<Self, GeneratorError> {
        let mut header = Vec::new();
        let mut device_id = String::new();
        let mut base_mappings = Vec::new();
        let mut layers = HashMap::new();
        let mut footer = Vec::new();
        let mut layer_order = Vec::new();

        let mut current_section = Section::Header;
        let mut current_layer: Option<String> = None;
        let mut current_layer_lines = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            match current_section {
                Section::Header => {
                    if trimmed.starts_with("device_start(") {
                        // Extract device ID
                        if let Some(start) = trimmed.find('"') {
                            if let Some(end) = trimmed[start + 1..].find('"') {
                                device_id = trimmed[start + 1..start + 1 + end].to_string();
                            }
                        }
                        current_section = Section::DeviceBody;
                    } else {
                        header.push(line.to_string());
                    }
                }
                Section::DeviceBody => {
                    if trimmed.starts_with("when_start(") {
                        // Save previous layer if any
                        if let Some(layer_id) = current_layer.take() {
                            layers.insert(layer_id.clone(), current_layer_lines.clone());
                            current_layer_lines.clear();
                        }

                        // Extract layer ID
                        let layer_id = Self::extract_layer_id(trimmed)?;
                        current_layer = Some(layer_id.clone());
                        layer_order.push(layer_id);
                        current_section = Section::InWhenBlock;
                    } else if trimmed.starts_with("device_end()") {
                        current_section = Section::Footer;
                    } else if !trimmed.is_empty() {
                        // Include both comments and mappings
                        base_mappings.push(line.to_string());
                    }
                }
                Section::InWhenBlock => {
                    if trimmed.starts_with("when_end()") {
                        // Save current layer
                        if let Some(layer_id) = current_layer.take() {
                            layers.insert(layer_id, current_layer_lines.clone());
                            current_layer_lines.clear();
                        }
                        current_section = Section::DeviceBody;
                    } else {
                        current_layer_lines.push(line.to_string());
                    }
                }
                Section::Footer => {
                    footer.push(line.to_string());
                }
            }
        }

        if device_id.is_empty() {
            return Err(GeneratorError::DeviceNotFound);
        }

        Ok(Self {
            header,
            device_id,
            base_mappings,
            layers,
            footer,
            layer_order,
        })
    }

    /// Extract layer ID from when_start line
    fn extract_layer_id(line: &str) -> Result<String, GeneratorError> {
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                return Ok(line[start + 1..start + 1 + end].to_string());
            }
        }
        // Try array syntax: when_start(["MD_00", "MD_01"])
        if line.contains('[') {
            // For multi-layer, use the first one as the identifier
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Ok(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
        Err(GeneratorError::SyntaxError(format!(
            "Cannot extract layer ID from: {}",
            line
        )))
    }

    /// Set a key mapping in a specific layer
    pub fn set_key_mapping(
        &mut self,
        layer: &str,
        key: &str,
        action: KeyAction,
    ) -> Result<(), GeneratorError> {
        Self::validate_key_name(key)?;

        let mapping_line = Self::generate_mapping_line(key, &action)?;

        if layer == "base" || layer.is_empty() {
            // Remove existing mapping for this key
            self.base_mappings
                .retain(|line| !Self::is_mapping_for_key(line, key));
            // Add new mapping
            self.base_mappings.push(mapping_line);
        } else {
            // Layer-specific mapping
            if !self.layers.contains_key(layer) {
                return Err(GeneratorError::LayerNotFound(layer.to_string()));
            }

            // SAFETY: Layer existence verified at line 228: contains_key check guarantees get_mut succeeds
            #[allow(clippy::unwrap_used)]
            let layer_lines = self.layers.get_mut(layer).unwrap();
            layer_lines.retain(|line| !Self::is_mapping_for_key(line, key));
            layer_lines.push(mapping_line);
        }

        Ok(())
    }

    /// Delete a key mapping from a layer
    pub fn delete_key_mapping(&mut self, layer: &str, key: &str) -> Result<(), GeneratorError> {
        Self::validate_key_name(key)?;

        if layer == "base" || layer.is_empty() {
            self.base_mappings
                .retain(|line| !Self::is_mapping_for_key(line, key));
        } else {
            if !self.layers.contains_key(layer) {
                return Err(GeneratorError::LayerNotFound(layer.to_string()));
            }

            // SAFETY: Layer existence verified at line 250: contains_key check guarantees get_mut succeeds
            #[allow(clippy::unwrap_used)]
            let layer_lines = self.layers.get_mut(layer).unwrap();
            layer_lines.retain(|line| !Self::is_mapping_for_key(line, key));
        }

        Ok(())
    }

    /// Add a new layer
    pub fn add_layer(
        &mut self,
        layer_id: &str,
        _name: &str,
        _mode: LayerMode,
    ) -> Result<(), GeneratorError> {
        Self::validate_layer_id(layer_id)?;

        if self.layers.contains_key(layer_id) {
            return Err(GeneratorError::LayerExists(layer_id.to_string()));
        }

        self.layers.insert(layer_id.to_string(), Vec::new());
        self.layer_order.push(layer_id.to_string());

        Ok(())
    }

    /// Rename a layer
    pub fn rename_layer(&mut self, layer_id: &str, new_id: &str) -> Result<(), GeneratorError> {
        Self::validate_layer_id(new_id)?;

        if !self.layers.contains_key(layer_id) {
            return Err(GeneratorError::LayerNotFound(layer_id.to_string()));
        }

        if self.layers.contains_key(new_id) {
            return Err(GeneratorError::LayerExists(new_id.to_string()));
        }

        // Move the layer
        // SAFETY: Layer existence verified at line 286: contains_key check guarantees remove succeeds
        #[allow(clippy::unwrap_used)]
        let layer_lines = self.layers.remove(layer_id).unwrap();
        self.layers.insert(new_id.to_string(), layer_lines);

        // Update order
        if let Some(idx) = self.layer_order.iter().position(|id| id == layer_id) {
            self.layer_order[idx] = new_id.to_string();
        }

        Ok(())
    }

    /// Delete a layer
    pub fn delete_layer(&mut self, layer_id: &str) -> Result<(), GeneratorError> {
        if !self.layers.contains_key(layer_id) {
            return Err(GeneratorError::LayerNotFound(layer_id.to_string()));
        }

        self.layers.remove(layer_id);
        self.layer_order.retain(|id| id != layer_id);

        Ok(())
    }

    /// List all layers with their mapping counts
    pub fn list_layers(&self) -> Vec<(String, usize)> {
        self.layer_order
            .iter()
            .filter_map(|id| {
                self.layers
                    .get(id)
                    .map(|lines| (id.clone(), count_mappings(lines)))
            })
            .collect()
    }

    /// Get all mappings in a layer
    pub fn get_layer_mappings(&self, layer_id: &str) -> Result<Vec<String>, GeneratorError> {
        if layer_id == "base" || layer_id.is_empty() {
            Ok(self
                .base_mappings
                .iter()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
                .map(|line| line.trim().to_string())
                .collect())
        } else {
            self.layers
                .get(layer_id)
                .map(|lines| {
                    lines
                        .iter()
                        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
                        .map(|line| line.trim().to_string())
                        .collect()
                })
                .ok_or_else(|| GeneratorError::LayerNotFound(layer_id.to_string()))
        }
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), GeneratorError> {
        let content = self.generate_source();
        // Validate syntax before saving
        self.validate_syntax(&content)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate Rhai source string
    fn generate_source(&self) -> String {
        let mut lines = Vec::new();

        // Header
        lines.extend(self.header.iter().cloned());

        // Device start
        lines.push(format!("device_start(\"{}\");", self.device_id));
        lines.push(String::new());

        // Base mappings
        if !self.base_mappings.is_empty() {
            lines.extend(self.base_mappings.iter().cloned());
            lines.push(String::new());
        }

        // Layers in order
        for layer_id in &self.layer_order {
            if let Some(layer_lines) = self.layers.get(layer_id) {
                lines.push(format!("when_start(\"{}\");", layer_id));
                lines.extend(layer_lines.iter().cloned());
                lines.push("when_end();".to_string());
                lines.push(String::new());
            }
        }

        // Device end
        lines.push("device_end();".to_string());

        // Footer
        if !self.footer.is_empty() {
            lines.push(String::new());
            lines.extend(self.footer.iter().cloned());
        }

        lines.join("\n")
    }

    /// Validate syntax by parsing with Rhai engine
    fn validate_syntax(&self, source: &str) -> Result<(), GeneratorError> {
        let engine = Engine::new();
        engine
            .compile(source)
            .map_err(|e| GeneratorError::SyntaxError(e.to_string()))?;
        Ok(())
    }

    /// Generate a mapping line from key and action
    fn generate_mapping_line(key: &str, action: &KeyAction) -> Result<String, GeneratorError> {
        match action {
            KeyAction::SimpleRemap { output } => {
                Self::validate_key_name(output)?;
                Ok(format!("  map(\"{}\", \"{}\");", key, output))
            }
            KeyAction::TapHold {
                tap,
                hold,
                threshold_ms,
            } => {
                Self::validate_key_name(tap)?;
                // hold is a modifier, different validation
                Ok(format!(
                    "  tap_hold(\"{}\", \"{}\", \"{}\", {});",
                    key, tap, hold, threshold_ms
                ))
            }
            KeyAction::Macro { sequence } => {
                // Generate macro sequence
                let mut steps = Vec::new();
                for step in sequence {
                    match step {
                        MacroStep::Press(k) => steps.push(format!("press(\"{}\")", k)),
                        MacroStep::Release(k) => steps.push(format!("release(\"{}\")", k)),
                        MacroStep::Wait(ms) => steps.push(format!("wait({})", ms)),
                    }
                }
                Ok(format!("  macro(\"{}\", [{}]);", key, steps.join(", ")))
            }
            KeyAction::Conditional { .. } => Err(GeneratorError::SyntaxError(
                "Conditional actions should use when blocks, not direct mappings".to_string(),
            )),
        }
    }

    /// Check if a line is a mapping for the given key
    fn is_mapping_for_key(line: &str, key: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.starts_with("map(") || trimmed.starts_with("tap_hold(") {
            // Extract first argument
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let first_arg = &trimmed[start + 1..start + 1 + end];
                    return first_arg == key;
                }
            }
        }
        false
    }

    /// Validate key name format
    fn validate_key_name(key: &str) -> Result<(), GeneratorError> {
        if !key.starts_with("VK_") && !key.starts_with("MD_") && !key.starts_with("LK_") {
            return Err(GeneratorError::InvalidKeyName(format!(
                "Key must start with VK_, MD_, or LK_: {}",
                key
            )));
        }
        if key.len() > 64 {
            return Err(GeneratorError::InvalidKeyName(
                "Key name too long (max 64 chars)".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate layer ID format
    fn validate_layer_id(layer_id: &str) -> Result<(), GeneratorError> {
        if !layer_id.starts_with("MD_") {
            return Err(GeneratorError::InvalidLayerId(format!(
                "Layer ID must start with MD_: {}",
                layer_id
            )));
        }
        if layer_id.len() > 32 {
            return Err(GeneratorError::InvalidLayerId(
                "Layer ID too long (max 32 chars)".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parsing state
#[derive(Debug, Clone, Copy, PartialEq)]
enum Section {
    Header,
    DeviceBody,
    InWhenBlock,
    Footer,
}

/// Count non-comment, non-empty lines in a layer
fn count_mappings(lines: &[String]) -> usize {
    lines
        .iter()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .count()
}

impl fmt::Display for RhaiGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate_source())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let source = r#"
// Header comment
device_start("*");

map("VK_A", "VK_B");

when_start("MD_00");
  map("VK_C", "VK_D");
when_end();

device_end();
"#;

        let gen = RhaiGenerator::parse(source).unwrap();
        assert_eq!(gen.device_id, "*");
        assert_eq!(gen.layers.len(), 1);
        assert!(gen.layers.contains_key("MD_00"));
    }

    #[test]
    fn test_set_key_mapping() {
        let source = r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#;

        let mut gen = RhaiGenerator::parse(source).unwrap();
        gen.set_key_mapping(
            "base",
            "VK_C",
            KeyAction::SimpleRemap {
                output: "VK_D".to_string(),
            },
        )
        .unwrap();

        let output = gen.to_string();
        assert!(output.contains(r#"map("VK_C", "VK_D")"#));
    }

    #[test]
    fn test_add_layer() {
        let source = r#"
device_start("*");
device_end();
"#;

        let mut gen = RhaiGenerator::parse(source).unwrap();
        gen.add_layer("MD_00", "Navigation", LayerMode::Single)
            .unwrap();

        let output = gen.to_string();
        assert!(output.contains("when_start(\"MD_00\")"));
        assert!(output.contains("when_end()"));
    }

    #[test]
    fn test_delete_layer() {
        let source = r#"
device_start("*");

when_start("MD_00");
  map("VK_C", "VK_D");
when_end();

device_end();
"#;

        let mut gen = RhaiGenerator::parse(source).unwrap();
        gen.delete_layer("MD_00").unwrap();

        let output = gen.to_string();
        assert!(!output.contains("when_start(\"MD_00\")"));
    }

    #[test]
    fn test_tap_hold_mapping() {
        let source = r#"
device_start("*");
device_end();
"#;

        let mut gen = RhaiGenerator::parse(source).unwrap();
        gen.set_key_mapping(
            "base",
            "VK_Space",
            KeyAction::TapHold {
                tap: "VK_Space".to_string(),
                hold: "MD_00".to_string(),
                threshold_ms: 200,
            },
        )
        .unwrap();

        let output = gen.to_string();
        assert!(output.contains(r#"tap_hold("VK_Space", "VK_Space", "MD_00", 200)"#));
    }

    #[test]
    fn test_display_implementation() {
        let source = r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#;

        let gen = RhaiGenerator::parse(source).unwrap();
        let output = format!("{}", gen);
        assert!(output.contains("device_start"));
        assert!(output.contains("device_end"));
    }

    #[test]
    fn test_validate_key_name() {
        assert!(RhaiGenerator::validate_key_name("VK_A").is_ok());
        assert!(RhaiGenerator::validate_key_name("MD_00").is_ok());
        assert!(RhaiGenerator::validate_key_name("InvalidKey").is_err());
    }

    #[test]
    fn test_validate_layer_id() {
        assert!(RhaiGenerator::validate_layer_id("MD_00").is_ok());
        assert!(RhaiGenerator::validate_layer_id("VK_A").is_err());
        assert!(RhaiGenerator::validate_layer_id("Invalid").is_err());
    }
}
