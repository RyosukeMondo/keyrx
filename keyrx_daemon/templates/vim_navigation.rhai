// Vim Navigation Template
//
// Hold CapsLock and use HJKL for arrow key navigation.
// Works system-wide in any application.

device_start("*");
  // Make CapsLock a custom modifier
  map("VK_CapsLock", "MD_00");

  // When CapsLock is held, HJKL becomes arrow keys
  when_start("MD_00");
    map("VK_H", "VK_Left");   // H → Left
    map("VK_J", "VK_Down");   // J → Down
    map("VK_K", "VK_Up");     // K → Up
    map("VK_L", "VK_Right");  // L → Right
  when_end();

  // Additional navigation keys
  when_start("MD_00");
    map("VK_U", "VK_PageUp");    // U → Page Up
    map("VK_D", "VK_PageDown");  // D → Page Down
    map("VK_I", "VK_Home");      // I → Home
    map("VK_A", "VK_End");       // A → End
  when_end();
device_end();
