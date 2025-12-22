// ============================================================================
// Example 06: Advanced Multi-Layer Setup
// ============================================================================
//
// PURPOSE:
//   Demonstrates a complex multi-layer keyboard configuration with:
//   - Multiple custom modifier layers
//   - Conditional mappings (when/when_not)
//   - Layer stacking (multiple modifiers active simultaneously)
//   - Lock states (toggle layers on/off)
//
// WHAT THIS DOES:
//   - Base layer: normal typing + some improvements
//   - Nav layer (MD_00): Vim-style navigation and editing
//   - Symbol layer (MD_01): Programming symbols and brackets
//   - Number layer (MD_02): Numpad-style number entry
//   - Function layer (MD_03): F-keys and media controls
//   - Gaming mode (LK_00): WASD optimizations, toggle with ScrollLock
//
// EXPECTED BEHAVIOR:
//   - Hold CapsLock: activate Nav layer (HJKL → arrows)
//   - Hold Space: activate Symbol layer (easy access to (), [], {}, etc.)
//   - Hold Enter: activate Number layer (home row becomes 1-9-0)
//   - Hold Tab: activate Function layer (F1-F12, media keys)
//   - Press ScrollLock: toggle Gaming mode on/off
//
// COMPILATION:
//   cargo run --bin keyrx_compiler -- compile examples/06-advanced-layers.rhai
//
// WHY THIS IS USEFUL:
//   - Minimize hand movement (everything accessible from home row)
//   - Optimize for programming (symbols easily accessible)
//   - Support different workflows (coding vs gaming)
//   - Demonstrates the full power of keyrx DSL
//
// ============================================================================

device_start("*");

// ============================================================================
// BASE LAYER: Always-active improvements
// ============================================================================

// CapsLock as dual-function: tap=Escape, hold=Nav layer
tap_hold("VK_CapsLock", "VK_Escape", "MD_00", 200);

// Space as dual-function: tap=Space, hold=Symbol layer
tap_hold("VK_Space", "VK_Space", "MD_01", 200);

// Enter as dual-function: tap=Enter, hold=Number layer
tap_hold("VK_Enter", "VK_Enter", "MD_02", 150);

// Tab as dual-function: tap=Tab, hold=Function layer
tap_hold("VK_Tab", "VK_Tab", "MD_03", 200);

// ScrollLock toggles Gaming mode (lock LK_00)
map("VK_ScrollLock", "LK_00");


// ============================================================================
// NAV LAYER (MD_00): Navigation and text editing
// ============================================================================

when_start("MD_00");
  // === Arrow keys on HJKL (Vim-style) ===
  map("VK_H", "VK_Left");
  map("VK_J", "VK_Down");
  map("VK_K", "VK_Up");
  map("VK_L", "VK_Right");

  // === Page navigation ===
  map("VK_U", "VK_PageUp");      // U = up (page)
  map("VK_D", "VK_PageDown");    // D = down (page)

  // === Line navigation ===
  map("VK_I", "VK_Home");        // I = inside (line start)
  map("VK_A", "VK_End");         // A = append (line end)

  // === Word navigation (Ctrl+Arrow) ===
  map("VK_W", with_ctrl("VK_Right"));   // W = word forward
  map("VK_B", with_ctrl("VK_Left"));    // B = back word

  // === Deletion ===
  map("VK_X", "VK_Delete");      // X = delete char
  map("VK_Backspace", with_ctrl("VK_Backspace"));  // Delete word

  // === Selection (Shift+Arrow) ===
  map("VK_Semicolon", with_shift("VK_Left"));  // Select left
  map("VK_Quote", with_shift("VK_Right")); // Select right
when_end();


// ============================================================================
// SYMBOL LAYER (MD_01): Programming symbols
// ============================================================================

when_start("MD_01");
  // === Brackets on home row ===
  map("VK_H", with_shift("VK_Num9"));           // H = (
  map("VK_J", "VK_LeftBracket");             // J = [
  map("VK_K", with_shift("VK_LeftBracket")); // K = {
  map("VK_L", with_shift("VK_RightBracket"));// L = }
  map("VK_Semicolon", "VK_RightBracket");    // ; = ]
  map("VK_Quote", with_shift("VK_Num0"));  // ' = )

  // === Common operators ===
  map("VK_Y", with_shift("VK_Equal"));   // Y = +
  map("VK_U", "VK_Minus");                // U = -
  map("VK_I", with_shift("VK_Num8"));        // I = *
  map("VK_O", "VK_Slash");                // O = /
  map("VK_P", "VK_Equal");                // P = =

  // === Other symbols ===
  map("VK_N", with_shift("VK_Num1"));        // N = !
  map("VK_M", with_shift("VK_Num2"));        // M = @
  map("VK_Comma", with_shift("VK_Num3"));    // , = #
  map("VK_Period", with_shift("VK_Num4"));   // . = $
  map("VK_Slash", with_shift("VK_Num5"));    // / = %
when_end();


// ============================================================================
// NUMBER LAYER (MD_02): Numpad on home row
// ============================================================================

when_start("MD_02");
  // === Numbers 1-9-0 ===
  map("VK_M", "VK_Num1");
  map("VK_Comma", "VK_Num2");
  map("VK_Period", "VK_Num3");
  map("VK_J", "VK_Num4");
  map("VK_K", "VK_Num5");
  map("VK_L", "VK_Num6");
  map("VK_U", "VK_Num7");
  map("VK_I", "VK_Num8");
  map("VK_O", "VK_Num9");
  map("VK_N", "VK_Num0");

  // === Operators ===
  map("VK_H", "VK_Minus");          // H = -
  map("VK_Semicolon", with_shift("VK_Equal")); // ; = +
  map("VK_Quote", "VK_Equal");            // ' = =
  map("VK_Slash", "VK_Slash");                 // / = /
  map("VK_P", with_shift("VK_Num8"));             // P = *
when_end();


// ============================================================================
// FUNCTION LAYER (MD_03): F-keys and media controls
// ============================================================================

when_start("MD_03");
  // === F-keys on number row ===
  map("VK_Num1", "VK_F1");
  map("VK_Num2", "VK_F2");
  map("VK_Num3", "VK_F3");
  map("VK_Num4", "VK_F4");
  map("VK_Num5", "VK_F5");
  map("VK_Num6", "VK_F6");
  map("VK_Num7", "VK_F7");
  map("VK_Num8", "VK_F8");
  map("VK_Num9", "VK_F9");
  map("VK_Num0", "VK_F10");
  map("VK_Minus", "VK_F11");
  map("VK_Equal", "VK_F12");

  // === Media controls on HJKL ===
  map("VK_H", "VK_MediaPrevious");  // H = previous track
  map("VK_J", "VK_VolumeDown");      // J = volume down
  map("VK_K", "VK_VolumeUp");        // K = volume up
  map("VK_L", "VK_MediaNext");  // L = next track
  map("VK_Space", "VK_MediaPlayPause"); // Space = play/pause
  map("VK_M", "VK_Mute");      // M = mute
when_end();


// ============================================================================
// GAMING MODE (LK_00): WASD optimizations
// ============================================================================
// When gaming mode is OFF (default), use normal mappings
// When gaming mode is ON (press ScrollLock), activate gaming optimizations

when_start("LK_00");
  // === Remove dual-function keys (instant response needed) ===
  // In gaming, we want instant keypresses, no tap-hold delay
  // These override the base layer tap_hold mappings

  // CapsLock = instant Ctrl (for crouch/sprint)
  map("VK_CapsLock", with_ctrl("VK_CapsLock"));

  // Space = instant Space (for jump)
  map("VK_Space", "VK_Space");

  // === Quick access macros ===
  // Q = quick save (F5 in many games)
  map("VK_Q", "VK_F5");

  // E = quick load (F9 in many games)
  map("VK_E", "VK_F9");

  // === Number keys as function keys (weapon switching) ===
  map("VK_Num1", "VK_F1");
  map("VK_Num2", "VK_F2");
  map("VK_Num3", "VK_F3");
  map("VK_Num4", "VK_F4");
when_end();


// ============================================================================
// LAYER STACKING: Multiple modifiers simultaneously
// ============================================================================

// Example: Hold CapsLock (Nav) + Space (Symbol) for advanced navigation

when_start(["MD_00", "MD_01"]);
  // Text manipulation shortcuts (Ctrl+Shift+Arrow for word selection)
  map("VK_W", with_mods("VK_Right", true, true, false, false));  // Select word right
  map("VK_B", with_mods("VK_Left", true, true, false, false));   // Select word left

  // Line manipulation (Ctrl+Shift+K = delete line in many editors)
  map("VK_X", with_mods("VK_K", false, true, false, false));
when_end();

device_end();

// ============================================================================
// NOTES:
//
// LAYER ACTIVATION:
// - MD_XX = Custom modifiers (hold key to activate layer)
// - LK_XX = Lock states (press once to toggle on/off)
// - Layers can be stacked (hold multiple modifiers together)
//
// DESIGN PRINCIPLES:
// - Keep frequently used keys on home row
// - Group related functions (navigation, symbols, numbers)
// - Use mnemonic mappings (H=left, L=right, U=up, D=down)
// - Avoid conflicts between layers
//
// CUSTOMIZATION TIPS:
// 1. Start with one layer, test it thoroughly
// 2. Add layers incrementally (don't overwhelm yourself)
// 3. Use muscle memory: consistent patterns across layers
// 4. Document your layout (comments help future you)
// 5. Adjust thresholds (tap_hold timing) to your typing speed
//
// DEBUGGING:
// - Use when_not("LK_00", ...) to disable features when gaming
// - Test each layer independently before combining
// - Keep a "reset" config handy in case you lock yourself out
// - Use daemon debug mode to see which modifiers are active
//
// PERFORMANCE:
// - Layers add zero latency (DFA is O(1) lookup)
// - Stacking 2-3 modifiers is fine
// - Avoid deeply nested conditionals if possible
//
// SEE ALSO:
// - Example 03: Simpler Vim navigation layer
// - Example 04: Dual-function key basics
// - Example 05: Multiple device configs
// - DSL_MANUAL.md: Complete function reference
// ============================================================================
