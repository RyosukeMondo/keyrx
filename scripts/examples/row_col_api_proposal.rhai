// ============================================================================
// Row-Column API - User-Facing Script Example
// ============================================================================
//
// Device Profile: ~/.config/keyrx/devices/099a_0638.json
//   - 6 rows, varying columns per row
//   - Full-size keyboard
//
// How it works:
//   1. Script load time: row-col → scan_code → KeyCode (via device profile)
//   2. Runtime: Only KeyCode remapping (same performance as current API)
//
// ============================================================================

fn on_init() {
    print_debug("=== Row-Column Based Configuration ===");

    // ========================================================================
    // BASIC REMAPPING - Position-based
    // ========================================================================

    // Remap by row-col position (instead of key name)
    remap_rc(3, 0, "Escape");           // r3_c0 (CapsLock) → Escape
    remap_rc(0, 0, "CapsLock");         // r0_c0 (Escape) → CapsLock

    // Works with any device profile - position is universal!
    // r3_c0 could be CapsLock on QWERTY, or some other key on custom layout

    // ========================================================================
    // TAP-HOLD - Position-based home row mods
    // ========================================================================

    // Left hand home row (row 3, cols 1-4)
    tap_hold_rc(3, 1, "A", "LeftCtrl");    // r3_c1: tap=A, hold=Ctrl
    tap_hold_rc(3, 2, "S", "LeftAlt");     // r3_c2: tap=S, hold=Alt
    tap_hold_rc(3, 3, "D", "LeftMeta");    // r3_c3: tap=D, hold=Meta
    tap_hold_rc(3, 4, "F", "LeftShift");   // r3_c4: tap=F, hold=Shift

    // Right hand home row (row 3, cols 7-10)
    tap_hold_rc(3, 7, "J", "RightShift"); // r3_c7: tap=J, hold=Shift
    tap_hold_rc(3, 8, "K", "RightMeta");  // r3_c8: tap=K, hold=Meta
    tap_hold_rc(3, 9, "L", "RightAlt");   // r3_c9: tap=L, hold=Alt
    tap_hold_rc(3, 10, "Semicolon", "RightCtrl"); // r3_c10

    // ========================================================================
    // BLOCK - Position-based
    // ========================================================================

    // Block keys by physical position (corner keys often hit by accident)
    block_rc(1, 13);  // r1_c13 (Insert)
    block_rc(0, 16);  // r0_c16 (Delete)
    block_rc(0, 15);  // r0_c15 (Pause)

    // ========================================================================
    // COMBOS - Position-based chord detection
    // ========================================================================

    // Two-key combo: bottom corners trigger screenshot
    combo_rc([5, 0], [5, 10], "PrintScreen");  // LeftCtrl + RightAlt → Screenshot

    // Three-key combo: home row positions
    combo_rc([3, 1], [3, 2], [3, 3], layer_push("nav"));  // A+S+D → Nav layer

    // ========================================================================
    // LAYER MAPPING - Position-based layer activation
    // ========================================================================

    layer_define("nav");

    // Make row 3 into arrow keys when layer is active
    layer_map_rc("nav", 3, 6, "Left");   // r3_c6 (H) → Left arrow
    layer_map_rc("nav", 3, 7, "Down");   // r3_c7 (J) → Down arrow
    layer_map_rc("nav", 3, 8, "Up");     // r3_c8 (K) → Up arrow
    layer_map_rc("nav", 3, 9, "Right");  // r3_c9 (L) → Right arrow

    // Layer toggle on position (instead of key name)
    tap_hold_rc(5, 4, "Space", layer_toggle("nav")); // r5_c4 (Space)

    print_debug("Row-column configuration loaded!");
}

// ============================================================================
// API SUMMARY
// ============================================================================
//
// Function Signatures:
//
//   remap_rc(row: int, col: int, to: string) -> Result
//     Example: remap_rc(3, 0, "Escape")
//
//   tap_hold_rc(row: int, col: int, tap: string, hold: string) -> Result
//     Example: tap_hold_rc(3, 1, "A", "LeftCtrl")
//
//   block_rc(row: int, col: int) -> Result
//     Example: block_rc(0, 16)
//
//   combo_rc(keys: [row-col pairs...], action: string) -> Result
//     Example: combo_rc([3, 1], [3, 2], "Escape")
//
//   layer_map_rc(layer: string, row: int, col: int, to: string) -> Result
//     Example: layer_map_rc("nav", 3, 6, "Left")
//
// Error Handling:
//   - If row-col not found in device profile → Runtime error
//   - Error message: "Position r3_c99 not found in device profile"
//   - Suggests: "Run 'keyrx devices' to see available positions"
//
// ============================================================================
// BENEFITS
// ============================================================================
//
// ✅ Layout-agnostic: Works with QWERTY, Dvorak, Colemak, custom layouts
// ✅ Position-based: Think in terms of finger positions, not key labels
// ✅ Portable: Same script works on different keyboard firmware/layouts
// ✅ Visual: Easy to visualize on keyboard layout diagram
// ✅ Stream deck support: Can use ANY grid layout (3x3, 4x4, etc.)
//
// Example: Stream Deck (3x3 grid)
//   Row 0: [0,0] [0,1] [0,2]
//   Row 1: [1,0] [1,1] [1,2]
//   Row 2: [2,0] [2,1] [2,2]
//
//   remap_rc(0, 0, "MediaPlayPause");
//   remap_rc(0, 1, "MediaNext");
//   remap_rc(0, 2, "MediaPrev");
//   // ... configure all 9 buttons by position!
