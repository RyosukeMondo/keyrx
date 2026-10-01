// Blank Profile Template
//
// This is a minimal valid keyrx configuration.
// Add your key mappings between device_start() and device_end().
//
// WHICH KEYBOARDS: device_start("*") applies to EVERY keyboard keyrx can see,
// including USB/Bluetooth ones and other programs' virtual keyboards. To
// limit the profile to one keyboard, run `keyrx_daemon list-devices` and put
// part of its name in the pattern, e.g. device_start("*Logitech*");
// (case-insensitive; `*` matches anything).

device_start("*");
  // Add your key mappings here
  // Example: map("VK_A", "VK_B");
device_end();
