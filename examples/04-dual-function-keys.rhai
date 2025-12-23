// ============================================================================
// Example 04: Dual-Function Keys (Tap vs Hold)
// ============================================================================
//
// PURPOSE:
//   Demonstrates tap-hold functionality where a key has different behavior
//   depending on whether you tap it quickly or hold it down.
//
// WHAT THIS DOES:
//   - CapsLock: tap=Escape, hold=Ctrl layer (MD_01)
//   - Space: tap=Space, hold=navigation layer (MD_00)
//   - Enter: tap=Enter, hold=Shift layer (MD_02)
//
// COMPILATION:
//   cargo run --bin keyrx_compiler -- compile examples/04-dual-function-keys.rhai
//
// ============================================================================
// TAP-HOLD BEHAVIOR EXPLAINED
// ============================================================================
//
// The tap_hold() function creates dual-function keys with timing-based behavior:
//
//   tap_hold(physical_key, tap_output, hold_modifier, threshold_ms)
//
// State Machine:
//   1. IDLE: Key not pressed
//   2. PENDING: Key pressed, waiting to determine tap or hold
//   3. HOLD: Threshold exceeded OR another key pressed (Permissive Hold)
//
// Decision Flow:
//   - Press key → Enter PENDING state, start timer
//   - Release before threshold → TAP: Output tap_output key
//   - Hold past threshold → HOLD: Activate hold_modifier
//   - Another key pressed while PENDING → HOLD: Immediately activate modifier
//     (This is "Permissive Hold" - enables natural typing flow)
//
// Timing:
//   - threshold_ms: Time in milliseconds to distinguish tap from hold
//   - Default: 200ms (if not specified)
//   - Recommended range: 150-250ms depending on typing speed
//   - Too low (< 100ms): Hard to trigger taps, accidental holds
//   - Too high (> 300ms): Typing feels sluggish, delays on taps
//
// ============================================================================

device_start("*");

// ============================================================================
// DUAL-FUNCTION KEY 1: CapsLock
// ============================================================================
// Behavior:
//   - Tap (< 200ms): Outputs Escape key (perfect for Vim users)
//   - Hold (>= 200ms): Activates MD_01, making Ctrl shortcuts available
//
// Why this works:
//   - CapsLock is rarely used as CapsLock
//   - Easy to reach with pinky finger
//   - Natural position for a modifier key
//
tap_hold("VK_CapsLock", "VK_Escape", "MD_01", 200);

// When MD_01 (CapsLock held) is active, enable common Ctrl shortcuts
// These mappings only activate when CapsLock is held past threshold
// OR when you press another key while CapsLock is in PENDING state
when_start("MD_01");
  // Common Ctrl shortcuts - output with physical Ctrl modifier
  // Example: Hold CapsLock + press C → Ctrl+C (copy)
  map("VK_C", with_ctrl("VK_C"));  // Ctrl+C (copy)
  map("VK_V", with_ctrl("VK_V"));  // Ctrl+V (paste)
  map("VK_X", with_ctrl("VK_X"));  // Ctrl+X (cut)
  map("VK_Z", with_ctrl("VK_Z"));  // Ctrl+Z (undo)
  map("VK_Y", with_ctrl("VK_Y"));  // Ctrl+Y (redo)
  map("VK_F", with_ctrl("VK_F"));  // Ctrl+F (find)
  map("VK_S", with_ctrl("VK_S"));  // Ctrl+S (save)
  map("VK_A", with_ctrl("VK_A"));  // Ctrl+A (select all)
  map("VK_T", with_ctrl("VK_T"));  // Ctrl+T (new tab)
  map("VK_W", with_ctrl("VK_W"));  // Ctrl+W (close tab)
  map("VK_N", with_ctrl("VK_N"));  // Ctrl+N (new)
  map("VK_P", with_ctrl("VK_P"));  // Ctrl+P (print)
when_end();


// ============================================================================
// DUAL-FUNCTION KEY 2: Space (Navigation Layer)
// ============================================================================
// Behavior:
//   - Tap (< 200ms): Outputs Space (normal typing)
//   - Hold (>= 200ms): Activates MD_00 for Vim-style navigation
//
// Permissive Hold example:
//   - Press Space, then quickly press H (before 200ms)
//   - MD_00 activates immediately, H becomes Left arrow
//   - This enables fast "Space+H" navigation without waiting
//
tap_hold("VK_Space", "VK_Space", "MD_00", 200);

// When holding Space (MD_00), activate Vim-style navigation
// The modifier is activated either by:
//   1. Holding Space for 200ms, OR
//   2. Pressing another key while Space is held (Permissive Hold)
when_start("MD_00");
  // Arrow keys on HJKL (Vim-style)
  map("VK_H", "VK_Left");   // H → Left
  map("VK_J", "VK_Down");   // J → Down
  map("VK_K", "VK_Up");     // K → Up
  map("VK_L", "VK_Right");  // L → Right

  // Page navigation
  map("VK_U", "VK_PageUp");    // U → Page Up
  map("VK_D", "VK_PageDown");  // D → Page Down

  // Line navigation
  map("VK_I", "VK_Home");      // I → Home (start of line)
  map("VK_A", "VK_End");       // A → End (end of line)

  // Word navigation (Ctrl+Arrow)
  map("VK_W", with_ctrl("VK_Right"));  // W → word forward
  map("VK_B", with_ctrl("VK_Left"));   // B → back word
when_end();


// ============================================================================
// DUAL-FUNCTION KEY 3: Enter (Shift Layer)
// ============================================================================
// Behavior:
//   - Tap (< 150ms): Outputs Enter (normal line break)
//   - Hold (>= 150ms): Activates MD_02 for Shift behavior
//
// Note: Using 150ms (faster threshold) because Enter is frequently tapped
// during normal typing. Adjust based on your typing speed.
//
tap_hold("VK_Enter", "VK_Enter", "MD_02", 150);

// When MD_02 (Enter held) is active, keys output with Shift
when_start("MD_02");
  // Letters become uppercase
  map("VK_A", with_shift("VK_A"));
  map("VK_B", with_shift("VK_B"));
  map("VK_C", with_shift("VK_C"));
  map("VK_D", with_shift("VK_D"));
  map("VK_E", with_shift("VK_E"));
  // ... add more letters as needed

  // Numbers become symbols
  map("VK_Num1", with_shift("VK_Num1"));  // 1 → !
  map("VK_Num2", with_shift("VK_Num2"));  // 2 → @
  map("VK_Num3", with_shift("VK_Num3"));  // 3 → #
  map("VK_Num4", with_shift("VK_Num4"));  // 4 → $
  map("VK_Num5", with_shift("VK_Num5"));  // 5 → %
  // ... add more as needed
when_end();


// ============================================================================
// EDGE CASE 1: Multiple Concurrent Tap-Holds
// ============================================================================
// You can hold multiple tap-hold keys simultaneously.
// Each is tracked independently.
//
// Example scenario: Hold Space (nav) + press H (left arrow)
//                   Then hold CapsLock (ctrl) + press S (Ctrl+S)
//
// Both MD_00 and MD_01 can be active at the same time.
// The system tracks up to 32 concurrent tap-hold keys.


// ============================================================================
// EDGE CASE 2: Combined Modifier Layers
// ============================================================================
// When both Space (MD_00) and CapsLock (MD_01) are held together,
// you can define special combined behaviors.
//
when_start(["MD_00", "MD_01"]);
  // Text manipulation shortcuts
  // Hold Space + CapsLock + W → Select word right (Ctrl+Shift+Right)
  map("VK_W", with_mods("VK_Right", true, true, false, false));
  // Hold Space + CapsLock + B → Select word left (Ctrl+Shift+Left)
  map("VK_B", with_mods("VK_Left", true, true, false, false));
when_end();


// ============================================================================
// EDGE CASE 3: Tap-Hold Inside Conditional
// ============================================================================
// You can define tap-hold keys that only work when another modifier is active.
// Example: Tab becomes a tap-hold for Function keys only when in nav layer
//
when_start("MD_00");
  // When Space is held, Tab becomes: tap=Tab, hold=MD_03 (Function layer)
  tap_hold("VK_Tab", "VK_Tab", "MD_03", 200);
when_end();

// MD_03 provides Function keys (only accessible when Space is held first)
when_start("MD_03");
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
when_end();


device_end();

// ============================================================================
// QUICK REFERENCE
// ============================================================================
//
// tap_hold() Syntax:
//   tap_hold(key, tap_action, hold_action, threshold_ms)
//
// Parameters:
//   - key: Physical key to modify (with VK_ prefix)
//   - tap_action: VK_ key to output on quick release
//   - hold_action: MD_ modifier to activate when held
//   - threshold_ms: Time in ms to distinguish tap from hold (default: 200)
//
// State Transitions:
//   IDLE → PENDING (on key press)
//   PENDING → TAP (release before threshold) → output tap key
//   PENDING → HOLD (threshold exceeded OR another key pressed)
//   HOLD → IDLE (on key release) → deactivate modifier
//
// ============================================================================
// THRESHOLD TUNING GUIDE
// ============================================================================
//
// Finding Your Optimal Threshold:
//
//   100ms - Very fast typists only. Risk: accidental holds
//   150ms - Fast typists. Good for frequently-tapped keys (Enter)
//   200ms - Default. Balanced for most users
//   250ms - Slower typists. Safer tap detection
//   300ms - Very deliberate holds. Risk: sluggish feel
//
// Tips:
//   1. Start with 200ms, adjust by 25ms increments
//   2. Frequently-tapped keys (Space, Enter) may need lower thresholds
//   3. Rarely-tapped keys (CapsLock) can use higher thresholds
//   4. If you miss taps often, increase threshold
//   5. If holds feel slow, decrease threshold
//
// ============================================================================
// PERMISSIVE HOLD EXPLAINED
// ============================================================================
//
// When you press a tap-hold key and then another key before the threshold:
//
//   1. Press CapsLock (enters PENDING state)
//   2. Press C (before 200ms threshold)
//   3. System immediately:
//      a. Transitions CapsLock to HOLD state
//      b. Activates MD_01 modifier
//      c. Processes C with MD_01 active → outputs Ctrl+C
//
// This enables fast modifier combos without waiting for the threshold.
// Called "Permissive Hold" because it permits the hold interpretation
// when interrupted by another keypress.
//
// Without Permissive Hold, you'd have to:
//   1. Press CapsLock
//   2. Wait 200ms
//   3. Then press C
//
// Permissive Hold makes typing feel natural and responsive.
//
// ============================================================================
// LIMITATIONS
// ============================================================================
//
// - tap_action must be a VK_ key (cannot be a modifier)
// - hold_action must be a MD_ modifier (cannot be a key or LK_ lock)
// - You need to map the modifier to actual keys with when_start()
// - Maximum 32 concurrent tap-hold keys tracked
// - Minimum threshold: 50ms (rejected if lower)
// - Maximum threshold: 1000ms (warning if higher)
//
// ============================================================================
// SEE ALSO
// ============================================================================
//
// - Example 03: Vim-style navigation (simpler modifier layer)
// - Example 06: Advanced multi-layer setup with tap-hold
// - docs/DSL_MANUAL.md: Complete tap_hold() reference
//
// ============================================================================
