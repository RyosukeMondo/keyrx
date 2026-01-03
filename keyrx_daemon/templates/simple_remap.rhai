// Simple Remap Template
//
// Basic one-to-one key remapping examples.
// Press the source key to get the target key.

device_start("*");
  // Example remappings - customize as needed
  map("VK_A", "VK_B");  // Press A, get B
  map("VK_Q", "VK_W");  // Press Q, get W
device_end();
