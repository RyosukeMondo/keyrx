//! Standard Ctrl Modifier Library
//!
//! This file provides complete Ctrl+key mappings for use with virtual modifiers.
//! Load this file into a when_start("MD_XX") block to make that virtual modifier
//! behave like the physical Ctrl key.
//!
//! Usage:
//!   when_start("MD_01");
//!       load("ctrl.rhai");
//!   when_end();

// ============================================================================
// Common Editing Shortcuts (Ctrl+A-Z)
// ============================================================================
map("A", with_ctrl("VK_A"));      // Select All
map("B", with_ctrl("VK_B"));      // Bold (text editors)
map("C", with_ctrl("VK_C"));      // Copy
map("D", with_ctrl("VK_D"));      // Duplicate/Delete (varies by app)
map("E", with_ctrl("VK_E"));      // End of line (varies by app)
map("F", with_ctrl("VK_F"));      // Find
map("G", with_ctrl("VK_G"));      // Go to line/Find next
map("H", with_ctrl("VK_H"));      // Replace/History
map("I", with_ctrl("VK_I"));      // Italic (text editors)
map("J", with_ctrl("VK_J"));      // Join lines (varies by app)
map("K", with_ctrl("VK_K"));      // Insert link (varies by app)
map("L", with_ctrl("VK_L"));      // Select address bar/Go to line
map("M", with_ctrl("VK_M"));      // Mute tab (browsers)
map("N", with_ctrl("VK_N"));      // New window/file
map("O", with_ctrl("VK_O"));      // Open file
map("P", with_ctrl("VK_P"));      // Print
map("Q", with_ctrl("VK_Q"));      // Quit application
map("R", with_ctrl("VK_R"));      // Reload/Refresh
map("S", with_ctrl("VK_S"));      // Save
map("T", with_ctrl("VK_T"));      // New tab
map("U", with_ctrl("VK_U"));      // Underline (text editors)
map("V", with_ctrl("VK_V"));      // Paste
map("W", with_ctrl("VK_W"));      // Close window/tab
map("X", with_ctrl("VK_X"));      // Cut
map("Y", with_ctrl("VK_Y"));      // Redo
map("Z", with_ctrl("VK_Z"));      // Undo

// ============================================================================
// Number Keys (Ctrl+0-9)
// ============================================================================
map("Num0", with_ctrl("VK_Num0"));  // Reset zoom (browsers)
map("Num1", with_ctrl("VK_Num1"));  // Switch to tab 1
map("Num2", with_ctrl("VK_Num2"));  // Switch to tab 2
map("Num3", with_ctrl("VK_Num3"));  // Switch to tab 3
map("Num4", with_ctrl("VK_Num4"));  // Switch to tab 4
map("Num5", with_ctrl("VK_Num5"));  // Switch to tab 5
map("Num6", with_ctrl("VK_Num6"));  // Switch to tab 6
map("Num7", with_ctrl("VK_Num7"));  // Switch to tab 7
map("Num8", with_ctrl("VK_Num8"));  // Switch to tab 8
map("Num9", with_ctrl("VK_Num9"));  // Switch to last tab

// ============================================================================
// Navigation Keys
// ============================================================================
map("Left", with_ctrl("VK_Left"));        // Move cursor one word left
map("Right", with_ctrl("VK_Right"));      // Move cursor one word right
map("Up", with_ctrl("VK_Up"));            // Move to previous paragraph/page
map("Down", with_ctrl("VK_Down"));        // Move to next paragraph/page
map("Home", with_ctrl("VK_Home"));        // Go to start of document
map("End", with_ctrl("VK_End"));          // Go to end of document
map("PageUp", with_ctrl("VK_PageUp"));    // Previous tab/page
map("PageDown", with_ctrl("VK_PageDown"));// Next tab/page

// ============================================================================
// Editing Keys
// ============================================================================
map("Backspace", with_ctrl("VK_Backspace"));  // Delete previous word
map("Delete", with_ctrl("VK_Delete"));        // Delete next word
map("Insert", with_ctrl("VK_Insert"));        // Copy (alternative)
map("Enter", with_ctrl("VK_Enter"));          // Insert line break (varies)

// ============================================================================
// Function Keys (Common Ctrl+F combinations)
// ============================================================================
map("F1", with_ctrl("VK_F1"));    // Help (varies by app)
map("F2", with_ctrl("VK_F2"));    // Rename (varies by app)
map("F3", with_ctrl("VK_F3"));    // Find next
map("F4", with_ctrl("VK_F4"));    // Close window/tab
map("F5", with_ctrl("VK_F5"));    // Hard refresh (browsers)
map("F6", with_ctrl("VK_F6"));    // Move focus (varies by app)
map("F7", with_ctrl("VK_F7"));    // Spell check (varies by app)
map("F8", with_ctrl("VK_F8"));    // (varies by app)
map("F9", with_ctrl("VK_F9"));    // (varies by app)
map("F10", with_ctrl("VK_F10"));  // (varies by app)
map("F11", with_ctrl("VK_F11"));  // (varies by app)
map("F12", with_ctrl("VK_F12"));  // Developer tools (browsers)

// ============================================================================
// Special Keys
// ============================================================================
map("Tab", with_ctrl("VK_Tab"));              // Switch window/tab forward
map("Space", with_ctrl("VK_Space"));          // Autocomplete/Select (varies)
map("Escape", with_ctrl("VK_Escape"));        // (varies by app)
map("Equal", with_ctrl("VK_Equal"));          // Zoom in (browsers)
map("Minus", with_ctrl("VK_Minus"));          // Zoom out (browsers)
map("LeftBracket", with_ctrl("VK_LeftBracket"));   // Decrease font size (varies)
map("RightBracket", with_ctrl("VK_RightBracket")); // Increase font size (varies)
map("Slash", with_ctrl("VK_Slash"));          // Toggle comment (code editors)
map("Backslash", with_ctrl("VK_Backslash"));  // (varies by app)
