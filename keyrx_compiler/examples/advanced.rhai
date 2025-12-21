// Advanced KeyRx Configuration Example
//
// This example demonstrates all KeyRx DSL features including:
// - Custom modifiers (MD_XX): Virtual modifier states for layer switching
// - Custom locks (LK_XX): Toggle states
// - Tap/hold behavior: Different actions for tap vs hold
// - Conditional mappings: when() and when_not()
// - Physical modifier outputs: with_shift(), with_ctrl(), with_alt()
// - Nested conditions: Modifier cascades
//
// Prefix system:
//   VK_ = Virtual Key (standard key output)
//   MD_ = Custom Modifier (custom modifier state, MD_00 through MD_FE)
//   LK_ = Custom Lock (toggle state, LK_00 through LK_FE)

device_start("*");

    // === Custom Modifiers for Layer Switching ===

    // CapsLock becomes a navigation layer modifier (MD_00)
    // When held, it activates MD_00 which enables navigation mappings
    map("CapsLock", "MD_00");

    // Space key: tap for space, hold for custom modifier MD_01
    // threshold_ms is the time in milliseconds to distinguish tap from hold
    tap_hold("Space", "VK_Space", "MD_01", 200);

    // === Custom Locks (Toggle States) ===

    // ScrollLock toggles custom lock LK_00 (game mode)
    map("ScrollLock", "LK_00");

    // === Conditional Mappings with Custom Modifiers ===

    // When CapsLock (MD_00) is held, enable Vim-style navigation
    when("MD_00", [
        // Home row navigation
        map("H", "VK_Left"),
        map("J", "VK_Down"),
        map("K", "VK_Up"),
        map("L", "VK_Right"),

        // Word navigation with physical modifiers
        // Ctrl+Left/Right for word jumping
        with_ctrl("B", "VK_Left"),   // Back one word
        with_ctrl("W", "VK_Right"),  // Forward one word

        // Line navigation
        map("0", "VK_Home"),
        map("4", "VK_End"),  // $ in Vim

        // Page navigation
        map("U", "VK_PageUp"),
        map("D", "VK_PageDown"),

        // Delete operations
        map("X", "VK_Delete"),
        with_ctrl("Backspace", "VK_Backspace"),  // Delete word
    ]);

    // When Space (MD_01) is held, enable number pad on right hand
    when("MD_01", [
        map("U", "VK_7"),
        map("I", "VK_8"),
        map("O", "VK_9"),
        map("J", "VK_4"),
        map("K", "VK_5"),
        map("L", "VK_6"),
        map("M", "VK_1"),
        map("Comma", "VK_2"),
        map("Period", "VK_3"),
        map("N", "VK_0"),
    ]);

    // === Multiple Conditions (AND logic) ===

    // When both MD_00 and MD_01 are active, enable special function keys
    when(["MD_00", "MD_01"], [
        map("1", "VK_F1"),
        map("2", "VK_F2"),
        map("3", "VK_F3"),
        map("4", "VK_F4"),
    ]);

    // === Negated Conditions (when_not) ===

    // When custom lock LK_00 is NOT active, allow normal Escape
    when_not("LK_00", [
        map("Escape", "VK_Escape"),
    ]);

    // When game mode (LK_00) IS active, remap keys for gaming
    when("LK_00", [
        // Disable CapsLock layer in game mode
        map("CapsLock", "VK_LCtrl"),

        // WASD movement (identity mapping for clarity)
        map("W", "VK_W"),
        map("A", "VK_A"),
        map("S", "VK_S"),
        map("D", "VK_D"),

        // Space for jump
        map("Space", "VK_Space"),

        // Numbers for weapon switching
        map("1", "VK_1"),
        map("2", "VK_2"),
        map("3", "VK_3"),
    ]);

    // === Physical Modifier Combinations ===

    // with_shift(): Output key with Shift pressed
    with_shift("F1", "VK_F1");

    // with_ctrl(): Output key with Ctrl pressed
    with_ctrl("F2", "VK_F2");

    // with_alt(): Output key with Alt pressed
    with_alt("F3", "VK_F3");

    // with_mods(): Combine multiple physical modifiers
    // Parameters: from, to, shift, ctrl, alt, win (all boolean)
    with_mods("F4", "VK_F4", true, true, false, false);

    // === Nested Modifier Cascades ===

    // Create a second-level navigation layer
    // When MD_00 is active, Tab becomes MD_02
    when("MD_00", [
        map("Tab", "MD_02"),
    ]);

    // When both MD_00 and MD_02 are active, enable window management
    when(["MD_00", "MD_02"], [
        // Window tiling shortcuts with Win key
        with_mods("H", "VK_Left", false, false, false, true),
        with_mods("L", "VK_Right", false, false, false, true),
        with_mods("K", "VK_Up", false, false, false, true),
        with_mods("J", "VK_Down", false, false, false, true),
    ]);

device_end();
