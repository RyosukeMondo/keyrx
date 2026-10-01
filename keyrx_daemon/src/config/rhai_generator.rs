//! RhaiGenerator: programmatic, minimal-diff edits of a Rhai configuration.
//!
//! The source text is the truth. An edit touches only the lines it must: a
//! changed mapping replaces its own line in place (keeping its indentation), a
//! new mapping is inserted next to its siblings, and every other line - blank
//! lines, comments, spacing, line endings, the trailing newline - is written
//! back byte for byte, so a profile kept in git shows a one-line diff for a
//! one-key change.
//!
//! Key names are compared by the key they denote, not by spelling: `CapsLock`
//! and `VK_CapsLock` are the same input key, so setting one replaces the other
//! instead of appending a duplicate that would not compile. Input keys are
//! bare names (a `VK_` prefix is tolerated); outputs are written `VK_...`.

use rhai::Engine;
use std::fmt;
use std::path::Path;
use thiserror::Error;

mod edit;
mod keys;
mod scan;

use scan::Layout;

pub use keys::{canonical_input, canonical_output, same_key};

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
    /// Simple key remap: map("A", "VK_B")
    SimpleRemap { output: String },

    /// Tap-hold: tap_hold("Space", "VK_Space", "MD_00", 200)
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

/// A Rhai configuration held as its original lines, edited in place.
#[derive(Debug)]
pub struct RhaiGenerator {
    lines: Vec<String>,
    /// Line ending of the original text (`"\n"` or `"\r\n"`).
    eol: &'static str,
    /// Whether the original text ended with a line ending.
    trailing_eol: bool,
}

impl RhaiGenerator {
    /// Load and parse a Rhai file
    pub fn load(path: &Path) -> Result<Self, GeneratorError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse Rhai source, checking its device/layer structure.
    pub fn parse(source: &str) -> Result<Self, GeneratorError> {
        let generator = Self {
            lines: source.lines().map(str::to_string).collect(),
            eol: if source.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            },
            trailing_eol: source.ends_with('\n'),
        };
        generator.layout()?;
        Ok(generator)
    }

    fn layout(&self) -> Result<Layout, GeneratorError> {
        Layout::scan(&self.lines)
    }

    /// Set a key mapping in a layer (`base` or empty for the device's base
    /// mappings). An existing mapping of the same key, however it was
    /// spelled, is replaced where it stands; otherwise the line is added
    /// after the last mapping of that layer.
    pub fn set_key_mapping(
        &mut self,
        layer: &str,
        key: &str,
        action: KeyAction,
    ) -> Result<(), GeneratorError> {
        let key = canonical_input(key)?;
        let body = edit::mapping_body(&key, &action)?;
        let layout = self.layout()?;
        let scope = layout.scope(layer)?;
        let existing = edit::matching_lines(&self.lines, &scope, &layout, &key);
        match existing.split_first() {
            Some((&first, rest)) => {
                let indent = edit::indent_of(&self.lines[first]).to_string();
                self.lines[first] = format!("{indent}{body}");
                edit::remove_lines(&mut self.lines, rest);
            }
            None => {
                let at = edit::insertion_point(&self.lines, &scope);
                let indent = edit::scope_indent(&self.lines, &scope);
                self.lines.insert(at, format!("{indent}{body}"));
            }
        }
        Ok(())
    }

    /// Delete a key mapping from a layer
    pub fn delete_key_mapping(&mut self, layer: &str, key: &str) -> Result<(), GeneratorError> {
        let key = canonical_input(key)?;
        let layout = self.layout()?;
        let scope = layout.scope(layer)?;
        let existing = edit::matching_lines(&self.lines, &scope, &layout, &key);
        edit::remove_lines(&mut self.lines, &existing);
        Ok(())
    }

    /// The mapping line (trimmed) for `key` in `layer`, if any.
    pub fn find_mapping(&self, layer: &str, key: &str) -> Result<Option<String>, GeneratorError> {
        let key = canonical_input(key)?;
        let layout = self.layout()?;
        let scope = layout.scope(layer)?;
        Ok(edit::matching_lines(&self.lines, &scope, &layout, &key)
            .first()
            .map(|&i| self.lines[i].trim().to_string()))
    }

    /// Add a new layer
    pub fn add_layer(
        &mut self,
        layer_id: &str,
        _name: &str,
        _mode: LayerMode,
    ) -> Result<(), GeneratorError> {
        validate_layer_id(layer_id)?;
        let layout = self.layout()?;
        if layout.layer(layer_id).is_some() {
            return Err(GeneratorError::LayerExists(layer_id.to_string()));
        }
        let at = layout.device_end;
        let mut block = Vec::new();
        if at > 0 && !self.lines[at - 1].trim().is_empty() {
            block.push(String::new());
        }
        block.push(format!("when_start(\"{layer_id}\");"));
        block.push("when_end();".to_string());
        block.push(String::new());
        self.lines.splice(at..at, block);
        Ok(())
    }

    /// Rename a layer
    pub fn rename_layer(&mut self, layer_id: &str, new_id: &str) -> Result<(), GeneratorError> {
        validate_layer_id(new_id)?;
        let layout = self.layout()?;
        let span = layout
            .layer(layer_id)
            .ok_or_else(|| GeneratorError::LayerNotFound(layer_id.to_string()))?;
        if layout.layer(new_id).is_some() {
            return Err(GeneratorError::LayerExists(new_id.to_string()));
        }
        let line = &self.lines[span.start];
        self.lines[span.start] =
            line.replacen(&format!("\"{layer_id}\""), &format!("\"{new_id}\""), 1);
        Ok(())
    }

    /// Delete a layer (its block and one blank line around it)
    pub fn delete_layer(&mut self, layer_id: &str) -> Result<(), GeneratorError> {
        let layout = self.layout()?;
        let span = layout
            .layer(layer_id)
            .ok_or_else(|| GeneratorError::LayerNotFound(layer_id.to_string()))?;
        let mut end = span.end + 1;
        if self.lines.get(end).is_some_and(|l| l.trim().is_empty()) {
            end += 1;
        }
        self.lines.drain(span.start..end);
        Ok(())
    }

    /// List all layers with their mapping counts
    pub fn list_layers(&self) -> Vec<(String, usize)> {
        let Ok(layout) = self.layout() else {
            return Vec::new();
        };
        layout
            .layers
            .iter()
            .map(|span| {
                let body = &self.lines[span.start + 1..span.end];
                (span.id.clone(), edit::count_mappings(body))
            })
            .collect()
    }

    /// Get all mappings in a layer
    pub fn get_layer_mappings(&self, layer_id: &str) -> Result<Vec<String>, GeneratorError> {
        let layout = self.layout()?;
        let scope = layout.scope(layer_id)?;
        Ok(scope
            .lines()
            .filter(|&i| !layout.in_other_scope(&scope, i))
            .map(|i| self.lines[i].trim())
            .filter(|l| !l.is_empty() && !l.starts_with("//"))
            .map(str::to_string)
            .collect())
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), GeneratorError> {
        let content = self.to_string();
        // Validate syntax before saving
        self.validate_syntax(&content)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate syntax by parsing with Rhai engine
    fn validate_syntax(&self, source: &str) -> Result<(), GeneratorError> {
        let engine = Engine::new();
        engine
            .compile(source)
            .map_err(|e| GeneratorError::SyntaxError(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Display for RhaiGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.lines.join(self.eol))?;
        if self.trailing_eol {
            f.write_str(self.eol)?;
        }
        Ok(())
    }
}

/// Validate layer ID format
fn validate_layer_id(layer_id: &str) -> Result<(), GeneratorError> {
    if !layer_id.starts_with("MD_") {
        return Err(GeneratorError::InvalidLayerId(format!(
            "Layer ID must start with MD_: {layer_id}"
        )));
    }
    if layer_id.len() > 32 {
        return Err(GeneratorError::InvalidLayerId(
            "Layer ID too long (max 32 chars)".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
