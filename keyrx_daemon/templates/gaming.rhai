// Gaming Profile Template
//
// Optimized key mappings for gaming.
// Common WASD adjustments and quick access keys.
//
// WHICH KEYBOARDS: device_start("*") applies to EVERY keyboard keyrx can see,
// including other programs' virtual keyboards and any gamepad/keyboard
// combos. Limit it to your gaming keyboard: run `keyrx_daemon list-devices`
// and use part of its name, e.g. device_start("*Razer*");

device_start("*");
  // Optional: Remap CapsLock to Ctrl for easier crouching
  map("VK_CapsLock", "VK_LCtrl");

  // Optional: Remap nearby keys for easier reach
  // Example: Q and E for abilities/actions
  // (These are already standard gaming keys, customize as needed)

  // Optional: Number row to function keys for quick item access
  // map("VK_Num1", "VK_F1");
  // map("VK_Num2", "VK_F2");
  // map("VK_Num3", "VK_F3");
  // map("VK_Num4", "VK_F4");
device_end();
