// ============================================================================
// Example 07: Home-Row Mods and Caps = Esc / Ctrl
// ============================================================================
//
// PURPOSE:
//   Tap a home-row key to type its letter; hold it to get a REAL modifier
//   (Shift / Ctrl / Alt / Meta). Because the hold presses the actual
//   modifier key, it works with every application and with the mouse.
//
// HOW IT WORKS:
//   tap_hold(key, tap, hold, threshold_ms) with hold = "VK_LCtrl" etc.
//   - tap (released inside the threshold, nothing typed inside it): the tap key
//   - hold (past the threshold, or another key is pressed AND released while
//     it is down): the modifier is pressed until the key is released
//   - rolling over from one key to the next is still typed, not held
//
// TUNING:
//   - 200ms is the usual start; raise it (250-300) if fast typing triggers
//     modifiers, lower it if holds feel slow.
//   - Use a longer threshold on the weaker fingers (pinky / ring).
//
// ============================================================================

// A plain helper: the script language lets you define functions, so one line
// per home-row key stays readable.
fn home_row(key, modifier, threshold_ms) {
  tap_hold(key, "VK_" + key, modifier, threshold_ms);
}

device_start("*");

  // Caps Lock: Escape on tap, Ctrl on hold
  tap_hold("CapsLock", "VK_Escape", "VK_LCtrl", 200);

  // Left hand: A S D F  =  Meta Alt Ctrl Shift
  home_row("A", "VK_LMeta", 250);
  home_row("S", "VK_LAlt", 225);
  home_row("D", "VK_LCtrl", 200);
  home_row("F", "VK_LShift", 200);

  // Right hand: J K L ;  =  Shift Ctrl Alt Meta
  home_row("J", "VK_RShift", 200);
  home_row("K", "VK_RCtrl", 200);
  home_row("L", "VK_RAlt", 225);
  tap_hold("Semicolon", "VK_Semicolon", "VK_RMeta", 250);

device_end();

// ============================================================================
// NOTES:
// - hold_only(key, "VK_LShift", 150) is the same without a tap output.
// - Mixing: hold may also be "MD_xx" to switch a layer instead (example 04).
// ============================================================================
