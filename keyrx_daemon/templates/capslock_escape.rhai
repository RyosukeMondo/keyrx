// CapsLock to Escape Template
//
// Remaps CapsLock to Escape - a popular modification for Vim/Emacs users.
// CapsLock is rarely used but in a prime location on most keyboards.

device_start("*");
  // Remap CapsLock to Escape
  map("VK_CapsLock", "VK_Escape");
device_end();
