//! Keyboard layout rendering: lays out the JIS 109-key grid and stitches the
//! header/footer templates around it.

use std::collections::HashMap;
use std::path::Path;

use super::template::{generate_html_footer, generate_html_header};
use super::LayerMappings;

pub(super) fn generate_keyboard_html(
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
