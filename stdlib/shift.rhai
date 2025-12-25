//! Standard Shift Modifier Library
//!
//! This file provides complete shifted key mappings for use with virtual modifiers.
//! Load this file into a when_start("MD_XX") block to make that virtual modifier
//! behave like the physical Shift key.
//!
//! Usage:
//!   when_start("MD_00");
//!       load("shift.rhai");
//!   when_end();

// ============================================================================
// Uppercase Letters (A-Z)
// ============================================================================
map("A", with_shift("VK_A"));
map("B", with_shift("VK_B"));
map("C", with_shift("VK_C"));
map("D", with_shift("VK_D"));
map("E", with_shift("VK_E"));
map("F", with_shift("VK_F"));
map("G", with_shift("VK_G"));
map("H", with_shift("VK_H"));
map("I", with_shift("VK_I"));
map("J", with_shift("VK_J"));
map("K", with_shift("VK_K"));
map("L", with_shift("VK_L"));
map("M", with_shift("VK_M"));
map("N", with_shift("VK_N"));
map("O", with_shift("VK_O"));
map("P", with_shift("VK_P"));
map("Q", with_shift("VK_Q"));
map("R", with_shift("VK_R"));
map("S", with_shift("VK_S"));
map("T", with_shift("VK_T"));
map("U", with_shift("VK_U"));
map("V", with_shift("VK_V"));
map("W", with_shift("VK_W"));
map("X", with_shift("VK_X"));
map("Y", with_shift("VK_Y"));
map("Z", with_shift("VK_Z"));

// ============================================================================
// Shifted Numbers (! @ # $ % ^ & * ( ))
// ============================================================================
map("Num1", with_shift("VK_Num1"));  // 1 → !
map("Num2", with_shift("VK_Num2"));  // 2 → @
map("Num3", with_shift("VK_Num3"));  // 3 → #
map("Num4", with_shift("VK_Num4"));  // 4 → $
map("Num5", with_shift("VK_Num5"));  // 5 → %
map("Num6", with_shift("VK_Num6"));  // 6 → ^
map("Num7", with_shift("VK_Num7"));  // 7 → &
map("Num8", with_shift("VK_Num8"));  // 8 → *
map("Num9", with_shift("VK_Num9"));  // 9 → (
map("Num0", with_shift("VK_Num0"));  // 0 → )

// ============================================================================
// Shifted Punctuation
// ============================================================================
map("Minus", with_shift("VK_Minus"));              // - → _
map("Equal", with_shift("VK_Equal"));              // = → +
map("LeftBracket", with_shift("VK_LeftBracket"));  // [ → {
map("RightBracket", with_shift("VK_RightBracket"));// ] → }
map("Backslash", with_shift("VK_Backslash"));      // \ → |
map("Semicolon", with_shift("VK_Semicolon"));      // ; → :
map("Quote", with_shift("VK_Quote"));              // ' → "
map("Comma", with_shift("VK_Comma"));              // , → <
map("Period", with_shift("VK_Period"));            // . → >
map("Slash", with_shift("VK_Slash"));              // / → ?
map("Grave", with_shift("VK_Grave"));              // ` → ~
