// ============================================================================
// Example 01: Simple Key Remapping
// ============================================================================
//
// PURPOSE:
//   Demonstrates the most basic keyrx functionality: remapping one key to another.
//   This example shows how to use the map() function to create simple 1:1 key mappings.
//
// WHAT THIS DOES:
//   - Remaps the 'A' key to output 'B' instead
//   - Remaps the 'Q' key to output 'W' instead
//   - Remaps the '1' key (top row) to output '2' instead
//
// EXPECTED BEHAVIOR:
//   - When you press the 'A' key, it will type 'B'
//   - When you press the 'Q' key, it will type 'W'
//   - When you press the '1' key, it will type '2'
//
// COMPILATION:
//   cargo run --bin keyrx_compiler -- compile examples/01-simple-remap.rhai
//
// USAGE:
//   After compilation, load the generated .krx file with the keyrx daemon.
//
// ============================================================================

// All mappings must be inside a device block
// The device pattern "*" matches all input devices (keyboards)
device_start("*");

  // Basic remapping: map physical key to virtual key
  // Format: map("<source>", "VK_<target>")
  map("A", "VK_B");  // Press A, get B

  // You can remap any key to any other key
  map("Q", "VK_W");  // Press Q, get W

  // Number keys work the same way
  map("Num1", "VK_Num2");  // Press 1, get 2

device_end();

// ============================================================================
// NOTES:
// - Key naming rule: the key you PRESS (first argument) is a bare name
//   ("A", "CapsLock"); the key that is OUTPUT (second argument) needs the
//   "VK_" prefix ("VK_B"). A "VK_" on the first argument is accepted and
//   means the same key, but the bare form is the convention.
// - Key names are case-sensitive ("CapsLock", not "capslock")
// - Mapping the same key twice in one scope is a compile error
// - See docs/user-guide/dsl-manual.md for the complete list of key names
// - Simple remapping is unidirectional: A→B doesn't affect B
// ============================================================================
