// ============================================================================
// Example 02: CapsLock to Escape (Classic Remap)
// ============================================================================
//
// PURPOSE:
//   Demonstrates a popular keyboard modification: remapping CapsLock to Escape.
//   This is one of the most common keyboard customizations, especially for
//   Vim/Emacs users who need easy access to Escape.
//
// WHAT THIS DOES:
//   - Remaps CapsLock key to Escape
//   - Optionally adds Shift+CapsLock to still toggle CapsLock when needed
//
// EXPECTED BEHAVIOR:
//   - Pressing CapsLock will send Escape instead
//   - Shift+CapsLock will toggle CapsLock on/off (optional)
//
// COMPILATION:
//   cargo run --bin keyrx_compiler -- compile examples/02-capslock-escape.rhai
//
// WHY THIS IS USEFUL:
//   - Escape is heavily used in Vim/Emacs but far from home row
//   - CapsLock is rarely used but in a prime location
//   - This swap improves ergonomics for text editing
//
// ============================================================================

device_start("*");

  // Simple approach: CapsLock becomes Escape
  map("VK_CapsLock", "VK_Escape");

  // Optional: If you still need CapsLock occasionally, you can use a modifier
  // Note: This requires using a custom modifier layer (see example 03)
  // For now, we'll just do the simple remap

device_end();

// ============================================================================
// NOTES:
// - After this remap, you cannot toggle CapsLock in the normal way
// - If you need CapsLock occasionally, see example 03 for conditional mappings
// - You can also remap Escape to CapsLock if you want to swap them
// - This is one of the most popular keyboard customizations
//
// VARIATIONS:
// - CapsLock → Ctrl (another popular choice)
// - CapsLock → Backspace (for easier deletion)
// - CapsLock → dual-function (tap=Escape, hold=Ctrl) - see example 04
//
// SEE ALSO:
// - Example 04 for dual-function CapsLock (tap/hold)
// - Example 03 for using CapsLock as a layer modifier
// ============================================================================
