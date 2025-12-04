// Comprehensive row-column API demonstration
// Shows all 5 row-col functions: remap_rc, tap_hold_rc, block_rc, combo_rc, layer_map_rc
//
// This script demonstrates position-based key mapping that works across any keyboard layout.
// Physical positions are specified as (row, col) instead of key names.

fn on_init() {
    print("=== Row-Column API Full Demo ===");

    // 1. BASIC REMAP: CapsLock → Escape
    // r3_c0 is the CapsLock position on most keyboards
    remap_rc(3, 0, "Escape");
    print("✓ Remapped r3_c0 (CapsLock) → Escape");

    // 2. TAP-HOLD: Home row modifiers
    // r3_c1 = A position: tap=A, hold=LeftCtrl
    tap_hold_rc(3, 1, "A", "LeftCtrl");
    print("✓ Tap-hold r3_c1 (A): tap=A, hold=LeftCtrl");

    // r3_c2 = S position: tap=S, hold=LeftAlt
    tap_hold_rc(3, 2, "S", "LeftAlt");
    print("✓ Tap-hold r3_c2 (S): tap=S, hold=LeftAlt");

    // r3_c3 = D position: tap=D, hold=LeftShift
    tap_hold_rc(3, 3, "D", "LeftShift");
    print("✓ Tap-hold r3_c3 (D): tap=D, hold=LeftShift");

    // r3_c4 = F position: tap=F, hold=LeftSuper
    tap_hold_rc(3, 4, "F", "LeftSuper");
    print("✓ Tap-hold r3_c4 (F): tap=F, hold=LeftSuper");

    // 3. BLOCK: Disable Insert key
    // r1_c13 = Insert position on most keyboards
    block_rc(1, 13);
    print("✓ Blocked r1_c13 (Insert)");

    // 4. COMBO: Multi-key shortcuts
    // CapsLock + A → Escape (emergency exit)
    combo_rc([[3, 0], [3, 1]], "Escape");
    print("✓ Combo [r3_c0, r3_c1] (CapsLock+A) → Escape");

    // Q + W → Save (Ctrl+S)
    combo_rc([[2, 1], [2, 2]], "C-S");
    print("✓ Combo [r2_c1, r2_c2] (Q+W) → Ctrl+S");

    // 5. LAYERS: Define navigation layer
    layer_define(1);
    print("✓ Defined layer 1 (navigation)");

    // Map home row to arrow keys on layer 1
    layer_map_rc(1, 3, 1, "Left");   // A → Left
    layer_map_rc(1, 3, 2, "Down");   // S → Down
    layer_map_rc(1, 3, 3, "Up");     // D → Up
    layer_map_rc(1, 3, 4, "Right");  // F → Right
    print("✓ Layer 1: home row → arrow keys");

    // Activate layer 1 with Space key (hold)
    tap_hold(  // Using regular API since layer_toggle is a string, not position-based
        "Space",
        "Space",
        "layer_toggle('1')"
    );
    print("✓ Space: tap=Space, hold=toggle layer 1");

    print("\n🎉 Row-column API initialized successfully!");
    print("📍 All key mappings are position-based - works on any keyboard layout!");
}

fn on_key_event(key, pressed_at) {
    // Events are handled automatically by the registered mappings
}
