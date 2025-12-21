// Simple KeyRx Configuration Example
//
// This example demonstrates basic key remapping using the KeyRx DSL.
// All output keys must have the appropriate prefix:
//   VK_ = Virtual Key (standard key output)
//   MD_ = Custom Modifier (custom modifier state, MD_00 through MD_FE)
//   LK_ = Custom Lock (toggle state, LK_00 through LK_FE)

// Define mappings for the default device
// Pattern "*" matches all devices
device_start("*");

    // Simple key remapping: CapsLock becomes Escape
    // Input keys have no prefix, output keys require VK_ prefix
    map("CapsLock", "VK_Escape");

    // Swap A and B keys
    map("A", "VK_B");
    map("B", "VK_A");

    // Remap number row to function keys
    map("1", "VK_F1");
    map("2", "VK_F2");
    map("3", "VK_F3");
    map("4", "VK_F4");

    // Arrow keys on the right hand home row
    // These use VK_ prefix for virtual key output
    map("J", "VK_Left");
    map("K", "VK_Down");
    map("L", "VK_Up");
    map("Semicolon", "VK_Right");

device_end();

// Device-specific configuration using pattern matching
// Pattern can match device name, vendor ID, product ID, etc.
device_start("USB Keyboard");

    // Remap Enter to Space on this specific device
    map("Enter", "VK_Space");

device_end();
