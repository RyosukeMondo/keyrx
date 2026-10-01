//! Locating the device block and `when_start`/`when_end` layers in a Rhai
//! source, as line ranges. Nothing here modifies text.

use std::ops::Range;

use super::GeneratorError;

/// One `when_start(..)` .. `when_end()` block, as indexes of those two lines.
#[derive(Debug, Clone)]
pub struct LayerSpan {
    pub id: String,
    pub start: usize,
    pub end: usize,
}

/// Where the first device block and its layers are.
#[derive(Debug)]
pub struct Layout {
    pub device_start: usize,
    /// Index of the `device_end()` line (the line count when it is missing).
    pub device_end: usize,
    pub layers: Vec<LayerSpan>,
}

/// The lines a mapping edit works on: a layer's body, or the device's base
/// mappings (the device body outside every layer).
#[derive(Debug)]
pub struct Scope {
    /// Candidate lines (a base scope still contains the layer blocks; use
    /// [`Layout::in_other_scope`] to skip them).
    pub range: Range<usize>,
    /// Where new lines go: the base scope ends at the first layer.
    pub insert_limit: usize,
    pub is_base: bool,
}

impl Scope {
    pub fn lines(&self) -> Range<usize> {
        self.range.clone()
    }
}

impl Layout {
    /// Scans `lines` for the first device block.
    pub fn scan(lines: &[String]) -> Result<Self, GeneratorError> {
        let mut device_start = None;
        let mut device_end = None;
        let mut layers = Vec::new();
        let mut open: Option<(String, usize)> = None;
        for (i, raw) in lines.iter().enumerate() {
            let line = raw.trim();
            if line.starts_with("//") {
                continue;
            }
            if device_start.is_none() {
                if line.starts_with("device_start(") {
                    device_start = Some(i);
                }
                continue;
            }
            if line.starts_with("when_start(") {
                if let Some((id, _)) = &open {
                    return Err(GeneratorError::SyntaxError(format!(
                        "when_start nested inside layer {id}"
                    )));
                }
                open = Some((first_quoted(line).unwrap_or_default(), i));
            } else if line.starts_with("when_end(") {
                if let Some((id, start)) = open.take() {
                    layers.push(LayerSpan { id, start, end: i });
                }
            } else if line.starts_with("device_end(") && open.is_none() {
                device_end = Some(i);
                break;
            }
        }
        if let Some((id, _)) = open {
            return Err(GeneratorError::UnclosedWhenBlock(id));
        }
        Ok(Self {
            device_start: device_start.ok_or(GeneratorError::DeviceNotFound)?,
            device_end: device_end.unwrap_or(lines.len()),
            layers,
        })
    }

    pub fn layer(&self, id: &str) -> Option<&LayerSpan> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// The scope for `layer` (`base` or empty = the device's base mappings).
    pub fn scope(&self, layer: &str) -> Result<Scope, GeneratorError> {
        if layer == "base" || layer.is_empty() {
            let first_layer = self.layers.first().map_or(self.device_end, |l| l.start);
            return Ok(Scope {
                range: self.device_start + 1..self.device_end,
                insert_limit: first_layer,
                is_base: true,
            });
        }
        let span = self
            .layer(layer)
            .ok_or_else(|| GeneratorError::LayerNotFound(layer.to_string()))?;
        Ok(Scope {
            range: span.start + 1..span.end,
            insert_limit: span.end,
            is_base: false,
        })
    }

    /// Whether line `i` belongs to a layer block that is not `scope` itself
    /// (only a base scope contains foreign blocks).
    pub fn in_other_scope(&self, scope: &Scope, i: usize) -> bool {
        scope.is_base && self.layers.iter().any(|l| (l.start..=l.end).contains(&i))
    }
}

/// The text of the first double-quoted string on `line`.
pub fn first_quoted(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let len = line[start..].find('"')?;
    Some(line[start..start + len].to_string())
}
