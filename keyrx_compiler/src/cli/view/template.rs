//! HTML page scaffolding: the document header (CSS, controls, legend) and
//! footer (view-mode/layer-toggle script) that wrap the rendered keyboard.

use std::path::Path;

use super::LayerMappings;

pub(super) fn generate_html_header(input: &Path, layer_mappings: &LayerMappings) -> String {
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

pub(super) fn generate_html_footer() -> String {
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
