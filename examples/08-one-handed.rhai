// ============================================================================
// Example 08: One-Handed Typing (left hand) with sticky keys
// ============================================================================
//
// PURPOSE:
//   Type the whole keyboard with the LEFT hand only, and without having to
//   hold two keys at once.
//
// WHAT YOU GET:
//   1. Mirror layer: hold Space and the keys flip to the other half of the
//      keyboard (A types ;  S types L  F types J ...). Tap Space for a space.
//   2. Layer latch: tap CapsLock to stay on the mirror layer without holding
//      Space; tap it again to go back.
//   3. Sticky Shift / Ctrl: tap Tab (Shift) or Grave (Ctrl) and the modifier
//      applies to the NEXT key only. Hold them to use them like normal keys.
//
// TUNING (slow or unsteady taps):
//   - Raise the Space threshold (250 -> 350). A longer threshold only delays
//     a hold when you press Space ALONE; typing a mirrored key while Space is
//     down still switches layers at once.
//   - If a deliberate pause before a key must not turn into a hold, use
//     tap_hold_timeout_only (see the manual).
//   - Sticky keys wait for the next key. The 5000 below releases an unused
//     latch after 5 seconds; use 0 to wait forever.
//
// ============================================================================

// The mirrored right half of the keyboard, typed with the left hand.
// The same table is used by the held layer (MD_00) and the latched layer
// (LK_00), so they can never drift apart.
fn mirror() {
  // Top letter row: Q W E R T  ->  P O I U Y
  map("Q", "VK_P");
  map("W", "VK_O");
  map("E", "VK_I");
  map("R", "VK_U");
  map("T", "VK_Y");

  // Home row: A S D F G  ->  ; L K J H
  map("A", "VK_Semicolon");
  map("S", "VK_L");
  map("D", "VK_K");
  map("F", "VK_J");
  map("G", "VK_H");

  // Bottom row: Z X C V B  ->  / . , M N
  map("Z", "VK_Slash");
  map("X", "VK_Period");
  map("C", "VK_Comma");
  map("V", "VK_M");
  map("B", "VK_N");

  // Number row: 1 2 3 4 5  ->  0 9 8 7 6
  map("Num1", "VK_Num0");
  map("Num2", "VK_Num9");
  map("Num3", "VK_Num8");
  map("Num4", "VK_Num7");
  map("Num5", "VK_Num6");
}

device_start("*");

  // Space: a space on tap, the mirror layer (MD_00) while held.
  tap_hold("Space", "VK_Space", "MD_00", 250);

  // CapsLock latches the mirror layer on and off (custom lock LK_00).
  map("CapsLock", "LK_00");

  // Sticky modifiers: a tap applies to the next key only.
  one_shot("Tab", "VK_LShift", 5000);
  one_shot("Grave", "VK_LCtrl", 5000);

  when_start("MD_00");
    mirror();
  when_end();

  when_start("LK_00");
    mirror();
  when_end();

device_end();

// ============================================================================
// NOTES:
// - The right-hand keys are untouched, so a second person can still type.
// - Enter and Backspace are reachable with the left hand's thumb/pinky on most
//   boards; add map() lines in mirror() for any other key you need to move.
// ============================================================================
