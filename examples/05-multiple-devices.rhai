// ============================================================================
// Example 05: Multiple Device Configurations
// ============================================================================
//
// PURPOSE:
//   Demonstrates how to configure different mappings for different keyboards.
//   Useful when you have multiple keyboards (laptop + external) and want
//   different layouts for each.
//
// WHAT THIS DOES:
//   - Laptop keyboard: minimal remapping (just CapsLock→Escape)
//   - External mechanical keyboard: full Vim layer + dual-function keys
//   - Fallback configuration: applies to any unmatched devices
//
// EXPECTED BEHAVIOR:
//   - On laptop keyboard: CapsLock becomes Escape
//   - On external keyboard: full custom layout with layers
//   - On other keyboards: basic navigation improvements
//
// COMPILATION:
//   cargo run --bin keyrx_compiler -- compile examples/05-multiple-devices.rhai
//
// WHY THIS IS USEFUL:
//   - Laptop keyboards have different key placement than external boards
//   - You might want conservative changes on laptop, aggressive on external
//   - Share one config file across multiple computers
//
// ============================================================================

// DEVICE 1: Laptop Keyboard
// Pattern matches typical laptop keyboard names
// (adjust pattern to match your actual device name)
device_start("*Laptop*Keyboard*");

  // Minimal changes for laptop - just the essentials
  // CapsLock → Escape (universal improvement)
  map("CapsLock", "VK_Escape");

  // Right Alt → Backspace (laptop backspace is far)
  map("RAlt", "VK_Backspace");

device_end();


// DEVICE 2: External Mechanical Keyboard
// Pattern matches your specific mechanical keyboard
// Examples: "*Keychron*", "*HHKB*", "*Ergodox*", "*Kinesis*"
device_start("*Keychron*");

  // LAYER 1: CapsLock as dual-function (tap=Esc, hold=Vim layer)
  tap_hold("CapsLock", "VK_Escape", "MD_00", 200);

  // LAYER 2: Space as dual-function (tap=Space, hold=symbol layer)
  tap_hold("Space", "VK_Space", "MD_01", 200);

  // LAYER 3: Enter as dual-function (tap=Enter, hold=number layer)
  tap_hold("Enter", "VK_Enter", "MD_02", 200);

  // Vim navigation layer (MD_00 = hold CapsLock)
  when_start("MD_00");
    // HJKL navigation
    map("H", "VK_Left");
    map("J", "VK_Down");
    map("K", "VK_Up");
    map("L", "VK_Right");

    // Extended navigation
    map("U", "VK_PageUp");
    map("D", "VK_PageDown");
    map("I", "VK_Home");
    map("A", "VK_End");

    // Word movement (Ctrl+Arrow)
    map("W", with_ctrl("VK_Right"));   // Next word
    map("B", with_ctrl("VK_Left"));    // Previous word
  when_end();

  // Symbol layer (MD_01 = hold Space)
  when_start("MD_01");
    // Common programming symbols on home row
    map("J", "VK_LeftBracket");   // [
    map("K", "VK_RightBracket");  // ]
    map("H", with_shift("VK_Num9")); // (
    map("L", with_shift("VK_Num0")); // )
    map("U", with_shift("VK_LeftBracket"));  // {
    map("I", with_shift("VK_RightBracket")); // }
  when_end();

  // Number layer (MD_02 = hold Enter)
  when_start("MD_02");
    // Numbers on home row (like a numpad)
    map("J", "VK_Num1");
    map("K", "VK_Num2");
    map("L", "VK_Num3");
    map("U", "VK_Num4");
    map("I", "VK_Num5");
    map("O", "VK_Num6");
    map("M", "VK_Num7");
    map("Comma", "VK_Num8");
    map("Period", "VK_Num9");
    map("N", "VK_Num0");
  when_end();

device_end();


// DEVICE 3: Fallback for all other devices
// The "*" pattern matches any device not matched by previous patterns
device_start("*");

  // Conservative changes that work well on any keyboard

  // CapsLock → Escape (universal improvement)
  map("CapsLock", "VK_Escape");

  // Right Alt as navigation modifier
  map("RAlt", "MD_10");

  when_start("MD_10");
    // Arrow keys on IJKL (more standard than HJKL)
    map("I", "VK_Up");
    map("J", "VK_Left");
    map("K", "VK_Down");
    map("L", "VK_Right");
  when_end();

device_end();

// ============================================================================
// NOTES:
// - Device patterns are matched in order (first match wins)
// - Pattern syntax: wildcards (*) match any characters
// - To see device names: check daemon logs or system device manager
// - Linux: /proc/bus/input/devices or evtest
// - Windows: Device Manager → Keyboards
//
// PATTERN MATCHING EXAMPLES:
// - "*Keychron*" matches "Keychron K2 Keyboard"
// - "*Laptop*" matches "Dell Laptop Keyboard"
// - "Vendor_04d9_Product_a09f" matches exact vendor/product ID
// - "*" matches everything (use as last fallback)
//
// WORKFLOW:
// 1. Run daemon in debug mode to see device names
// 2. Create device blocks with specific patterns
// 3. Add fallback device("*") for unmatched devices
// 4. Test on each keyboard to verify correct config loads
//
// TIPS:
// - Keep laptop configs minimal (you might want vanilla behavior sometimes)
// - Go wild on external keyboards (they're dedicated to one computer)
// - Use descriptive comments to remember which device is which
// - Test each device separately before combining
//
// SEE ALSO:
// - DSL_MANUAL.md for device pattern syntax
// - Example 03 for custom modifier layers
// - Example 04 for dual-function keys
// - Example 06 for advanced multi-layer setup
// ============================================================================
