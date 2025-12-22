// ============================================================================
// Example 03: Vim-Style Navigation with Custom Modifier Layer
// ============================================================================
//
// PURPOSE:
//   Demonstrates conditional mappings using custom modifier layers.
//   Implements Vim-style HJKL navigation that works system-wide.
//
// WHAT THIS DOES:
//   - CapsLock becomes a custom modifier (MD_00)
//   - When CapsLock is held down + HJKL, you get arrow keys
//   - H → Left, J → Down, K → Up, L → Right
//   - Works in any application (browser, editor, terminal, etc.)
//
// EXPECTED BEHAVIOR:
//   - Tap CapsLock alone: nothing happens (it's a modifier)
//   - Hold CapsLock + H: moves cursor left
//   - Hold CapsLock + J: moves cursor down
//   - Hold CapsLock + K: moves cursor up
//   - Hold CapsLock + L: moves cursor right
//
// COMPILATION:
//   cargo run --bin keyrx_compiler -- compile examples/03-vim-navigation.rhai
//
// WHY THIS IS USEFUL:
//   - Keep hands on home row while navigating
//   - Works in applications that don't support Vim keybindings
//   - More ergonomic than reaching for arrow keys
//
// ============================================================================

device_start("*");

  // STEP 1: Make CapsLock a custom modifier (MD_00)
  // When you press CapsLock, it activates modifier 00 but doesn't output anything
  map("VK_CapsLock", "MD_00");

  // STEP 2: When modifier MD_00 is active, remap HJKL to arrow keys
  // The when() function creates conditional mappings that only work when the modifier is pressed
  when_start("MD_00");
    map("VK_H", "VK_Left");   // H → Left arrow
    map("VK_J", "VK_Down");   // J → Down arrow
    map("VK_K", "VK_Up");     // K → Up arrow
    map("VK_L", "VK_Right");  // L → Right arrow
  when_end();

  // OPTIONAL: Add more navigation keys
  when_start("MD_00");
    map("VK_U", "VK_PageUp");    // U → Page Up
    map("VK_D", "VK_PageDown");  // D → Page Down
    map("VK_I", "VK_Home");      // I → Home
    map("VK_A", "VK_End");       // A → End
  when_end();

device_end();

// ============================================================================
// NOTES:
// - MD_00 through MD_FE are custom modifiers (255 total)
// - Custom modifiers don't send any key output themselves
// - when() creates mappings that only activate when the modifier is held
// - You can stack multiple when() blocks for the same modifier
// - HJKL keys work normally when CapsLock is not pressed
//
// ADVANCED:
// - You can use multiple modifiers: when(["MD_00", "MD_01"], || { ... })
// - You can negate conditions: when_not("MD_00", || { ... })
// - See DSL_MANUAL.md for more conditional mapping patterns
//
// VARIATIONS:
// - Use RightAlt instead of CapsLock as the modifier
// - Add text editing keys (W/B for word forward/back)
// - Add selection keys (hold Shift+CapsLock+HJKL for selection)
//
// SEE ALSO:
// - Example 04 for dual-function keys (tap/hold different actions)
// - Example 06 for complex multi-layer setups
// ============================================================================
