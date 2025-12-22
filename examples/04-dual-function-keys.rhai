// ============================================================================
// Example 04: Dual-Function Keys (Tap vs Hold)
// ============================================================================
//
// PURPOSE:
//   Demonstrates tap-hold functionality where a key has different behavior
//   depending on whether you tap it quickly or hold it down.
//
// WHAT THIS DOES:
//   - CapsLock: tap=Escape, hold=Ctrl
//   - Space: tap=Space, hold=custom modifier (MD_00) for navigation layer
//   - Enter: tap=Enter, hold=Shift
//
// EXPECTED BEHAVIOR:
//   - Tap CapsLock quickly (< 200ms): sends Escape
//   - Hold CapsLock down (>= 200ms): acts as Ctrl modifier
//   - Tap Space quickly: sends Space
//   - Hold Space down: activates navigation layer (no output)
//   - Tap Enter quickly: sends Enter
//   - Hold Enter down: acts as Shift modifier
//
// COMPILATION:
//   cargo run --bin keyrx_compiler -- compile examples/04-dual-function-keys.rhai
//
// WHY THIS IS USEFUL:
//   - Maximizes keyboard efficiency by giving keys dual purposes
//   - Reduces need to reach for modifier keys
//   - Popular in ergonomic keyboard communities
//
// ============================================================================

device_start("*");

  // DUAL-FUNCTION KEY 1: CapsLock
  // Tap: Escape (for Vim users)
  // Hold: Ctrl (for shortcuts like Ctrl+C, Ctrl+V)
  // Threshold: 200ms (if released before 200ms, it's a tap)
  tap_hold("VK_CapsLock", "VK_Escape", "MD_01", 200);

  // Make MD_01 act like physical Ctrl when held
  // When MD_01 is active, we want Ctrl+key combinations
  when_start("MD_01");
    // Common Ctrl shortcuts - output with physical Ctrl modifier
    map("VK_C", with_ctrl("VK_C"));  // Ctrl+C (copy)
    map("VK_V", with_ctrl("VK_V"));  // Ctrl+V (paste)
    map("VK_X", with_ctrl("VK_X"));  // Ctrl+X (cut)
    map("VK_Z", with_ctrl("VK_Z"));  // Ctrl+Z (undo)
    map("VK_F", with_ctrl("VK_F"));  // Ctrl+F (find)
    map("VK_S", with_ctrl("VK_S"));  // Ctrl+S (save)
    map("VK_A", with_ctrl("VK_A"));  // Ctrl+A (select all)
    map("VK_T", with_ctrl("VK_T"));  // Ctrl+T (new tab)
    map("VK_W", with_ctrl("VK_W"));  // Ctrl+W (close tab)
  when_end();

  // DUAL-FUNCTION KEY 2: Space
  // Tap: Space (normal typing)
  // Hold: Navigation layer (MD_00 from example 03)
  // Threshold: 200ms
  tap_hold("VK_Space", "VK_Space", "MD_00", 200);

  // When holding Space (MD_00), activate Vim navigation
  when_start("MD_00");
    map("VK_H", "VK_Left");
    map("VK_J", "VK_Down");
    map("VK_K", "VK_Up");
    map("VK_L", "VK_Right");
  when_end();

  // DUAL-FUNCTION KEY 3: Enter
  // Tap: Enter (normal line break)
  // Hold: Shift (for capital letters and symbols)
  // Threshold: 150ms (slightly faster for typing flow)
  tap_hold("VK_Enter", "VK_Enter", "MD_02", 150);

  // Make MD_02 act like physical Shift
  when_start("MD_02");
    // Letters become uppercase
    map("VK_A", with_shift("VK_A"));
    map("VK_B", with_shift("VK_B"));
    map("VK_C", with_shift("VK_C"));
    // ... (add more as needed)

    // Numbers become symbols
    map("VK_Num1", with_shift("VK_Num1"));  // 1 → !
    map("VK_Num2", with_shift("VK_Num2"));  // 2 → @
    // ... (add more as needed)
  when_end();

device_end();

// ============================================================================
// NOTES:
// - tap_hold() format: tap_hold(key, tap_action, hold_action, threshold_ms)
// - tap_action must be a VK_ key (what to send on quick tap)
// - hold_action must be a MD_ modifier (what modifier to activate on hold)
// - threshold_ms determines tap vs hold (typical values: 150-250ms)
// - If key is released before threshold: tap action is sent
// - If key is held beyond threshold: modifier is activated
//
// THRESHOLD TUNING:
// - Too low (< 100ms): hard to trigger taps, accidental holds
// - Too high (> 300ms): slow typing, waiting for tap to register
// - Recommended: 150-200ms for most users
// - Adjust based on your typing speed and preference
//
// LIMITATIONS:
// - Tap action cannot be a modifier (must be VK_ key)
// - Hold action cannot be a key (must be MD_ modifier)
// - You need to map the modifier to actual keys with when()
//
// ADVANCED:
// - Combine tap-hold with conditionals for complex behaviors
// - Use different modifiers for different hold keys
// - Stack modifiers: hold Space+Enter together
//
// SEE ALSO:
// - Example 03 for custom modifier layers
// - Example 06 for complex multi-layer setups
// - DSL_MANUAL.md for complete tap_hold() reference
// ============================================================================
