//! Script context and operation location types.
//!
//! Types for tracking script context during validation, including
//! layer and modifier definitions, and source locations.

use std::collections::HashSet;

use crate::scripting::{PendingOp, TimingUpdate};

use super::super::types::SourceLocation;

/// Operation with associated metadata for validation.
#[derive(Debug, Clone)]
pub struct LocatedOp {
    /// The operation itself.
    pub op: PendingOp,
    /// Operation index (order in script).
    pub index: usize,
}

impl LocatedOp {
    /// Create a new located operation.
    pub fn new(op: PendingOp, index: usize) -> Self {
        Self { op, index }
    }
}

/// Script context containing parsed metadata.
#[derive(Debug, Clone, Default)]
pub struct ScriptContext {
    /// Defined layer names.
    pub layers: HashSet<String>,
    /// Defined modifier names.
    pub modifiers: HashSet<String>,
    /// Script lines for source location context.
    pub lines: Vec<String>,
}

impl ScriptContext {
    /// Create a new script context from script source.
    pub fn from_script(script: &str) -> Self {
        Self {
            layers: HashSet::new(),
            modifiers: HashSet::new(),
            lines: script.lines().map(String::from).collect(),
        }
    }

    /// Get a line from the script (1-indexed).
    pub fn get_line(&self, line_num: usize) -> Option<&str> {
        if line_num > 0 && line_num <= self.lines.len() {
            Some(&self.lines[line_num - 1])
        } else {
            None
        }
    }

    /// Create a source location with context from this script.
    pub fn source_location(&self, line: usize, column: Option<usize>) -> SourceLocation {
        let mut loc = SourceLocation::new(line);
        if let Some(col) = column {
            loc = loc.with_column(col);
        }
        if let Some(context) = self.get_line(line) {
            loc = loc.with_context(context.trim());
        }
        loc
    }
}

/// Result of parsing a script, containing operations and context.
#[derive(Debug, Clone)]
pub struct ParsedScript {
    /// Collected operations.
    pub ops: Vec<PendingOp>,
    /// Script context with definitions.
    pub context: ScriptContext,
}

/// Collect layer and modifier definitions from operations.
pub fn collect_definitions(ops: &[PendingOp]) -> (HashSet<String>, HashSet<String>) {
    let mut layers = HashSet::new();
    let mut modifiers = HashSet::new();

    for op in ops {
        match op {
            PendingOp::LayerDefine { name, .. } => {
                layers.insert(name.clone());
            }
            PendingOp::DefineModifier { name, .. } => {
                modifiers.insert(name.clone());
            }
            _ => {}
        }
    }

    (layers, modifiers)
}

/// Populate script context with layer and modifier definitions from operations.
pub fn populate_context_from_ops(ops: &[PendingOp], context: &mut ScriptContext) {
    for op in ops {
        match op {
            PendingOp::LayerDefine { name, .. } => {
                context.layers.insert(name.clone());
            }
            PendingOp::DefineModifier { name, .. } => {
                context.modifiers.insert(name.clone());
            }
            _ => {}
        }
    }
}

/// Find the approximate line number for an operation by searching for patterns.
///
/// This is a best-effort heuristic since Rhai doesn't provide position info
/// during function execution. Returns None if no match is found.
pub fn find_operation_line(script: &str, op: &PendingOp) -> Option<usize> {
    let pattern = match op {
        PendingOp::Remap { from, to } => {
            format!("remap(\"{}\", \"{}\")", from.name(), to.name())
        }
        PendingOp::Block { key } => {
            format!("block(\"{}\")", key.name())
        }
        PendingOp::Pass { key } => {
            format!("pass(\"{}\")", key.name())
        }
        PendingOp::TapHold { key, tap, .. } => {
            format!("tap_hold(\"{}\", \"{}\"", key.name(), tap.name())
        }
        PendingOp::LayerDefine { name, .. } => {
            format!("define_layer(\"{}\"", name)
        }
        PendingOp::LayerPush { name } => {
            format!("layer_push(\"{}\")", name)
        }
        PendingOp::LayerToggle { name } => {
            format!("layer_toggle(\"{}\")", name)
        }
        PendingOp::LayerMap { layer, key, .. } => {
            format!("layer_map(\"{}\", \"{}\"", layer, key.name())
        }
        PendingOp::LayoutDefine { id, .. } => {
            format!("layout_define(\"{}\"", id)
        }
        PendingOp::LayoutEnable { id } => {
            format!("layout_enable(\"{}\")", id)
        }
        PendingOp::LayoutDisable { id } => {
            format!("layout_disable(\"{}\")", id)
        }
        PendingOp::LayoutRemove { id } => {
            format!("layout_remove(\"{}\")", id)
        }
        PendingOp::LayoutSetPriority { id, .. } => {
            format!("layout_set_priority(\"{}\"", id)
        }
        PendingOp::DefineModifier { name, .. } => {
            format!("define_modifier(\"{}\")", name)
        }
        PendingOp::ModifierActivate { name, .. } => {
            format!("modifier_activate(\"{}\")", name)
        }
        PendingOp::ModifierDeactivate { name, .. } => {
            format!("modifier_deactivate(\"{}\")", name)
        }
        PendingOp::ModifierOneShot { name, .. } => {
            format!("modifier_one_shot(\"{}\")", name)
        }
        PendingOp::SetTiming(timing) => match timing {
            TimingUpdate::TapTimeout(ms) => format!("tap_timeout({})", ms),
            TimingUpdate::ComboTimeout(ms) => format!("combo_timeout({})", ms),
            TimingUpdate::HoldDelay(ms) => format!("hold_delay({})", ms),
            _ => return None,
        },
        PendingOp::Combo { .. } | PendingOp::LayerPop => return None,
    };

    for (idx, line) in script.lines().enumerate() {
        if line.contains(&pattern) {
            return Some(idx + 1); // 1-indexed
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

    #[test]
    fn script_context_provides_line_access() {
        let script = "line1\nline2\nline3";
        let context = ScriptContext::from_script(script);
        assert_eq!(context.get_line(1), Some("line1"));
        assert_eq!(context.get_line(2), Some("line2"));
        assert_eq!(context.get_line(3), Some("line3"));
        assert_eq!(context.get_line(0), None);
        assert_eq!(context.get_line(4), None);
    }

    #[test]
    fn script_context_creates_source_location() {
        let script = "remap(\"A\", \"B\");\nblock(\"C\");";
        let context = ScriptContext::from_script(script);
        let loc = context.source_location(1, Some(5));
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, Some(5));
        assert_eq!(loc.context, Some("remap(\"A\", \"B\");".into()));
    }

    #[test]
    fn find_operation_line_locates_remap() {
        let script = r#"
            // Comment
            remap("CapsLock", "Escape");
            block("Insert");
        "#;
        let op = PendingOp::Remap {
            from: KeyCode::CapsLock,
            to: KeyCode::Escape,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(3));
    }

    #[test]
    fn find_operation_line_locates_block() {
        let script = r#"
            remap("A", "B");
            block("Insert");
        "#;
        let op = PendingOp::Block {
            key: KeyCode::Insert,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(3));
    }

    #[test]
    fn find_operation_line_locates_layer_define() {
        let script = r#"
            define_layer("navigation");
            layer_push("navigation");
        "#;
        let op = PendingOp::LayerDefine {
            name: "navigation".to_string(),
            transparent: false,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(2));
    }

    #[test]
    fn find_operation_line_locates_modifier_ops() {
        let script = r#"
            define_modifier("hyper");
            modifier_activate("hyper");
        "#;
        let op = PendingOp::ModifierActivate {
            name: "hyper".to_string(),
            id: 0,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(3));
    }

    #[test]
    fn find_operation_line_returns_none_for_no_match() {
        let script = "remap(\"A\", \"B\");";
        let op = PendingOp::Block { key: KeyCode::C };
        let line = find_operation_line(script, &op);
        assert_eq!(line, None);
    }

    #[test]
    fn located_op_stores_index() {
        let op = PendingOp::Block { key: KeyCode::A };
        let located = LocatedOp::new(op.clone(), 5);
        assert_eq!(located.index, 5);
        match located.op {
            PendingOp::Block { key } => assert_eq!(key, KeyCode::A),
            _ => panic!("wrong op type"),
        }
    }
}
