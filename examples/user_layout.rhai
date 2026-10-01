// Custom keyboard layout (based on user's original config)
//
// This layout demonstrates keyrx's internationalization support with:
// - Japanese key names (日本語)
// - TapHold modifiers (MD_00-MD_09)
// - Modified output (Shift+key)
// - Conditional layers (when MD_00 held)

device_start("*");  // Apply to all keyboards

// ============================================================
// TapHold Modifiers (MD_00 - MD_09)
// Hold = activate modifier, Tap = output key
// ============================================================
tap_hold("B", "VK_Enter", "MD_00", 200);       // B: tap=Enter, hold=MD_00
tap_hold("V", "VK_Delete", "MD_01", 200);      // V: tap=Delete, hold=MD_01
tap_hold("M", "VK_Backspace", "MD_02", 200);   // M: tap=Backspace, hold=MD_02
tap_hold("X", "VK_Delete", "MD_03", 200);      // X: tap=Delete, hold=MD_03
tap_hold("Num1", "VK_Num1", "MD_04", 200);     // 1: tap=1 (passthrough), hold=MD_04
tap_hold("LCtrl", "VK_Space", "MD_05", 200);   // LCtrl: tap=Space, hold=MD_05
tap_hold("C", "VK_Delete", "MD_06", 200);      // C: tap=Delete, hold=MD_06
tap_hold("Tab", "VK_Space", "MD_07", 200);     // Tab: tap=Space, hold=MD_07
tap_hold("Q", "VK_Minus", "MD_08", 200);       // Q: tap=Minus, hold=MD_08
tap_hold("A", "VK_Tab", "MD_09", 200);         // A: tap=Tab, hold=MD_09
hold_only("LMeta", "MD_07");                  // LWin: hold=MD_07, tap=suppressed

// ============================================================
// MD_00 Layer (B hold) - Using PHYSICAL key positions
// Navigation, Numbers, Function keys
// ============================================================
when_start("MD_00");
    // Special
    map("Zenkaku", "VK_Enter");                // 半/全 -> Enter

    // Navigation (physical number row keys)
    map("Num2", "VK_Left");                    // phys 2 -> Left
    map("Num3", "VK_Right");                   // phys 3 (outputs ,) -> Right
    map("Num4", "VK_Down");                    // phys 4 (outputs .) -> Down
    map("Num5", "VK_Up");                      // phys 5 (outputs P) -> Up
    map("Num8", "VK_Home");                    // phys 8 (outputs G) -> Home
    map("Num9", "VK_End");                     // phys 9 (outputs C) -> End

    // @ (Atmark) = Shift+2 on JIS keyboard (physical keys)
    map("Slash", with_shift("VK_Num2"));       // phys / -> @

    // Numbers on Dvorak home row (physical QWERTY top row)
    // Physical: W  E  R  T  Y  U  I  O  P  [
    // Outputs:  A  O  E  U  I  D  H  T  N  S
    // MD_00:    1  2  3  4  5  6  7  8  9  0
    map("Minus", "VK_F12");                    // phys - -> F12
    map("W", "VK_Num1");                       // phys W (outputs A) -> 1
    map("E", "VK_Num2");                       // phys E (outputs O) -> 2
    map("R", "VK_Num3");                       // phys R (outputs E) -> 3
    map("T", "VK_Num4");                       // phys T (outputs U) -> 4
    map("Y", "VK_Num5");                       // phys Y (outputs I) -> 5
    map("U", "VK_Num6");                       // phys U (outputs D) -> 6
    map("I", "VK_Num7");                       // phys I (outputs H) -> 7
    map("O", "VK_Num8");                       // phys O (outputs T) -> 8
    map("P", "VK_Num9");                       // phys P (outputs N) -> 9
    map("LeftBracket", "VK_Num0");             // phys [ (outputs S) -> 0

    // Function keys (physical home row SDFGHJKL;, -> F1-F10)
    map("S", "VK_F1");                         // phys S -> F1
    map("D", "VK_F2");                         // phys D -> F2
    map("F", "VK_F3");                         // phys F -> F3
    map("G", "VK_F4");                         // phys G -> F4
    map("H", "VK_F5");                         // phys H -> F5
    map("J", "VK_F6");                         // phys J -> F6
    map("K", "VK_F7");                         // phys K -> F7
    map("L", "VK_F8");                         // phys L -> F8
    map("Semicolon", "VK_F9");                 // phys ; -> F9
    map("Quote", "VK_F10");                    // phys ' -> F10
    map("Backslash", "VK_F11");                // phys \ -> F11
    map("RightBracket", "VK_F12");             // phys ] -> F12

    // Space -> F12
    map("Space", "VK_F12");                    // phys Space -> F12
when_end();

// ============================================================
// MD_01 Layer (V hold) - Left/Right Mirror
// Rows: [2 - -], [W - [S]], [S - '] each 10 keys mirrored
// ============================================================
when_start("MD_01");
    // Row 1: Number row mirror (2 3 4 5 6 | 7 8 9 0 -)
    // Base:  '  ,  .  P  Y  |  F  G  C  R  L
    map("Num2", "VK_L");                       // 2 -> L (mirror of -)
    map("Num3", "VK_R");                       // 3 -> R (mirror of 0)
    map("Num4", "VK_C");                       // 4 -> C (mirror of 9)
    map("Num5", "VK_G");                       // 5 -> G (mirror of 8)
    map("Num6", "VK_F");                       // 6 -> F (mirror of 7)
    map("Num7", "VK_Y");                       // 7 -> Y (mirror of 6)
    map("Num8", "VK_P");                       // 8 -> P (mirror of 5)
    map("Num9", "VK_Period");                  // 9 -> . (mirror of 4)
    map("Num0", "VK_Comma");                   // 0 -> , (mirror of 3)
    map("Minus", with_shift("VK_Num7"));       // - -> ' (mirror of 2)

    // Row 2: Top letter row mirror (W E R T Y | U I O P [)
    // Base:  A  O  E  U  I  |  D  H  T  N  S
    map("W", "VK_S");                          // W -> S (mirror of [)
    map("E", "VK_N");                          // E -> N (mirror of P)
    map("R", "VK_T");                          // R -> T (mirror of O)
    map("T", "VK_H");                          // T -> H (mirror of I)
    map("Y", "VK_D");                          // Y -> D (mirror of U)
    map("U", "VK_I");                          // U -> I (mirror of Y)
    map("I", "VK_U");                          // I -> U (mirror of T)
    map("O", "VK_E");                          // O -> E (mirror of R)
    map("P", "VK_O");                          // P -> O (mirror of E)
    map("LeftBracket", "VK_A");                // [ -> A (mirror of W)

    // Row 3: Home row mirror (S D F G H | J K L ; ')
    // Base:  ;  Q  J  K  X  |  B  M  W  V  Z
    map("S", "VK_Z");                          // S -> Z (mirror of ')
    map("D", "VK_V");                          // D -> V (mirror of ;)
    map("F", "VK_W");                          // F -> W (mirror of L)
    map("G", "VK_M");                          // G -> M (mirror of K)
    map("H", "VK_B");                          // H -> B (mirror of J)
    map("J", "VK_X");                          // J -> X (mirror of H)
    map("K", "VK_K");                          // K -> K (mirror of G)
    map("L", "VK_J");                          // L -> J (mirror of F)
    map("Semicolon", "VK_Q");                  // ; -> Q (mirror of D)
    map("Quote", "VK_Semicolon");              // ' -> ; (mirror of S)
when_end();

// ============================================================
// MD_02 Layer (M hold) - Symbols
// Row 1: @#$%^&*|() [2 - -]
// Row 2: (){}()[]<> [W - []
// Row 3: ?/^!`~*+=\ [S - ']
// ============================================================
when_start("MD_02");
    // Row 1: Symbols @#$%^&*|() on number row
    // Mappings adjusted for JIS output
    map("Num2", "VK_LeftBracket");             // 2 -> @ (JIS @ key)
    map("Num3", with_shift("VK_Num3"));        // 3 -> #
    map("Num4", with_shift("VK_Num4"));        // 4 -> $
    map("Num5", with_shift("VK_Num5"));        // 5 -> %
    map("Num6", "VK_Equal");                   // 6 -> ^ (JIS ^ key)
    map("Num7", with_shift("VK_Num6"));        // 7 -> & (JIS Shift+6)
    map("Num8", with_shift("VK_Quote"));       // 8 -> * (JIS Shift+:)
    map("Num9", with_shift("VK_Yen"));         // 9 -> | (JIS Shift+Yen)
    map("Num0", with_shift("VK_Num8"));        // 0 -> ( (JIS Shift+8)
    map("Minus", with_shift("VK_Num9"));       // - -> ) (JIS Shift+9)

    // Row 2: Brackets (){}()[]<> on top letter row
    map("W", with_shift("VK_Num8"));           // W -> (
    map("E", with_shift("VK_Num9"));           // E -> )
    map("R", with_shift("VK_RightBracket"));   // R -> { (JIS Shift+[)
    map("T", with_shift("VK_Backslash"));      // T -> } (JIS Shift+])
    map("Y", with_shift("VK_Num8"));           // Y -> (
    map("U", with_shift("VK_Num9"));           // U -> )
    map("I", "VK_RightBracket");               // I -> [ (JIS [)
    map("O", "VK_Backslash");                  // O -> ] (JIS ])
    map("P", with_shift("VK_Comma"));          // P -> <
    map("LeftBracket", with_shift("VK_Period")); // [ -> >

    // Row 3: Symbols ?/^!`~*+=\ on home row
    map("S", with_shift("VK_Slash"));          // S -> ?
    map("D", "VK_Slash");                      // D -> /
    map("F", "VK_Equal");                      // F -> ^
    map("G", with_shift("VK_Num1"));           // G -> !
    map("H", with_shift("VK_LeftBracket"));    // H -> ` (JIS Shift+@)
    map("J", with_shift("VK_Equal"));          // J -> ~ (JIS Shift+^)
    map("K", with_shift("VK_Quote"));          // K -> * (JIS Shift+:)
    map("L", with_shift("VK_Semicolon"));      // L -> + (JIS Shift+;)
    map("Semicolon", with_shift("VK_Minus"));  // ; -> = (JIS Shift+-)
    map("Quote", "VK_Ro");                     // ' -> \ (JIS Ro/Backslash)
when_end();

// ============================================================
// MD_03 Layer (X hold) - Symbols Mirror (MD_02 mirrored)
// Row 1: )(|*&^%$#@ [2 - -] (mirror of @#$%^&*|())
// Row 2: ><][)(}{)( [W - [] (mirror of (){}()[]<>)
// Row 3: \=+*~`!^/? [S - '] (mirror of ?/^!`~*+=\)
// ============================================================
when_start("MD_03");
    // Row 1: Mirrored symbols )(|*&^%$#@
    map("Num2", with_shift("VK_Num0"));        // 2 -> ) (mirror of -)
    map("Num3", with_shift("VK_Num9"));        // 3 -> ( (mirror of 0)
    map("Num4", with_shift("VK_Backslash"));   // 4 -> | (mirror of 9)
    map("Num5", with_shift("VK_Num8"));        // 5 -> * (mirror of 8)
    map("Num6", with_shift("VK_Num7"));        // 6 -> & (mirror of 7)
    map("Num7", with_shift("VK_Num6"));        // 7 -> ^ (mirror of 6)
    map("Num8", with_shift("VK_Num5"));        // 8 -> % (mirror of 5)
    map("Num9", with_shift("VK_Num4"));        // 9 -> $ (mirror of 4)
    map("Num0", with_shift("VK_Num3"));        // 0 -> # (mirror of 3)
    map("Minus", with_shift("VK_Num2"));       // - -> @ (mirror of 2)

    // Row 2: Mirrored brackets ><][)(}{)(
    map("W", with_shift("VK_Period"));         // W -> > (mirror of [)
    map("E", with_shift("VK_Comma"));          // E -> < (mirror of P)
    map("R", "VK_RightBracket");               // R -> ] (mirror of O)
    map("T", "VK_LeftBracket");                // T -> [ (mirror of I)
    map("Y", with_shift("VK_Num0"));           // Y -> ) (mirror of U)
    map("U", with_shift("VK_Num9"));           // U -> ( (mirror of Y)
    map("I", with_shift("VK_RightBracket"));   // I -> } (mirror of T)
    map("O", with_shift("VK_LeftBracket"));    // O -> { (mirror of R)
    map("P", with_shift("VK_Num0"));           // P -> ) (mirror of E)
    map("LeftBracket", with_shift("VK_Num9")); // [ -> ( (mirror of W)

    // Row 3: Mirrored symbols \=+*~`!^/?
    map("S", "VK_Backslash");                  // S -> \ (mirror of ')
    map("D", "VK_Equal");                      // D -> = (mirror of ;)
    map("F", with_shift("VK_Equal"));          // F -> + (mirror of L)
    map("G", with_shift("VK_Num8"));           // G -> * (mirror of K)
    map("H", with_shift("VK_Grave"));          // H -> ~ (mirror of J)
    map("J", "VK_Grave");                      // J -> ` (mirror of H)
    map("K", with_shift("VK_Num1"));           // K -> ! (mirror of G)
    map("L", with_shift("VK_Num6"));           // L -> ^ (mirror of F)
    map("Semicolon", "VK_Slash");              // ; -> / (mirror of D)
    map("Quote", with_shift("VK_Slash"));      // ' -> ? (mirror of S)
when_end();

// ============================================================
// MD_04 Layer (1 hold) - Common Shortcuts
// Row 1: C-Z,C-X,C-C,C-V,C-Y [2 - 6]
// Row 2: C-G,A-X,C-W,Kanji,S-Apps [W - Y]
// Row 3: C-A,C-F,C-J,A-Space,C-S-B [S - H]
// ============================================================
when_start("MD_04");
    // Row 1: Undo/Cut/Copy/Paste/Redo shortcuts
    // with_mods(key, shift, ctrl, alt, win)
    map("Num2", with_mods("VK_Z", false, true, false, false));  // 2 -> C-Z (Undo)
    map("Num3", with_mods("VK_X", false, true, false, false));  // 3 -> C-X (Cut)
    map("Num4", with_mods("VK_C", false, true, false, false));  // 4 -> C-C (Copy)
    map("Num5", with_mods("VK_V", false, true, false, false));  // 5 -> C-V (Paste)
    map("Num6", with_mods("VK_Y", false, true, false, false));  // 6 -> C-Y (Redo)

    // Row 2: Various shortcuts
    map("W", with_mods("VK_G", false, true, false, false));     // W -> C-G (Find Next)
    map("E", with_mods("VK_X", false, false, true, false));     // E -> A-X (Emacs M-x)
    map("R", with_mods("VK_W", false, true, false, false));     // R -> C-W (Close)
    map("T", "VK_Zenkaku");                                     // T -> Kanji/Zenkaku
    map("Y", with_mods("VK_Menu", true, false, false, false));  // Y -> S-Apps (Context Menu)

    // Row 3: More shortcuts
    map("S", with_mods("VK_A", false, true, false, false));     // S -> C-A (Select All)
    map("D", with_mods("VK_F", false, true, false, false));     // D -> C-F (Find)
    map("F", with_mods("VK_J", false, true, false, false));     // F -> C-J
    map("G", with_mods("VK_Space", false, false, true, false)); // G -> A-Space (Window Menu)
    map("H", with_mods("VK_B", true, true, false, false));      // H -> C-S-B (Ctrl+Shift+B)
when_end();

// ============================================================
// MD_05 Layer (LCtrl hold) - Numbers & F-keys (reversed)
// Row 1: 09876 [W - Y]
// Row 2: F10,F9,F8,F7,F6 [S - H]
// ============================================================
when_start("MD_05");
    // Row 1: Numbers reversed (right hand access on left)
    map("W", "VK_Num0");                       // W -> 0
    map("E", "VK_Num9");                       // E -> 9
    map("R", "VK_Num8");                       // R -> 8
    map("T", "VK_Num7");                       // T -> 7
    map("Y", "VK_Num6");                       // Y -> 6

    // Row 2: Function keys reversed
    map("S", "VK_F10");                        // S -> F10
    map("D", "VK_F9");                         // D -> F9
    map("F", "VK_F8");                         // F -> F8
    map("G", "VK_F7");                         // G -> F7
    map("H", "VK_F6");                         // H -> F6
when_end();

// ============================================================
// MD_06 Layer (C hold) - Navigation & Tab Control
// Row 1: C-PageUp,C-PageDown,C-S-Tab,C-Tab,C-F4 [2 - 6]
// Row 2: C-Left,C-Right,C-Down,C-Up,C-W [W - Y]
// Row 3: A-Left,A-Right,A-Down,A-Up,A-F5 [S - H]
// ============================================================
when_start("MD_06");
    // Row 1: Tab/Window navigation
    // with_mods(key, shift, ctrl, alt, win)
    map("Num2", with_mods("VK_PageUp", false, true, false, false));   // 2 -> C-PageUp (Prev Tab)
    map("Num3", with_mods("VK_PageDown", false, true, false, false)); // 3 -> C-PageDown (Next Tab)
    map("Num4", with_mods("VK_Tab", true, true, false, false));       // 4 -> C-S-Tab (Prev Tab)
    map("Num5", with_mods("VK_Tab", false, true, false, false));      // 5 -> C-Tab (Next Tab)
    map("Num6", with_mods("VK_F4", false, true, false, false));       // 6 -> C-F4 (Close Tab)

    // Row 2: Ctrl+Arrow (word navigation)
    map("W", with_mods("VK_Left", false, true, false, false));        // W -> C-Left (Word Left)
    map("E", with_mods("VK_Right", false, true, false, false));       // E -> C-Right (Word Right)
    map("R", with_mods("VK_Down", false, true, false, false));        // R -> C-Down
    map("T", with_mods("VK_Up", false, true, false, false));          // T -> C-Up
    map("Y", with_mods("VK_W", false, true, false, false));           // Y -> C-W (Close)

    // Row 3: Alt+Arrow (history/special navigation)
    map("S", with_mods("VK_Left", false, false, true, false));        // S -> A-Left (Back)
    map("D", with_mods("VK_Right", false, false, true, false));       // D -> A-Right (Forward)
    map("F", with_mods("VK_Down", false, false, true, false));        // F -> A-Down
    map("G", with_mods("VK_Up", false, false, true, false));          // G -> A-Up
    map("H", with_mods("VK_F5", false, false, true, false));          // H -> A-F5
when_end();

// ============================================================
// MD_07 Layer (LWin hold) - Number row
// ============================================================
when_start("MD_07");
    map("Num2", "VK_Num1");                    // 2 -> 1
    map("Num3", "VK_Num2");                    // 3 -> 2
    map("Num4", "VK_Num3");                    // 4 -> 3
    map("Num5", "VK_Num4");                    // 5 -> 4
    map("Num6", "VK_Num5");                    // 6 -> 5
    map("Y", "VK_Num6");                       // Y -> 6
    map("T", "VK_Num7");                       // T -> 7
    map("R", "VK_Num8");                       // R -> 8
    map("E", "VK_Num9");                       // E -> 9
    map("W", "VK_Num0");                       // W -> 0
when_end();

// ============================================================
// MD_08 Layer (Q hold) - Home/End Selection
// Row 2: Home,End,S-Home,S-End [W - T]
// ============================================================
when_start("MD_08");
    // Row 2: Home/End and selection
    // with_mods(key, shift, ctrl, alt, win)
    map("W", "VK_Home");                                        // W -> Home
    map("E", "VK_End");                                         // E -> End
    map("R", with_mods("VK_Home", true, false, false, false));  // R -> S-Home (Select to start)
    map("T", with_mods("VK_End", true, false, false, false));   // T -> S-End (Select to end)
when_end();

// ============================================================
// MD_0A Layer (N hold) - Shift layer with JIS colon fix
// N acts as Shift for all keys. On JIS, Shift+Semicolon='+',
// so S is mapped to VK_Quote (JIS colon) instead.
// ============================================================
when_start("MD_0A");
    // Letters: shifted versions of base-layer Dvorak remaps
    map("W", with_shift("VK_A"));              // W→A → Shift+A
    map("E", with_shift("VK_O"));              // E→O → Shift+O
    map("R", with_shift("VK_E"));              // R→E → Shift+E
    map("T", with_shift("VK_U"));              // T→U → Shift+U
    map("Y", with_shift("VK_I"));              // Y→I → Shift+I
    map("U", with_shift("VK_D"));              // U→D → Shift+D
    map("I", with_shift("VK_H"));              // I→H → Shift+H
    map("O", with_shift("VK_T"));              // O→T → Shift+T
    map("P", with_shift("VK_N"));              // P→N → Shift+N
    map("D", with_shift("VK_Q"));              // D→Q → Shift+Q
    map("F", with_shift("VK_J"));              // F→J → Shift+J
    map("G", with_shift("VK_K"));              // G→K → Shift+K
    map("H", with_shift("VK_X"));              // H→X → Shift+X
    map("J", with_shift("VK_B"));              // J→B → Shift+B
    map("K", with_shift("VK_M"));              // K→M → Shift+M
    map("L", with_shift("VK_W"));              // L→W → Shift+W

    // TapHold keys: shifted versions of their tap outputs
    map("B", with_shift("VK_Enter"));            // B→Enter → Shift+Enter

    // JIS colon fix: Quote key = ':' unshifted on JIS
    map("S", "VK_Quote");                       // S → : (colon on JIS)

    // Other base-layer keys with shift
    map("LeftBracket", with_shift("VK_S"));     // [→S → Shift+S
    map("Quote", with_shift("VK_Z"));           // '→Z → Shift+Z
    map("Semicolon", with_shift("VK_V"));       // ;→V → Shift+V
    map("Minus", with_shift("VK_L"));           // -→L → Shift+L
    map("Equal", with_shift("VK_Slash"));        // =→/ → Shift+/

    // Number row: shifted Dvorak outputs
    map("Num2", with_shift("VK_Num2"));          // 2→' → " (double quote on JIS)
    map("Num3", with_shift("VK_Comma"));        // 3→, → <
    map("Num4", with_shift("VK_Period"));       // 4→. → >
    map("Num5", with_shift("VK_P"));            // 5→P → Shift+P
    map("Num6", with_shift("VK_Y"));            // 6→Y → Shift+Y
    map("Num7", with_shift("VK_F"));            // 7→F → Shift+F
    map("Num8", with_shift("VK_G"));            // 8→G → Shift+G
    map("Num9", with_shift("VK_C"));            // 9→C → Shift+C
    map("Num0", with_shift("VK_R"));            // 0→R → Shift+R

    // JIS underscore: Shift+Ro = _
    map("Ro", with_shift("VK_Ro"));            // ろ→- → _ (underscore on JIS)
    map("RightBracket", with_shift("VK_Ro"));  // [→- → _ (underscore on JIS)

    // Space passthrough
    map("Space", "VK_Space");
when_end();

// ============================================================
// Modified Output (with Shift)
// ============================================================
// JIS keyboard: ' (single quote) = Shift+7
map("Num2", with_shift("VK_Num7"));            // 2 -> ' (Shift+7 on JIS)

// ============================================================
// Simple Remappings
// ============================================================
map("LeftBracket", "VK_S");    // [ -> S
map("RightBracket", "VK_Minus"); // ] -> -
map("Quote", "VK_Z");          // ' -> Z
map("Equal", "VK_Slash");      // = -> /

// === Letter Row Remappings (excluding TapHold keys) ===
// Note: A, B, C, M, Q, V, X are now TapHold - removed from simple mappings
map("D", "VK_Q");                // D -> Q
map("E", "VK_O");                // E -> O
map("F", "VK_J");                // F -> J
map("G", "VK_K");                // G -> K
map("H", "VK_X");                // H -> X
map("I", "VK_H");                // I -> H
map("J", "VK_B");                // J -> B
map("K", "VK_M");                // K -> M
map("L", "VK_W");                // L -> W
tap_hold("N", "VK_LShift", "MD_0A", 200); // N: tap=LShift, hold=MD_0A (shift layer)
map("O", "VK_T");                // O -> T
map("P", "VK_N");                // P -> N
map("R", "VK_E");                // R -> E
map("S", "VK_Semicolon");        // S -> Semicolon
map("T", "VK_U");                // T -> U
map("U", "VK_D");                // U -> D
map("W", "VK_A");                // W -> A
map("Y", "VK_I");                // Y -> I
// Z -> Z (passthrough)

// === Number Row Remappings (excluding TapHold/modified keys) ===
// Note: 1 is TapHold, 2 is modified output
map("Num3", "VK_Comma");         // 3 -> Comma
map("Num4", "VK_Period");        // 4 -> Period
map("Num5", "VK_P");             // 5 -> P
map("Num6", "VK_Y");             // 6 -> Y
map("Num7", "VK_F");             // 7 -> F
map("Num8", "VK_G");             // 8 -> G
map("Num9", "VK_C");             // 9 -> C
map("Num0", "VK_R");             // 0 -> R

// === Function Keys ===
map("F1", "VK_LMeta");           // F1 -> LWin
map("F2", "VK_Escape");          // F2 -> Escape
map("F3", "VK_LCtrl");           // F3 -> LCtrl
map("F4", "VK_LAlt");            // F4 -> LAlt
map("F5", "VK_Backspace");       // F5 -> Backspace
map("F6", "VK_Delete");          // F6 -> Delete
map("F8", "VK_Tab");             // F8 -> Tab
map("F9", "VK_Tab");             // F9 -> Tab
map("F10", "VK_Tab");            // F10 -> Tab
map("F11", "VK_Tab");            // F11 -> Tab
map("F12", "VK_Tab");            // F12 -> Tab

// === Special Keys ===
map("Space", "VK_Space");        // Space passthrough (blocked by MD_00 Space->F12)
map("Escape", "VK_Num5");        // Escape -> 5
map("Backspace", "VK_Delete");   // Backspace -> Delete
map("Delete", "VK_Num4");        // Delete -> 4
map("Enter", "VK_Yen");          // Enter -> Yen (円キー)

// === Modifier Keys ===
// Note: LCtrl is TapHold (MD_05), Tab is TapHold (MD_07)
map("LAlt", "VK_LCtrl");         // LAlt -> LCtrl

// === Punctuation ===
map("Minus", "VK_L");            // Minus -> L
map("Semicolon", "VK_V");        // ; -> V
map("Comma", "VK_F9");           // , -> F9
map("Period", "VK_F10");         // . -> F10
map("Slash", "VK_F11");          // / -> F11
map("Ro", "VK_Minus");           // ろ (JIS Ro) -> - (hyphen)

// === Japanese IME Keys (日本語キーボード) ===
map("無変換", "VK_Space");       // 無変換 (Muhenkan) -> Space
map("変換", "VK_RAlt");          // 変換 (Henkan) -> RAlt
map("ひらがな", "VK_RCtrl");     // ひらがな (Hiragana) -> RCtrl
map("カタカナ", "VK_RShift");    // カタカナ (Katakana) -> RShift
map("全角", "VK_Escape");        // 全角/半角 (Zenkaku) -> Escape
map("PrintScreen", "VK_Zenkaku"); // PrintScreen -> 全角/半角

// ============================================================
// Dvorak-Japanese IME Optimization (窓使いの憂鬱 DvorakY)
// Remap low-frequency bottom-row keys to high-frequency kana combos
// Active only when Japanese IME is on
// ============================================================
// Physical keys S,D,F,G,H = Dvorak ;,Q,J,K,X
when_start(["IME", "LANG_JA"]);
  sequence("S", ["VK_Y", "VK_A"]);  // phys S (Dvk ;) -> ya (や)
  sequence("D", ["VK_Y", "VK_O"]);  // phys D (Dvk Q) -> yo (よ)
  sequence("F", ["VK_Y", "VK_E"]);  // phys F (Dvk J) -> ye (いぇ)
  sequence("G", ["VK_Y", "VK_U"]);  // phys G (Dvk K) -> yu (ゆ)
  sequence("H", ["VK_Y", "VK_I"]);  // phys H (Dvk X) -> yi (い)
  map("Num9", "VK_K");              // phys 9 (Dvk C) -> k (か行 prefix)
when_end();

device_end();
