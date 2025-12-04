// ============================================================================
// Stream Deck Style Device - 3x3 Grid Configuration
// ============================================================================
//
// Device Profile: ~/.config/keyrx/devices/custom_streamdeck.json
//   {
//     "vendor_id": 1234,
//     "product_id": 5678,
//     "name": "Custom 3x3 Macro Pad",
//     "rows": 3,
//     "cols_per_row": [3, 3, 3],
//     "keymap": {
//       "1": { "scan_code": 1, "row": 0, "col": 0, "alias": "r0_c0" },
//       "2": { "scan_code": 2, "row": 0, "col": 1, "alias": "r0_c1" },
//       ...
//     }
//   }
//
// Physical Layout:
//   ┌─────┬─────┬─────┐
//   │ 0,0 │ 0,1 │ 0,2 │  ← Row 0 (Media controls)
//   ├─────┼─────┼─────┤
//   │ 1,0 │ 1,1 │ 1,2 │  ← Row 1 (Window management)
//   ├─────┼─────┼─────┤
//   │ 2,0 │ 2,1 │ 2,2 │  ← Row 2 (App shortcuts)
//   └─────┴─────┴─────┘
//
// ============================================================================

fn on_init() {
    print_debug("=== 3x3 Stream Deck Configuration ===");

    // ========================================================================
    // ROW 0: Media Controls
    // ========================================================================
    remap_rc(0, 0, "MediaPlayPause");  // Top-left: Play/Pause
    remap_rc(0, 1, "MediaNext");       // Top-middle: Next track
    remap_rc(0, 2, "MediaPrev");       // Top-right: Previous track

    // ========================================================================
    // ROW 1: Window Management (Linux/i3wm example)
    // ========================================================================
    // These send custom key combos for window manager shortcuts
    combo_rc([1, 0], "LeftMeta", "H");  // Middle-left: Focus left
    combo_rc([1, 1], "LeftMeta", "J");  // Middle-middle: Focus down
    combo_rc([1, 2], "LeftMeta", "K");  // Middle-right: Focus up

    // ========================================================================
    // ROW 2: Application Shortcuts
    // ========================================================================
    combo_rc([2, 0], "LeftMeta", "Enter");  // Bottom-left: Terminal
    combo_rc([2, 1], "LeftMeta", "B");      // Bottom-middle: Browser
    combo_rc([2, 2], "LeftMeta", "E");      // Bottom-right: Editor

    print_debug("Stream deck configured: 3x3 grid");
}

// ============================================================================
// ALTERNATIVE: Layer-based approach
// ============================================================================
//
// fn on_init() {
//     layer_define("media");
//     layer_define("windows");
//     layer_define("apps");
//
//     // Top row activates layers
//     tap_hold_rc(0, 0, "MediaPlayPause", layer_push("media"));
//     tap_hold_rc(0, 1, "Escape", layer_push("windows"));
//     tap_hold_rc(0, 2, "Escape", layer_push("apps"));
//
//     // Configure each layer
//     layer_map_rc("media", 1, 0, "VolumeDown");
//     layer_map_rc("media", 1, 1, "VolumeMute");
//     layer_map_rc("media", 1, 2, "VolumeUp");
//
//     layer_map_rc("windows", 1, 0, combo("Meta", "1"));
//     layer_map_rc("windows", 1, 1, combo("Meta", "2"));
//     // ... etc
// }
//
// ============================================================================
// BENEFITS FOR STREAM DECK DEVICES
// ============================================================================
//
// ✅ No key names needed - only positions
// ✅ Visual mapping matches physical grid exactly
// ✅ Easy to understand: [row, col] = button position
// ✅ Works with ANY grid size (2x2, 3x3, 4x4, 5x3, etc.)
// ✅ Device-agnostic: Change device, same script structure
