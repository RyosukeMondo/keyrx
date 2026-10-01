//! View subcommand handler - generates HTML visualization of key mappings.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::error::ParseError as ParserParseError;
use crate::parser::Parser;
use keyrx_core::config::{BaseKeyMapping, Condition, KeyCode, KeyMapping};

mod keycodes;
mod render;
mod template;

use keycodes::keycode_to_label;
use render::generate_keyboard_html;

/// Errors that can occur during the view subcommand.
#[derive(Debug)]
pub enum ViewCommandError {
    ParseError(ParserParseError),
    IoError(io::Error),
}

impl std::fmt::Display for ViewCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(err) => write!(f, "Parse error: {:?}", err),
            Self::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for ViewCommandError {}

impl From<io::Error> for ViewCommandError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<ParserParseError> for ViewCommandError {
    fn from(err: ParserParseError) -> Self {
        Self::ParseError(err)
    }
}

/// Layer mappings storage: layer_name -> (KeyCode -> (output, class))
type LayerMappings = HashMap<String, HashMap<String, (String, String)>>;

/// Handles the view subcommand - generates HTML visualization.
pub fn handle_view(input: &Path, output: &Path, open: bool) -> Result<(), ViewCommandError> {
    let mut parser = Parser::new();
    let config = parser.parse_script(input)?;

    // Build mapping lookup: KeyCode -> (output_display, type_class)
    let mut base_mappings: HashMap<String, (String, String)> = HashMap::new();
    // Layer mappings: layer_name -> (KeyCode -> (output, class))
    let mut layer_mappings: LayerMappings = HashMap::new();

    for device in &config.devices {
        for mapping in &device.mappings {
            collect_mappings(mapping, &mut base_mappings, &mut layer_mappings);
        }
    }

    let html = generate_keyboard_html(input, &base_mappings, &layer_mappings);
    fs::write(output, &html)?;
    println!("Generated: {}", output.display());

    if open {
        if let Err(e) = open::that(output) {
            eprintln!("Could not open browser: {}", e);
        }
    }

    Ok(())
}

fn collect_mappings(
    mapping: &KeyMapping,
    base_map: &mut HashMap<String, (String, String)>,
    layer_map: &mut LayerMappings,
) {
    match mapping {
        KeyMapping::Base(base) => {
            let (from, output, class) = get_base_mapping_info(base);
            base_map.insert(keycode_to_id(&from), (output, class.to_string()));
        }
        KeyMapping::Conditional {
            condition,
            mappings,
        } => {
            let layer_name = get_layer_name(condition);
            let layer = layer_map.entry(layer_name).or_default();
            for m in mappings {
                let (from, output, class) = get_base_mapping_info(m);
                layer.insert(keycode_to_id(&from), (output, class.to_string()));
            }
        }
    }
}

fn get_layer_name(condition: &Condition) -> String {
    match condition {
        Condition::ModifierActive(id) => format!("MD_{:02X}", id),
        Condition::LockActive(id) => format!("LK_{:02X}", id),
        Condition::AllActive(items) => {
            // For complex conditions, just use the first item
            format!("MULTI_{}", items.len())
        }
        Condition::NotActive(_) => "NOT".to_string(),
        Condition::DeviceMatches(pattern) => {
            // Truncate long patterns for display
            if pattern.len() > 15 {
                format!("DEV_{}", &pattern[..12])
            } else {
                format!("DEV_{}", pattern)
            }
        }
        Condition::ImeActive => "IME".to_string(),
        Condition::InputLanguage(lang) => format!("LANG_{}", lang.to_uppercase()),
    }
}

fn keycode_to_id(keycode: &KeyCode) -> String {
    format!("{:?}", keycode)
}

fn get_base_mapping_info(base: &BaseKeyMapping) -> (KeyCode, String, &'static str) {
    match base {
        BaseKeyMapping::Simple { from, to } => (*from, keycode_to_label(to).to_string(), "simple"),
        BaseKeyMapping::Modifier { from, modifier_id } => {
            (*from, format!("M{:X}", modifier_id), "modifier")
        }
        BaseKeyMapping::Lock { from, lock_id } => (*from, format!("L{:X}", lock_id), "lock"),
        BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            ..
        } => (
            *from,
            format!("{}/M{:X}", keycode_to_label(tap), hold_modifier),
            "taphold",
        ),
        BaseKeyMapping::HoldOnly {
            from,
            hold_modifier,
            threshold_ms,
        } => (
            *from,
            format!("M{:X}/{}ms", hold_modifier, threshold_ms),
            "holdonly",
        ),
        BaseKeyMapping::TapHoldKey {
            from, tap, hold, ..
        } => {
            let hold = keycode_to_label(hold);
            match tap {
                Some(tap) => (
                    *from,
                    format!("{}/{}", keycode_to_label(tap), hold),
                    "taphold",
                ),
                None => (*from, format!("-/{}", hold), "holdonly"),
            }
        }
        BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        } => {
            let mut prefix = String::new();
            if *shift {
                prefix.push('S');
            }
            if *ctrl {
                prefix.push('C');
            }
            if *alt {
                prefix.push('A');
            }
            if *win {
                prefix.push('W');
            }
            (
                *from,
                format!("{}+{}", prefix, keycode_to_label(to)),
                "modified",
            )
        }
        BaseKeyMapping::Sequence { from, keys } => {
            let seq_str: Vec<&str> = keys.iter().map(|k| keycode_to_label(k)).collect();
            (*from, seq_str.join("→"), "sequence")
        }
    }
}
