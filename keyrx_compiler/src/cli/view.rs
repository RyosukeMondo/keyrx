//! View subcommand handler - generates HTML visualization of key mappings.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::error::ParseError as ParserParseError;
use crate::parser::Parser;
use keyrx_core::config::{BaseKeyMapping, Condition, KeyCode, KeyMapping};

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
    }
}

fn keycode_to_id(keycode: &KeyCode) -> String {
    format!("{:?}", keycode)
}

fn keycode_to_label(keycode: &KeyCode) -> &'static str {
    match keycode {
        KeyCode::A => "A",
        KeyCode::B => "B",
        KeyCode::C => "C",
        KeyCode::D => "D",
        KeyCode::E => "E",
        KeyCode::F => "F",
        KeyCode::G => "G",
        KeyCode::H => "H",
        KeyCode::I => "I",
        KeyCode::J => "J",
        KeyCode::K => "K",
        KeyCode::L => "L",
        KeyCode::M => "M",
        KeyCode::N => "N",
        KeyCode::O => "O",
        KeyCode::P => "P",
        KeyCode::Q => "Q",
        KeyCode::R => "R",
        KeyCode::S => "S",
        KeyCode::T => "T",
        KeyCode::U => "U",
        KeyCode::V => "V",
        KeyCode::W => "W",
        KeyCode::X => "X",
        KeyCode::Y => "Y",
        KeyCode::Z => "Z",
        KeyCode::Num0 => "0",
        KeyCode::Num1 => "1",
        KeyCode::Num2 => "2",
        KeyCode::Num3 => "3",
        KeyCode::Num4 => "4",
        KeyCode::Num5 => "5",
        KeyCode::Num6 => "6",
        KeyCode::Num7 => "7",
        KeyCode::Num8 => "8",
        KeyCode::Num9 => "9",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::Escape => "Esc",
        KeyCode::Tab => "Tab",
        KeyCode::CapsLock => "Caps",
        KeyCode::LShift => "LShift",
        KeyCode::RShift => "RShift",
        KeyCode::LCtrl => "LCtrl",
        KeyCode::RCtrl => "RCtrl",
        KeyCode::LAlt => "LAlt",
        KeyCode::RAlt => "RAlt",
        KeyCode::LMeta => "Win",
        KeyCode::RMeta => "Win",
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Backspace => "BS",
        KeyCode::Delete => "Del",
        KeyCode::Insert => "Ins",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PgUp",
        KeyCode::PageDown => "PgDn",
        KeyCode::Up => "↑",
        KeyCode::Down => "↓",
        KeyCode::Left => "←",
        KeyCode::Right => "→",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::LeftBracket => "[",
        KeyCode::RightBracket => "]",
        KeyCode::Backslash => "\\",
        KeyCode::Semicolon => ";",
        KeyCode::Quote => "'",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",
        KeyCode::Grave => "`",
        KeyCode::PrintScreen => "PrtSc",
        KeyCode::ScrollLock => "ScrLk",
        KeyCode::Pause => "Pause",
        KeyCode::NumLock => "Num",
        KeyCode::Numpad0 => "0",
        KeyCode::Numpad1 => "1",
        KeyCode::Numpad2 => "2",
        KeyCode::Numpad3 => "3",
        KeyCode::Numpad4 => "4",
        KeyCode::Numpad5 => "5",
        KeyCode::Numpad6 => "6",
        KeyCode::Numpad7 => "7",
        KeyCode::Numpad8 => "8",
        KeyCode::Numpad9 => "9",
        KeyCode::NumpadAdd => "+",
        KeyCode::NumpadSubtract => "-",
        KeyCode::NumpadMultiply => "*",
        KeyCode::NumpadDivide => "/",
        KeyCode::NumpadEnter => "Ent",
        KeyCode::NumpadDecimal => ".",
        // Japanese keys
        KeyCode::Zenkaku => "半/全",
        KeyCode::Katakana => "カナ",
        KeyCode::Hiragana => "ひら",
        KeyCode::Henkan => "変換",
        KeyCode::Muhenkan => "無変換",
        KeyCode::Yen => "¥",
        KeyCode::Ro => "ろ",
        KeyCode::KatakanaHiragana => "カナ",
        // Korean keys
        KeyCode::Hangeul => "한글",
        KeyCode::Hanja => "한자",
        _ => "?",
    }
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
    }
}

fn generate_keyboard_html(
    input: &Path,
    base_mappings: &HashMap<String, (String, String)>,
    layer_mappings: &LayerMappings,
) -> String {
    // JIS 109-key layout definition (6 rows)
    // Each key: (id, label, width_units)
    let rows: Vec<Vec<(&str, &str, f32)>> = vec![
        // Row 0: Function row
        vec![
            ("Escape", "Esc", 1.0),
            ("", "", 0.5),
            ("F1", "F1", 1.0),
            ("F2", "F2", 1.0),
            ("F3", "F3", 1.0),
            ("F4", "F4", 1.0),
            ("", "", 0.5),
            ("F5", "F5", 1.0),
            ("F6", "F6", 1.0),
            ("F7", "F7", 1.0),
            ("F8", "F8", 1.0),
            ("", "", 0.5),
            ("F9", "F9", 1.0),
            ("F10", "F10", 1.0),
            ("F11", "F11", 1.0),
            ("F12", "F12", 1.0),
            ("", "", 0.25),
            ("PrintScreen", "PrtSc", 1.0),
            ("ScrollLock", "ScrLk", 1.0),
            ("Pause", "Pause", 1.0),
        ],
        // Row 1: Number row
        vec![
            ("Zenkaku", "半/全", 1.0),
            ("Num1", "1", 1.0),
            ("Num2", "2", 1.0),
            ("Num3", "3", 1.0),
            ("Num4", "4", 1.0),
            ("Num5", "5", 1.0),
            ("Num6", "6", 1.0),
            ("Num7", "7", 1.0),
            ("Num8", "8", 1.0),
            ("Num9", "9", 1.0),
            ("Num0", "0", 1.0),
            ("Minus", "-", 1.0),
            ("Equal", "=", 1.0),
            ("Yen", "¥", 1.0),
            ("Backspace", "BS", 1.0),
            ("", "", 0.25),
            ("Insert", "Ins", 1.0),
            ("Home", "Home", 1.0),
            ("PageUp", "PgUp", 1.0),
            ("", "", 0.25),
            ("NumLock", "Num", 1.0),
            ("NumpadDivide", "/", 1.0),
            ("NumpadMultiply", "*", 1.0),
            ("NumpadSubtract", "-", 1.0),
        ],
        // Row 2: QWERTY row
        vec![
            ("Tab", "Tab", 1.5),
            ("Q", "Q", 1.0),
            ("W", "W", 1.0),
            ("E", "E", 1.0),
            ("R", "R", 1.0),
            ("T", "T", 1.0),
            ("Y", "Y", 1.0),
            ("U", "U", 1.0),
            ("I", "I", 1.0),
            ("O", "O", 1.0),
            ("P", "P", 1.0),
            ("LeftBracket", "[", 1.0),
            ("RightBracket", "]", 1.0),
            ("Enter", "Enter", 1.5),
            ("", "", 0.25),
            ("Delete", "Del", 1.0),
            ("End", "End", 1.0),
            ("PageDown", "PgDn", 1.0),
            ("", "", 0.25),
            ("Numpad7", "7", 1.0),
            ("Numpad8", "8", 1.0),
            ("Numpad9", "9", 1.0),
            ("NumpadAdd", "+", 1.0),
        ],
        // Row 3: Home row
        vec![
            ("CapsLock", "Caps", 1.75),
            ("A", "A", 1.0),
            ("S", "S", 1.0),
            ("D", "D", 1.0),
            ("F", "F", 1.0),
            ("G", "G", 1.0),
            ("H", "H", 1.0),
            ("J", "J", 1.0),
            ("K", "K", 1.0),
            ("L", "L", 1.0),
            ("Semicolon", ";", 1.0),
            ("Quote", "'", 1.0),
            ("Backslash", "\\", 1.0),
            ("", "", 4.5),
            ("Numpad4", "4", 1.0),
            ("Numpad5", "5", 1.0),
            ("Numpad6", "6", 1.0),
            ("", "", 1.0),
        ],
        // Row 4: Bottom letter row
        vec![
            ("LShift", "LShift", 2.25),
            ("Z", "Z", 1.0),
            ("X", "X", 1.0),
            ("C", "C", 1.0),
            ("V", "V", 1.0),
            ("B", "B", 1.0),
            ("N", "N", 1.0),
            ("M", "M", 1.0),
            ("Comma", ",", 1.0),
            ("Period", ".", 1.0),
            ("Slash", "/", 1.0),
            ("Ro", "ろ", 1.0),
            ("RShift", "RShift", 1.75),
            ("", "", 0.25),
            ("Up", "↑", 1.0),
            ("", "", 1.25),
            ("Numpad1", "1", 1.0),
            ("Numpad2", "2", 1.0),
            ("Numpad3", "3", 1.0),
            ("NumpadEnter", "Ent", 1.0),
        ],
        // Row 5: Space row
        vec![
            ("LCtrl", "LCtrl", 1.25),
            ("LMeta", "Win", 1.25),
            ("LAlt", "LAlt", 1.25),
            ("Muhenkan", "無変換", 1.25),
            ("Space", "Space", 5.0),
            ("Henkan", "変換", 1.25),
            ("Hiragana", "ひら", 1.0),
            ("RAlt", "RAlt", 1.25),
            ("RMeta", "Win", 1.25),
            ("Menu", "Menu", 1.25),
            ("RCtrl", "RCtrl", 1.25),
            ("", "", 0.25),
            ("Left", "←", 1.0),
            ("Down", "↓", 1.0),
            ("Right", "→", 1.0),
            ("", "", 0.25),
            ("Numpad0", "0", 2.0),
            ("NumpadDecimal", ".", 1.0),
            ("", "", 1.0),
        ],
    ];

    let mut html = generate_html_header(input, layer_mappings);

    html.push_str(r#"<div class="keyboard">"#);

    for row in &rows {
        html.push_str(r#"<div class="row">"#);
        for (id, label, width) in row {
            if id.is_empty() {
                // Spacer
                html.push_str(&format!(
                    r#"<div class="spacer" style="width: {}px;"></div>"#,
                    width * 50.0
                ));
            } else {
                let (remap, class) = base_mappings
                    .get(*id)
                    .map(|(r, c)| (r.as_str(), c.as_str()))
                    .unwrap_or(("", ""));
                let remapped_class = if remap.is_empty() { "" } else { " remapped" };

                // Collect layer remaps for this key
                let layer_data = generate_layer_data_attrs(id, layer_mappings);

                html.push_str(&format!(
                    r#"<div class="key {}{}" style="width: {}px;" data-id="{}" data-original="{}" data-base-remap="{}"{}>
                        <span class="original">{}</span>
                        <span class="remap">{}</span>
                    </div>"#,
                    class, remapped_class, width * 50.0, id, label, remap, layer_data, label, remap
                ));
            }
        }
        html.push_str("</div>\n");
    }

    html.push_str("</div>\n");
    html.push_str(&generate_html_footer());
    html
}

fn generate_layer_data_attrs(key_id: &str, layer_mappings: &LayerMappings) -> String {
    let mut attrs = String::new();
    for (layer_name, mappings) in layer_mappings {
        if let Some((remap, class)) = mappings.get(key_id) {
            attrs.push_str(&format!(
                r#" data-layer-{}="{}" data-layer-{}-class="{}""#,
                layer_name.to_lowercase().replace('_', "-"),
                remap,
                layer_name.to_lowercase().replace('_', "-"),
                class
            ));
        }
    }
    attrs
}

fn generate_html_header(input: &Path, layer_mappings: &LayerMappings) -> String {
    // Generate layer button HTML
    let layer_buttons: String = layer_mappings
        .keys()
        .map(|name| {
            let btn_id = name.to_lowercase().replace('_', "-");
            format!(
                r#"<button onclick="toggleLayer('{}')" id="btn-layer-{}" class="layer-btn">{} Layer</button>"#,
                btn_id, btn_id, name
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>KeyRx Layout: {}</title>
<style>
* {{ box-sizing: border-box; }}
body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Hiragino Sans', sans-serif;
    margin: 0;
    padding: 20px;
    background: #1a1a2e;
    color: #eee;
}}
h1 {{ color: #00d9ff; margin-bottom: 5px; }}
.source {{ color: #888; font-size: 0.9em; margin-bottom: 20px; }}
.controls {{
    margin: 20px 0;
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}}
.controls button {{
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    background: #0f3460;
    color: #fff;
    transition: background 0.2s;
}}
.controls button:hover {{ background: #1f4068; }}
.controls button.active {{ background: #00d9ff; color: #000; }}
.layer-controls {{
    margin: 15px 0;
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    padding: 10px;
    background: rgba(255, 107, 107, 0.1);
    border-radius: 8px;
    border: 1px solid rgba(255, 107, 107, 0.3);
}}
.layer-controls .label {{
    color: #ff6b6b;
    font-weight: bold;
    margin-right: 10px;
    display: flex;
    align-items: center;
}}
.layer-btn {{
    background: #4a2020 !important;
    border: 1px solid #ff6b6b !important;
}}
.layer-btn:hover {{ background: #6a3030 !important; }}
.layer-btn.active {{ background: #ff6b6b !important; color: #000 !important; }}
.legend {{
    display: flex;
    gap: 15px;
    flex-wrap: wrap;
    margin: 15px 0;
    font-size: 0.85em;
}}
.legend-item {{
    display: flex;
    align-items: center;
    gap: 6px;
}}
.legend-color {{
    width: 16px;
    height: 16px;
    border-radius: 3px;
}}
.keyboard {{
    display: inline-block;
    background: #16213e;
    padding: 15px;
    border-radius: 10px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.3);
}}
.row {{
    display: flex;
    margin-bottom: 4px;
}}
.key {{
    height: 50px;
    margin: 2px;
    background: #2d3a5a;
    border-radius: 5px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    cursor: default;
    transition: all 0.15s;
    border: 1px solid #3d4a6a;
    position: relative;
}}
.key:hover {{
    background: #3d4a7a;
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.3);
}}
.key .original {{
    color: #888;
    font-size: 10px;
}}
.key .remap {{
    color: #ffd93d;
    font-weight: bold;
    font-size: 12px;
    min-height: 14px;
}}
.key.remapped {{
    background: #1f4068;
    border-color: #00d9ff;
}}
.key.simple.remapped {{ border-color: #4ade80; }}
.key.modifier.remapped {{ border-color: #00d9ff; background: rgba(0, 217, 255, 0.15); }}
.key.lock.remapped {{ border-color: #a78bfa; background: rgba(167, 139, 250, 0.15); }}
.key.taphold.remapped {{ border-color: #ff6b6b; background: rgba(255, 107, 107, 0.15); }}
.key.modified.remapped {{ border-color: #4ade80; background: rgba(74, 222, 128, 0.15); }}
.key.layer-active {{ border-color: #fbbf24 !important; background: rgba(251, 191, 36, 0.2) !important; }}
.spacer {{ height: 50px; }}

/* View modes */
body.show-original .key .remap {{ display: none; }}
body.show-original .key .original {{ font-size: 12px; color: #fff; }}
body.show-remap .key .original {{ display: none; }}
body.show-remap .key .remap {{ font-size: 12px; }}
body.show-remap .key:not(.remapped) .remap {{ color: #666; }}
body.show-remap .key:not(.remapped)::after {{ content: attr(data-original); color: #666; font-size: 12px; }}
body.show-code .key .original, body.show-code .key .remap {{ display: none; }}
body.show-code .key::after {{ content: attr(data-id); color: #888; font-size: 9px; }}
</style>
</head>
<body class="show-both">
<h1>KeyRx Layout Viewer</h1>
<p class="source">Source: <code>{}</code></p>
<div class="controls">
    <button onclick="setView('both')" class="active" id="btn-both">Original + Remap</button>
    <button onclick="setView('original')" id="btn-original">Original Only</button>
    <button onclick="setView('remap')" id="btn-remap">Remap Only</button>
    <button onclick="setView('code')" id="btn-code">KeyCode</button>
</div>
<div class="layer-controls">
    <span class="label">Layers:</span>
    <button onclick="toggleLayer('base')" class="layer-btn active" id="btn-layer-base">Base Layer</button>
    {}
</div>
<div class="legend">
    <div class="legend-item"><div class="legend-color" style="background: #4ade80;"></div> Simple</div>
    <div class="legend-item"><div class="legend-color" style="background: #00d9ff;"></div> Modifier</div>
    <div class="legend-item"><div class="legend-color" style="background: #a78bfa;"></div> Lock</div>
    <div class="legend-item"><div class="legend-color" style="background: #ff6b6b;"></div> TapHold</div>
    <div class="legend-item"><div class="legend-color" style="background: #4ade80;"></div> Modified</div>
    <div class="legend-item"><div class="legend-color" style="background: #fbbf24;"></div> Layer Active</div>
    <span style="color: #666; margin-left: 20px;">Bordered = Remapped</span>
</div>
"#,
        input.file_name().unwrap_or_default().to_string_lossy(),
        input.display(),
        layer_buttons
    )
}

fn generate_html_footer() -> String {
    r#"
<script>
let currentLayer = 'base';

function setView(mode) {
    const classList = document.body.className.split(' ').filter(c => c.startsWith('show-') === false);
    classList.push('show-' + mode);
    document.body.className = classList.join(' ');
    document.querySelectorAll('.controls button').forEach(b => b.classList.remove('active'));
    document.getElementById('btn-' + mode).classList.add('active');
}

function toggleLayer(layerId) {
    currentLayer = layerId;
    document.querySelectorAll('.layer-btn').forEach(b => b.classList.remove('active'));
    document.getElementById('btn-layer-' + layerId).classList.add('active');

    // Update all keys
    document.querySelectorAll('.key').forEach(key => {
        const baseRemap = key.getAttribute('data-base-remap') || '';
        const remapSpan = key.querySelector('.remap');

        // Reset layer-active class
        key.classList.remove('layer-active');

        if (layerId === 'base') {
            // Show base layer mappings
            if (remapSpan) remapSpan.textContent = baseRemap;
            if (baseRemap) {
                key.classList.add('remapped');
            } else {
                key.classList.remove('remapped');
            }
        } else {
            // Show layer mappings
            const layerRemap = key.getAttribute('data-layer-' + layerId);
            if (layerRemap) {
                if (remapSpan) remapSpan.textContent = layerRemap;
                key.classList.add('remapped', 'layer-active');
            } else if (baseRemap) {
                // Fallback to base remap if no layer remap
                if (remapSpan) remapSpan.textContent = baseRemap;
                key.classList.add('remapped');
                key.classList.remove('layer-active');
            } else {
                if (remapSpan) remapSpan.textContent = '';
                key.classList.remove('remapped', 'layer-active');
            }
        }
    });
}
</script>
<footer style="margin-top: 30px; color: #555; font-size: 0.8em;">
Generated by keyrx_compiler view - JIS 109 Layout
</footer>
</body>
</html>
"#.to_string()
}
