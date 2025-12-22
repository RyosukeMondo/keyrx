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
  // Format: map("VK_<source>", "VK_<target>")
  map("VK_A", "VK_B");  // Press A, get B

  // You can remap any key to any other key
  map("VK_Q", "VK_W");  // Press Q, get W

  // Number keys work the same way
  map("VK_Num1", "VK_Num2");  // Press 1, get 2

device_end();

// ============================================================================
// NOTES:
// - All key names must start with "VK_" prefix (Virtual Key)
// - Key names are case-sensitive (VK_A not vk_a)
// - See DSL_MANUAL.md for complete list of key names
// - Simple remapping is unidirectional: A→B doesn't affect B
// ============================================================================
